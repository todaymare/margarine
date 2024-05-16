use std::ops::Deref;

use crate::tys::{bool::BoolTy, string::StringTy, TypeKind};

use super::{global::GlobalPtr, Value};

#[derive(Clone, Copy, Debug)]
pub struct StringValue<'ctx>(Value<'ctx>);


impl<'ctx> StringValue<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a string
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Array);

        Self(val)
    }


    pub fn ty(self) -> StringTy<'ctx> { unsafe { StringTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for StringValue<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}

