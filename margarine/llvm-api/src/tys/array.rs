use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct ArrayTy<'ctx>(Type<'ctx>);

impl<'ctx> ArrayTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a fp
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Array));

        Self(ty)
    }
}


impl<'ctx> Deref for ArrayTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
