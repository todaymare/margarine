use std::collections::HashMap;

use common::string_map::StringIndex;
use llvm_api::{
    builder::Builder,
    ctx::ContextRef,
    module::Module,
    tys::{func::FunctionType, Type as LLVMType},
    values::{func::FunctionPtr, ptr::Ptr, Value},
};
use parser::{
    dt::FunctionAbi,
    nodes::{decl::DeclId, expr::ExprId},
};

use crate::{
    llvm_codegen::{Conversion, Env},
    syms::{
        func::{FunctionArgument, FunctionTy},
        sym_map::{GenListId, SymbolId},
        ty::{Type, TypeHash},
    },
};

#[derive(Default)]
pub(crate) struct TypeMappings<'ctx> {
    c: HashMap<TypeHash, LLVMType<'ctx>>,
}

#[derive(Clone, Copy)]
pub(crate) struct StartupArg<'ctx> {
    pub(crate) value: Value<'ctx>,
    pub(crate) ty: Type,
    pub(crate) inout: bool,
}


mod c;
mod entry;
mod margarine;

pub(crate) use entry::{declare_program_entry, emit_program_entry};

#[derive(Clone)]
pub(crate) struct Callable<'ctx> {
    lowered: Lowered<'ctx>,
    ret: Type,
}

#[derive(Clone)]
enum Lowered<'ctx> {
    Margarine(margarine::LoweredSignature<'ctx>),
    C(c::LoweredSignature<'ctx>),
}

#[derive(Clone, Copy)]
pub(crate) struct DefinitionBinding<'ctx> {
    pub(crate) sret: Option<Ptr<'ctx>>,
}
impl<'ctx> Callable<'ctx> {
    pub(crate) fn function_type(&self) -> FunctionType<'ctx> {
        match &self.lowered {
            Lowered::Margarine(signature) => signature.function_type,
            Lowered::C(signature) => signature.function_type,
        }
    }

    pub(crate) fn declare_extern(&self, module: Module<'ctx>, name: &str) -> FunctionPtr<'ctx> {
        module
            .get_function(name)
            .unwrap_or_else(|| module.function(name, self.function_type()))
    }

    pub(crate) fn define(&self, module: Module<'ctx>, name: &str) -> FunctionPtr<'ctx> {
        match &self.lowered {
            Lowered::Margarine(_) => module.function(name, self.function_type()),
            Lowered::C(_) => module.function_definition(name, self.function_type()),
        }
    }

    pub(crate) fn configure(
        &self,
        ctx: ContextRef<'ctx>,
        function: FunctionPtr<'ctx>,
        is_never: bool,
    ) {
        if let Lowered::C(signature) = &self.lowered {
            c::configure(ctx, function, signature);
        }
        if is_never {
            function.set_noreturn(ctx);
        }
    }

    pub(crate) fn definition_name(
        &self,
        conv: &Conversion<'_, '_, '_, '_, 'ctx>,
        default: StringIndex,
        decl: DeclId,
    ) -> StringIndex {
        match &self.lowered {
            Lowered::Margarine(_) => default,
            Lowered::C(_) => c::definition_name(conv, default, decl),
        }
    }
}


pub(crate) fn lower_callable<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    function: FunctionTy<'_>,
    ret: Type,
    args: &[(Type, bool)],
) -> Callable<'ctx> {
    let lowered = match function.abi() {
        FunctionAbi::C => Lowered::C(c::lower_callable(conv, ret, args)),
        _ => Lowered::Margarine(margarine::lower_callable(conv, ret, args)),
    };

    Callable { lowered, ret }
}

pub(crate) fn function_value_type<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    abi: FunctionAbi,
) -> LLVMType<'ctx> {
    match abi {
        FunctionAbi::C => *conv.llvm_context().ptr(),
        _ => *conv.function_ref_type(),
    }
}

pub(crate) fn emit_inout_writeback<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    callable: Option<&Callable<'ctx>>,
    ty: Type,
    destination: Ptr<'ctx>,
    value: Value<'ctx>,
) {
    match callable.map(|callable| &callable.lowered) {
        Some(Lowered::C(_)) => c::emit_inout_writeback(conv, builder, ty, destination, value),
        _ => builder.store(destination, value),
    }
}

