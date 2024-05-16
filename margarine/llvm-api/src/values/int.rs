use std::ops::Deref;

use crate::tys::{integer::IntegerTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Integer<'ctx>(Value<'ctx>);


impl<'ctx> Integer<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't an integer
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Integer);

        Self(val)
    }


    pub fn ty(self) -> IntegerTy<'ctx> { unsafe { IntegerTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Integer<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
