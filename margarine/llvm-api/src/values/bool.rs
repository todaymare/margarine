use std::ops::Deref;

use crate::tys::{bool::BoolTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Bool<'ctx>(Value<'ctx>);


impl<'ctx> Bool<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a bool
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Integer);

        Self(val)
    }


    pub fn ty(self) -> BoolTy <'ctx> { unsafe { BoolTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Bool<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
