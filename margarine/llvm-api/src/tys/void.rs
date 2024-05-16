use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct Void<'ctx>(Type<'ctx>);

impl<'ctx> Void<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't void
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Void);

        Self(ty)
    }
}


impl<'ctx> Deref for Void<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
