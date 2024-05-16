use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct UnitTy<'ctx>(Type<'ctx>);

impl<'ctx> UnitTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a unit type
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Integer));

        Self(ty)
    }
}


impl<'ctx> Deref for UnitTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
