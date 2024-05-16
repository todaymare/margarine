use std::{ops::Deref, ptr::{null_mut, NonNull}};

use llvm_sys::{core::{LLVMCountStructElementTypes, LLVMGetStructElementTypes, LLVMIsOpaqueStruct, LLVMStructSetBody}, LLVMType};

use crate::{module::Module, tys::TypeKind, values::Value};

use super::Type;

#[derive(Debug, Clone, Copy)]
pub struct StructTy<'ctx>(Type<'ctx>);


impl<'ctx> StructTy<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a fp
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert!(matches!(ty.kind(), TypeKind::Struct));

        Self(ty)
    }


    pub fn is_opaque(self) -> bool {
        unsafe { LLVMIsOpaqueStruct(self.llvm_ty().as_ptr()) == 1 } 
    }


    pub fn set_fields(self, fields: &[Type<'ctx>]) {
        assert!(self.is_opaque());

        // &[Type] == &[*mut LLVMType]
        let mut vec = vec![];
        for t in fields { vec.push(unsafe { t.llvm_ty() }.as_ptr()) }
        

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   vec.as_mut_ptr(), fields.len() as u32, 0) };
        dbg!(self);
    }


    pub fn fields_count(self) -> usize {
        unsafe { LLVMCountStructElementTypes(self.llvm_ty().as_ptr()) as usize }
    }


    pub fn fields(self) -> Vec<Type<'ctx>> {
        let argc = self.fields_count();

        let mut args = Vec::with_capacity(argc);
        unsafe { LLVMGetStructElementTypes(self.llvm_ty().as_ptr(), args.as_mut_ptr()) };
        unsafe { args.set_len(argc) };

        let args = {
            let mut vec = Vec::with_capacity(argc);
            for i in args { vec.push(Type::new(NonNull::new(i).unwrap())) };
            vec
        };

        args
    }

}


impl<'ctx> Deref for StructTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
