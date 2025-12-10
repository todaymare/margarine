use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct FPTy<'ctx>(Type<'ctx>);

impl<'ctx> FPTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a fp
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::F32 | TypeKind::F64));

        Self(ty)
    }
}


impl<'ctx> Deref for FPTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
