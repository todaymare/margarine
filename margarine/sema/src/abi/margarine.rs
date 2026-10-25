use llvm_api::{
    builder::Builder,
    tys::func::FunctionType,
    values::{func::FunctionPtr, ptr::Ptr, Value},
};
use parser::nodes::expr::ExprId;
use sti::ext::FromIn;

use crate::{
    abi::DefinitionBinding,
    llvm_codegen::{Conversion, Env},
    syms::{func::FunctionArgument, ty::Type},
};

#[derive(Clone)]
pub(crate) struct LoweredSignature<'ctx> {
    pub function_type: FunctionType<'ctx>,
}

pub(crate) fn lower_callable<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    ret: Type,
    args: &[(Type, bool)],
) -> LoweredSignature<'ctx> {
    let boundary_ret = conv.logical_llvm_type(ret);
    let mut source_args = std::vec::Vec::with_capacity(args.len() + 1);
    for &(arg, inout) in args {
        let ty = if inout {
            *conv.llvm_context().ptr()
        } else {
            conv.logical_llvm_type(arg)
        };
        source_args.push(ty);
    }
    source_args.push(*conv.llvm_context().ptr());

    LoweredSignature {
        function_type: boundary_ret.fn_ty(conv.llvm_context().arena, &source_args, false),
    }
}

pub(crate) fn bind_definition<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    env: &mut Env<'_, 'ctx>,
    builder: &mut Builder<'ctx>,
    args: &[FunctionArgument<'_>],
    arg_types: &[Type],
) -> DefinitionBinding<'ctx> {
    assert_eq!(args.len(), arg_types.len());

    for (i, arg) in args.iter().enumerate() {
        let arg_ty = arg_types[i];
        let llvm_ty = conv.logical_llvm_type(arg_ty);
        let param = builder.arg(i).unwrap();
        if arg.is_inout() {
            let local = builder.local(llvm_ty);
            let value = builder.load(builder.local_get(param).as_ptr(), llvm_ty);
            builder.local_set(local, value);
            env.alloc_var(arg.name(), local, arg_ty, true);
            env.add_inout(param, local, arg_ty);
            continue;
        }

        let value = builder.local_get(param);
        let local = builder.local(llvm_ty);
        let owned = conv.emit_copy(builder, value, arg_ty);
        builder.local_set(local, owned);
        env.alloc_var(arg.name(), local, arg_ty, false);
    }
    DefinitionBinding {
        sret: None,
    }
}

pub(crate) fn function_value<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    function: FunctionPtr<'ctx>,
) -> Value<'ctx> {
    let null = builder.ptr_null();
    function_value_with_env(conv, builder, function, null)
}

pub(crate) fn function_value_with_env<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    function: FunctionPtr<'ctx>,
    environment: Ptr<'ctx>,
) -> Value<'ctx> {
    *builder.struct_instance(conv.function_ref_type(), [*function, *environment])
}

pub(crate) fn copy_function_value<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    value: Value<'ctx>,
) -> Value<'ctx> {
    let func_ref = value.as_struct();
    let capture_ptr = builder.field_load(func_ref, 1).as_ptr();
    let is_closure = builder.bool_not(builder.ptr_is_null(capture_ptr));
    let slot = builder.alloca_store(value);
    builder.iff(is_closure, |builder| {
        let cloned = conv.clone_capture(builder, capture_ptr);
        let env_ptr = builder.field_ptr(slot, conv.function_ref_type(), 1);
        builder.store(env_ptr, *cloned);
    });
    builder.load(slot, value.ty())
}

pub(crate) fn drop_function_value<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    value: Value<'ctx>,
) {
    let func_ref = value.as_struct();
    let capture_ptr = builder.field_load(func_ref, 1).as_ptr();
    let is_closure = builder.bool_not(builder.ptr_is_null(capture_ptr));
    builder.iff(is_closure, |builder| {
        let ctx = conv.llvm_context();
        let capture_header = ctx.literal_struct(&[*conv.usize_type(), *ctx.ptr()], false);
        let drop_fn_ptr = builder.field_ptr(capture_ptr, capture_header, 1);
        let drop_fn_val = builder.load(drop_fn_ptr, *ctx.ptr());
        let drop_fn_ptr = drop_fn_val.as_func();
        let drop_fn_ty = ctx.void().fn_ty(ctx.arena, &[*ctx.ptr()], false);
        builder.call(drop_fn_ptr, drop_fn_ty, &[*capture_ptr]);
    });
}

pub(crate) fn emit_definition_return<'ctx>(
    builder: &Builder<'ctx>,
    value: Value<'ctx>,
) {
    builder.ret(value);
}

pub(crate) fn emit_call<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    func_ty: FunctionType<'ctx>,
    callee: Value<'ctx>,
    values: &[(Value<'ctx>, Type, Option<ExprId>)],
) -> Value<'ctx> {
    let func_ref = callee.as_struct();
    let func_ptr = builder.field_load(func_ref, 0).as_func();
    let func_env = builder.field_load(func_ref, 1).as_ptr();
    let llvm_args = sti::vec::Vec::from_in(
        conv.llvm_context().arena,
        values.iter().map(|value| value.0).chain(core::iter::once(*func_env)),
    );
    builder.call(func_ptr, func_ty, &llvm_args)
}
pub(crate) fn emit_startup_call<'ctx>(
    builder: &Builder<'ctx>,
    function: FunctionPtr<'ctx>,
    function_type: FunctionType<'ctx>,
    values: &[crate::abi::StartupArg<'ctx>],
) {
    let args = values
        .iter()
        .map(|value| value.value)
        .chain(core::iter::once(*builder.ptr_null()))
        .collect::<std::vec::Vec<_>>();
    builder.call(function, function_type, &args);
}
