use std::ops::Deref;

use llvm_sys::core::LLVMIsAGlobalValue;

use crate::tys::{ptr::PtrTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Ptr<'ctx>(Value<'ctx>);


impl<'ctx> Ptr<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a function
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Ptr);

        Self(val)
    }


    pub fn ty(self) -> PtrTy<'ctx> { unsafe { PtrTy::new(self.deref().ty()) } }
    

    pub fn is_global(self) -> bool {
        unsafe { !LLVMIsAGlobalValue(self.llvm_val().as_ptr()).is_null() } 
    }
}


impl<'ctx> Deref for Ptr<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
