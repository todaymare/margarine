pub mod func;
pub mod ptr;
pub mod bool;
pub mod int;
pub mod fp;
pub mod strct;
pub mod unit;
pub mod union;
pub mod array;
pub mod string;
pub mod global;

use std::{marker::PhantomData, ptr::NonNull};

use llvm_sys::{core::{LLVMPrintValueToString, LLVMTypeOf}, LLVMValue};

use crate::{info::Message, tys::{Type, TypeKind}};

use self::{array::Array, bool::Bool, fp::FP, func::FunctionPtr, int::Integer, ptr::Ptr, strct::Struct, union::Union, unit::Unit};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Value<'ctx> {
    ptr: NonNull<LLVMValue>,
    phantom: PhantomData<&'ctx ()>,
}


impl<'ctx> Value<'ctx> {
    pub fn new(ptr: NonNull<LLVMValue>) -> Self { Self { ptr, phantom: PhantomData } }


    pub fn ty(self) -> Type<'ctx> {
        let ty = unsafe { LLVMTypeOf(self.ptr.as_ptr()) };
        let ty = NonNull::new(ty).expect("unable to retrieve type information from a value");
        Type::new(ty)
    }


    pub fn name(self) -> Message {
        let msg = unsafe { LLVMPrintValueToString(self.ptr.as_ptr()) };
        let msg = NonNull::new(msg).expect("failed to retrieve the name of the value");
        unsafe { Message::new(msg) }
    }


    pub unsafe fn llvm_val(self) -> NonNull<LLVMValue> { self.ptr }


    pub fn as_integer(self) -> Integer<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Integer);
        unsafe { Integer::new(self) }
    }


    pub fn as_fp(self) -> FP<'ctx> {
        assert!(matches!(self.ty().kind(), TypeKind::F32 | TypeKind::F64));
        unsafe { FP::new(self) }
    }


    pub fn as_func(self) -> FunctionPtr<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Function);
        unsafe { FunctionPtr::new(self) }
    }


    pub fn as_ptr(self) -> Ptr<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Ptr);
        unsafe { Ptr::new(self) }
    }


    pub fn as_struct(self) -> Struct<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Struct);
        unsafe { Struct::new(self) }
    }


    pub fn as_union(self) -> Union<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Struct);
        unsafe { Union::new(self) }
    }


    pub fn as_bool(self) -> Bool<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Integer);
        assert_eq!(self.as_integer().ty().bit_size(), 1);
        unsafe { Bool::new(self) }
    }


    pub fn as_unit(self) -> Unit<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Integer);
        assert_eq!(self.as_integer().ty().bit_size(), 1);
        unsafe { Unit::new(self) }
    }


    pub fn as_array(self) -> Array<'ctx> {
        assert_eq!(self.ty().kind(), TypeKind::Array);
        unsafe { Array::new(self) }
    }
}


impl<'ctx> core::fmt::Debug for Value<'ctx> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name().as_str())
    }
}

