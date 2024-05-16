pub mod integer;
pub mod fp;
pub mod bool;
pub mod func;
pub mod ptr;
pub mod strct;
pub mod unit;
pub mod void;
pub mod union;
pub mod array;
pub mod string;

use std::{marker::PhantomData, ptr::NonNull};

use llvm_sys::{core::{LLVMFunctionType, LLVMGetTypeKind, LLVMPrintTypeToString, LLVMTypeIsSized}, target::{LLVMABISizeOfType, LLVMGetModuleDataLayout}, LLVMType};
use sti::{arena::Arena, traits::FromIn};

use crate::{info::Message, module::Module, values::{func::FunctionPtr, int::Integer}};

use self::{array::ArrayTy, bool::BoolTy, fp::FPTy, func::FunctionType, integer::IntegerTy, ptr::PtrTy, strct::StructTy, union::UnionTy, unit::UnitTy, void::Void};

#[derive(Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Type<'ctx> {
    ptr: NonNull<LLVMType>,
    phantom: PhantomData<&'ctx ()>,
}


impl<'ctx> Type<'ctx> {
    pub fn new(ty: NonNull<LLVMType>) -> Self {
        Self { ptr: ty, phantom: PhantomData }
    }


    pub fn kind(self) -> TypeKind {
        use llvm_sys::LLVMTypeKind::*;

        let ty_kind = unsafe { LLVMGetTypeKind(self.ptr.as_ptr()) };

        match ty_kind {
            LLVMIntegerTypeKind  => TypeKind::Integer,
            LLVMVoidTypeKind     => TypeKind::Void,
            LLVMFloatTypeKind    => TypeKind::F32,
            LLVMDoubleTypeKind   => TypeKind::F64,
            LLVMFunctionTypeKind => TypeKind::Function,
            LLVMPointerTypeKind  => TypeKind::Ptr,
            LLVMStructTypeKind   => TypeKind::Struct,
            LLVMArrayTypeKind    => TypeKind::Array,

            _ => unimplemented!(),
        }
    }


    pub fn name(self) -> Message {
        let msg = unsafe { LLVMPrintTypeToString(self.ptr.as_ptr()) };
        let msg = NonNull::new(msg).expect("failed to retrieve the name of the type");
        unsafe { Message::new(msg) }
    }


    pub fn fn_ty(self, args: &[Type<'ctx>], is_variadic: bool) -> FunctionType<'ctx> {
        let pool = Arena::tls_get_temp();
        let mut args = sti::vec::Vec::from_in(&*pool, args.iter().map(|x| x.ptr.as_ptr()));

        let ty = unsafe { LLVMFunctionType(self.ptr.as_ptr(),
                                            args.as_mut_ptr(),
                                            args.len().try_into().expect("too many arguments"),
                                            is_variadic as i32) };

        let ty = NonNull::new(ty).expect("failed to create a function type");

        unsafe { FunctionType::new(Type::new(ty)) }
    }


    pub fn is_sized(self) -> bool {
        unsafe { LLVMTypeIsSized(self.ptr.as_ptr()) == 1 }
    }


    pub fn size_of(self, module: Module<'ctx>) -> Option<usize> {
        let dl = unsafe { LLVMGetModuleDataLayout(module.ptr.as_ptr()) };
        if dl.is_null() { panic!("data layout is not set"); }

        if !self.is_sized() { return None };
        Some(unsafe { LLVMABISizeOfType(dl, self.ptr.as_ptr()) as usize })
    }


    pub unsafe fn llvm_ty(self) -> NonNull<LLVMType> { self.ptr }


    pub fn as_void(self) -> Void<'ctx> {
        assert_eq!(self.kind(), TypeKind::Void);
        unsafe { Void::new(self) }
    }


    pub fn as_integer(self) -> IntegerTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Integer);
        unsafe { IntegerTy::new(self) }
    }


    pub fn as_fp(self) -> FPTy<'ctx> {
        assert!(matches!(self.kind(), TypeKind::F32 | TypeKind::F64));
        unsafe { FPTy::new(self) }
    }


    pub fn as_func(self) -> FunctionType<'ctx> {
        assert_eq!(self.kind(), TypeKind::Function);
        unsafe { FunctionType::new(self) }
    }


    pub fn as_ptr(self) -> PtrTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Ptr);
        unsafe { PtrTy::new(self) }
    }


    pub fn as_struct(self) -> StructTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Struct);
        unsafe { StructTy::new(self) }
    }


    pub fn as_union(self) -> UnionTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Struct);
        unsafe { UnionTy::new(self) }
    }


    pub fn as_bool(self) -> BoolTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Integer);
        assert_eq!(self.as_integer().bit_size(), 1);
        unsafe { BoolTy::new(self) }
    }


    pub fn as_unit(self) -> UnitTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Integer);
        assert_eq!(self.as_integer().bit_size(), 1);
        unsafe { UnitTy::new(self) }
    }


    pub fn as_array(self) -> ArrayTy<'ctx> {
        assert_eq!(self.kind(), TypeKind::Array);
        unsafe { ArrayTy::new(self) }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TypeKind {
    Void,
    Integer,
    F32,
    F64,
    Function,
    Ptr,
    Struct,
    Array,
}


impl<'ctx> core::fmt::Debug for Type<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name().as_str())
    }
}
