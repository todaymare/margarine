use std::ops::Deref;

use crate::tys::{unit::UnitTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Unit<'ctx>(Value<'ctx>);


impl<'ctx> Unit<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a function
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Integer);

        Self(val)
    }


    pub fn ty(self) -> UnitTy<'ctx> { unsafe { UnitTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Unit<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
