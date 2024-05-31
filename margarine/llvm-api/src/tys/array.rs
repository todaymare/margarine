use std::{ops::Deref, ptr::NonNull};

use llvm_sys::core::LLVMGetElementType;

use crate::{module::Module, tys::TypeKind};

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


    pub fn element_ty(self) -> Type<'ctx> {
        let ty = unsafe { LLVMGetElementType(self.llvm_ty().as_ptr()) };
        Type::new(NonNull::new(ty).unwrap())
    }


    pub fn len(self, module: Module<'ctx>) -> usize {
        let arr_size = self.size_of(module).unwrap();
        let elem_size = self.element_ty().size_of(module).unwrap();

        let len = arr_size / elem_size;
        len as usize
    }
}


impl<'ctx> Deref for ArrayTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
