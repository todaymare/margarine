use std::ops::Deref;

use crate::tys::{union::UnionTy, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Union<'ctx>(Value<'ctx>);


impl<'ctx> Union<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a union
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Struct);

        Self(val)
    }


    pub fn ty(self) -> UnionTy<'ctx> { unsafe { UnionTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Union<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