pub(crate) fn function_value<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    ty: Type,
    function: FunctionPtr<'ctx>,
) -> Value<'ctx> {
    match conv
        .function_definition(ty)
        .map(|function| function.abi())
    {
        Some(FunctionAbi::C) => c::function_value(function),
        _ => margarine::function_value(conv, builder, function),
    }
}
pub(crate) fn function_value_with_env<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    ty: Type,
    function: FunctionPtr<'ctx>,
    environment: Ptr<'ctx>,
) -> Value<'ctx> {
    match conv
        .function_definition(ty)
        .map(|function| function.abi())
    {
        Some(FunctionAbi::C) => c::function_value(function),
        _ => margarine::function_value_with_env(conv, builder, function, environment),
    }
}

pub(crate) fn copy_function_value<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    ty: Type,
    value: Value<'ctx>,
) -> Value<'ctx> {
    match conv
        .function_definition(ty)
        .map(|function| function.abi())
    {
        Some(FunctionAbi::C) => value,
        _ => margarine::copy_function_value(conv, builder, value),
    }
}

pub(crate) fn drop_function_value<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    ty: Type,
    value: Value<'ctx>,
) {
    if !matches!(
        conv.function_definition(ty).map(|function| function.abi()),
        Some(FunctionAbi::C),
    ) {
        margarine::drop_function_value(conv, builder, value);
    }
}

pub(crate) fn bind_definition<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    env: &mut Env<'_, 'ctx>,
    builder: &mut Builder<'ctx>,
    callable: &Callable<'ctx>,
    args: &[FunctionArgument<'_>],
    arg_types: &[Type],
) -> DefinitionBinding<'ctx> {
    match &callable.lowered {
        Lowered::Margarine(_) => {
            margarine::bind_definition(conv, env, builder, args, arg_types)
        }
        Lowered::C(signature) => {
            c::bind_definition(conv, env, builder, signature, args, arg_types)
        }
    }
}


pub(crate) fn emit_definition_return<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    callable: &Callable<'ctx>,
    binding: &DefinitionBinding<'ctx>,
    value: Value<'ctx>,
) {
    match &callable.lowered {
        Lowered::Margarine(_) => margarine::emit_definition_return(builder, value),
        Lowered::C(signature) => {
            c::emit_definition_return(conv, builder, signature, callable.ret, binding, value)
        }
    }
}

pub(crate) fn emit_call<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    callable: &Callable<'ctx>,
    callee: Value<'ctx>,
    values: &[(Value<'ctx>, Type, Option<ExprId>)],
) -> Value<'ctx> {
    match &callable.lowered {
        Lowered::Margarine(signature) => {
            margarine::emit_call(conv, builder, signature.function_type, callee, values)
        }
        Lowered::C(signature) => {
            c::emit_call(conv, builder, signature, callable.ret, callee, values)
        }
    }
}
pub(crate) fn emit_startup_call<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    symbol: SymbolId,
    function: FunctionPtr<'ctx>,
    _function_type: FunctionType<'ctx>,
) {
    let ty = Type::Ty(symbol, GenListId::EMPTY);
    let (function_ty, ret, arg_types) = conv
        .resolved_function_signature(ty)
        .expect("startup symbol must refer to a function");
    let arg_modes = function_ty
        .args()
        .iter()
        .zip(&arg_types)
        .map(|(arg, ty)| (*ty, arg.is_inout()))
        .collect::<std::vec::Vec<_>>();
    let callable = lower_callable(conv, function_ty, ret, &arg_modes);
    let values = arg_types
        .iter()
        .zip(function_ty.args())
        .map(|(ty, arg)| StartupArg {
            value: if arg.is_inout() {
                *builder.alloca_store(builder.const_zero(conv.logical_llvm_type(*ty)))
            } else {
                builder.const_zero(conv.logical_llvm_type(*ty))
            },
            ty: *ty,
            inout: arg.is_inout(),
        })
        .collect::<std::vec::Vec<_>>();

    match &callable.lowered {
        Lowered::Margarine(signature) => {
            margarine::emit_startup_call(builder, function, signature.function_type, &values)
        }
        Lowered::C(signature) => {
            c::emit_startup_call(conv, builder, signature, ret, function, &values)
        }
    }
}
