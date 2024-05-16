use std::ops::Deref;

use llvm_sys::core::LLVMGetIntTypeWidth;

use crate::tys::TypeKind;

use super::Type;

#[derive(Debug, Clone, Copy)]
pub struct IntegerTy<'ctx>(Type<'ctx>);

impl<'ctx> IntegerTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if the type isn't an integer
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Integer);

        Self(ty)
    }


    pub fn bit_size(self) -> usize {
        unsafe { LLVMGetIntTypeWidth(self.llvm_ty().as_ptr()) as usize }
    }
}


impl<'ctx> Deref for IntegerTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
