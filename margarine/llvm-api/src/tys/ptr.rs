use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct PtrTy<'ctx>(Type<'ctx>);

impl<'ctx> PtrTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a ptr 
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Ptr);

        Self(ty)
    }
}


impl<'ctx> Deref for PtrTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

