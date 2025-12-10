use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct BoolTy<'ctx>(Type<'ctx>);

impl<'ctx> BoolTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a bool
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Integer);

        Self(ty)
    }
}


impl<'ctx> Deref for BoolTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
