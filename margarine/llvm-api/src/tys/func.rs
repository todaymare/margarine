use std::{ops::Deref, ptr::{null_mut, NonNull}};

use llvm_sys::core::{LLVMCountParamTypes, LLVMGetParamTypes, LLVMGetReturnType};

use crate::tys::TypeKind;

use super::Type;

#[derive(Clone, Copy, Debug)]
pub struct FunctionType<'ctx>(Type<'ctx>);

impl<'ctx> FunctionType<'ctx> {

    /// # Safety
    /// Undefined behaviour if `ty` isn't a function
    pub unsafe fn new(ty: Type<'ctx>) -> Self {
        debug_assert_eq!(ty.kind(), TypeKind::Function);

        Self(ty)
    }


    pub fn argument_count(self) -> usize {
        unsafe { LLVMCountParamTypes(self.llvm_ty().as_ptr()) as usize }
    }


    pub fn ret(self) -> Type<'ctx> {
        Type::new(NonNull::new(unsafe { LLVMGetReturnType(self.llvm_ty().as_ptr()) }).unwrap())
    }


    pub fn args(self) -> Vec<Type<'ctx>> {
        let argc = self.argument_count();

        let mut args = Vec::with_capacity(argc);
        unsafe { LLVMGetParamTypes(self.llvm_ty().as_ptr(), args.as_mut_ptr()) };
        unsafe { args.set_len(argc) };

        let args = {
            let mut vec = Vec::with_capacity(argc);
            for i in args { vec.push(Type::new(NonNull::new(i).unwrap())) };
            vec
        };

        args
    }
}


impl<'ctx> Deref for FunctionType<'ctx> {
    type Target = Type<'ctx>;

    fn deref(&self) -> &Self::Target { &self.0 }
}
