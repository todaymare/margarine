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

        let mut max = 0;

        dbg!(fields);
        for ty in fields {
            max = max.max(ty.size_of(module).unwrap());
        }

        let array = ctx.integer(max as u32 * 8);
        dbg!(array);

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   [array.llvm_ty().as_ptr()].as_mut_ptr(),
                                   1, 0) };
    }
}


impl<'ctx> Deref for UnionTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
