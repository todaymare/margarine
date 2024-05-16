use std::ops::Deref;

use crate::tys::TypeKind;

use super::Type;

// hbabfkdioemzsofn azsiolsinf aocfmnfjuasokgkjsbnasmnvbitygtwooooooowwwww
// merhabajrfowjhwodfjasanasilsijdjaodfboranvjdngisxguhnaydin wsocfvfhrf
#[derive(Clone, Copy, Debug)]
pub struct StringTy<'ctx>(Type<'ctx>);

impl<'ctx> StringTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a string
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Array);

        Self(ty)
    }
}


impl<'ctx> Deref for StringTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
