use std::ops::Deref;

use llvm_sys::core::{LLVMIsOpaqueStruct, LLVMStructSetBody};

use crate::{ctx::ContextRef, module::Module, tys::TypeKind};

use super::Type;

#[derive(Debug, Clone, Copy)]
pub struct UnionTy<'ctx>(Type<'ctx>);


impl<'ctx> UnionTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a union
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Struct));

        Self(ty)
    }


    pub fn is_opaque(self) -> bool {
        unsafe { LLVMIsOpaqueStruct(self.llvm_ty().as_ptr()) == 1 } 
    }


    pub fn set_fields(self, ctx: ContextRef<'ctx>, module: Module<'ctx>, fields: &[Type<'ctx>]) {
        assert!(self.is_opaque());

        let index_ty = ctx.integer(fields.len().count_ones().next_power_of_two());

        let mut max = 0;

        for ty in fields {
            dbg!(ty.size_of(module).unwrap());
            max = max.max(ty.size_of(module).unwrap());
        }

        let unit = ctx.integer(8);
        let array = ctx.array(*unit, max);

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   [index_ty.llvm_ty().as_ptr(), array.llvm_ty().as_ptr()].as_mut_ptr(),
                                   2, 0) };
    }
}


impl<'ctx> Deref for UnionTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
