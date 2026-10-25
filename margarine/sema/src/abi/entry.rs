use llvm_api::{
    tys::{func::FunctionType, integer::IntegerTy},
    values::func::FunctionPtr,
};

use crate::{
    llvm_codegen::{CompilationTarget, Conversion},
    syms::sym_map::SymbolId,
};

pub(crate) struct ProgramEntry<'ctx> {
    function: FunctionPtr<'ctx>,
    function_type: FunctionType<'ctx>,
}

pub(crate) fn declare_program_entry<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    shared: bool,
    i32_ty: IntegerTy<'ctx>,
) -> Option<ProgramEntry<'ctx>> {
    if shared {
        return None;
    }

    let ctx = conv.llvm_context();
    let function_type = if matches!(conv.target(), CompilationTarget::Wasm32UnknownUnknown) {
        (*i32_ty).fn_ty(ctx.arena, &[], false)
    } else {
        (*i32_ty).fn_ty(ctx.arena, &[*i32_ty, *ctx.ptr()], false)
    };
    let function = conv.llvm_module().function("main", function_type);
    function.set_c_calling_convention();
    Some(ProgramEntry { function, function_type })
}

pub(crate) fn emit_program_entry<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    entry: ProgramEntry<'ctx>,
    startups: &[SymbolId],
    abort_fn: FunctionPtr<'ctx>,
    abort_fn_ty: FunctionType<'ctx>,
) {
    let ctx = conv.llvm_context();
    let builder = entry.function.builder(ctx, entry.function_type);
    if !matches!(conv.target(), CompilationTarget::Wasm32UnknownUnknown) {
        let void = ctx.void();
        let i32_ty = ctx.integer(32);
        let set_env_args_fn_ty = (*void).fn_ty(ctx.arena, &[*i32_ty, *ctx.ptr()], false);
        let set_env_args_fn = conv.llvm_module().function("margarineSetEnvArgs", set_env_args_fn_ty);
        set_env_args_fn.set_linkage(llvm_api::values::func::Linkage::External);
        let argc = builder.local_get(builder.arg(0).unwrap());
        let argv = builder.local_get(builder.arg(1).unwrap());
        builder.call(set_env_args_fn, set_env_args_fn_ty, &[argc, argv]);
    }

    for &sym_id in startups {
        if let Some((func_ptr, func_ty)) = conv.compiled_function(sym_id) {
            super::emit_startup_call(conv, &builder, sym_id, func_ptr, func_ty);
        }
    }

    builder.call(abort_fn, abort_fn_ty, &[*ctx.const_int(ctx.integer(32), 0, false)]);
    builder.unreachable();
}
