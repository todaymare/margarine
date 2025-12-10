use std::ops::Deref;

use crate::tys::{fp::FPTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct FP<'ctx>(Value<'ctx>);


impl<'ctx> FP<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a fp
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert!(matches!(val.ty().kind(), TypeKind::F32 | TypeKind::F64));

        Self(val)
    }


    pub fn ty(self) -> FPTy<'ctx> { unsafe { FPTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for FP<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

