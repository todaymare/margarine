use std::ops::Deref;

use llvm_sys::core::{LLVMSetInitializer, LLVMSetLinkage};

use crate::tys::{ptr::PtrTy, TypeKind};

use super::{func::Linkage, Value};

#[derive(Clone, Copy, Debug)]
pub struct GlobalPtr<'ctx>(Value<'ctx>);


impl<'ctx> GlobalPtr<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a global ptr
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Ptr);
        debug_assert!(val.as_ptr().is_global());

        Self(val)
    }


    pub fn ty(self) -> PtrTy<'ctx> { unsafe { PtrTy::new(self.deref().ty()) } }


    pub fn set_initialiser(self, initialiser: Value<'ctx>) {
        unsafe { LLVMSetInitializer(self.llvm_val().as_ptr(), initialiser.llvm_val().as_ptr()) }
    }


    pub fn set_linkage(self, linkage: Linkage) {
        unsafe { LLVMSetLinkage(self.llvm_val().as_ptr(), linkage.llvm_linkage()) }
    }
}


impl<'ctx> Deref for GlobalPtr<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
