use std::{ops::Deref, ptr::NonNull};

use llvm_sys::core::{LLVMAddAttributeAtIndex, LLVMCountStructElementTypes, LLVMCreateEnumAttribute, LLVMGetEnumAttributeKindForName, LLVMGetStructElementTypes, LLVMIsOpaqueStruct, LLVMStructSetBody};
use sti::arena::Arena;

use crate::{cstr, tys::TypeKind};

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


    pub fn set_fields(self, fields: &[Type<'ctx>], packed: bool) {
        assert!(self.is_opaque());

        // &[Type] == &[*mut LLVMType]
        let mut vec = vec![];
        for t in fields { vec.push(unsafe { t.llvm_ty() }.as_ptr()) }
        

        unsafe { LLVMStructSetBody(self.llvm_ty().as_ptr(),
                                   vec.as_mut_ptr(), fields.len() as u32, packed as i32) };
    }


    pub fn fields_count(self) -> usize {
        unsafe { LLVMCountStructElementTypes(self.llvm_ty().as_ptr()) as usize }
    }


    pub fn fields<'a>(self, arena: &'a Arena) -> sti::vec::Vec<Type<'ctx>, &'a Arena> {
        let argc = self.fields_count();

        let mut args = sti::vec::Vec::with_cap_in(arena, argc);
        unsafe { LLVMGetStructElementTypes(self.llvm_ty().as_ptr(), args.as_mut_ptr()) };
        unsafe { args.set_len(argc) };

        let args = {
            let mut vec = sti::vec::Vec::with_cap_in(arena, argc);
            for (_, i) in args {
                vec.push(Type::new(NonNull::new(i).unwrap())); 
            };
            vec
        };

        args
    }

}


impl<'ctx> Deref for StructTy<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
