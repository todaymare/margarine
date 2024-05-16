use std::{ops::Deref, ptr::null_mut};

use llvm_sys::core::{LLVMCountStructElementTypes, LLVMGetParamTypes};

use crate::tys::{strct::StructTy, Type, TypeKind};

use super::Value;

#[derive(Clone, Copy, Debug)]
pub struct Struct<'ctx>(Value<'ctx>);


impl<'ctx> Struct<'ctx> {
    /// # Safety
    /// Undefined behaviour if the value isn't a struct
    pub unsafe fn new(val: Value<'ctx>) -> Self {
        debug_assert_eq!(val.ty().kind(), TypeKind::Struct);

        Self(val)
    }


    pub fn ty(self) -> StructTy<'ctx> { unsafe { StructTy::new(self.deref().ty()) } }
}


impl<'ctx> Deref for Struct<'ctx> {
    type Target = Value<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
