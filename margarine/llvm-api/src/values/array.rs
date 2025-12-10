use std::ops::Deref;

use crate::tys::{array::ArrayTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Array<'ctx>(Value<'ctx>);


impl<'ctx> Array<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a struct
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Array);

        Self(val)
    }


    pub fn ty(self) -> ArrayTy<'ctx> { unsafe { ArrayTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Array<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
