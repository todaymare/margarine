use std::{ops::Deref, ptr::NonNull};

use llvm_sys::core::{LLVMGetElementType, LLVMGetVectorSize};

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct VectorTy<'ctx>(Type<'ctx>);

impl<'ctx> VectorTy<'ctx> {
    /// # Safety
    /// Undefined behaviour if `ty` isn't a vector
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Vector));

        Self(ty)
    }

    pub fn element_ty(self) -> Type<'ctx> {
        let ty = unsafe { LLVMGetElementType(self.llvm_ty().as_ptr()) };
        Type::new(NonNull::new(ty).unwrap())
    }

    pub fn element_count(self) -> u32 {
        unsafe { LLVMGetVectorSize(self.llvm_ty().as_ptr()) }
    }

    pub fn len(self) -> u32 {
        self.element_count()
    }
}

impl<'ctx> Deref for VectorTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
