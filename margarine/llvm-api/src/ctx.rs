use std::{any::Any, ops::Deref, ptr::NonNull};

use llvm_sys::{core::{LLVMArrayType2, LLVMConstArray2, LLVMConstInt, LLVMConstNamedStruct, LLVMConstReal, LLVMConstStringInContext, LLVMConstStructInContext, LLVMContextCreate, LLVMContextDispose, LLVMDoubleType, LLVMDoubleTypeInContext, LLVMFloatType, LLVMFloatTypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerTypeInContext, LLVMStructCreateNamed, LLVMVoidTypeInContext}, LLVMContext};
use sti::{arena::Arena, format_in};

use crate::{module::Module, tys::{array::ArrayTy, bool::BoolTy, fp::FPTy, integer::IntegerTy, ptr::PtrTy, strct::StructTy, union::UnionTy, unit::UnitTy, void::Void, Type}, values::{array::Array, bool::Bool, fp::FP, int::Integer, strct::Struct, string::StringValue, unit::Unit, Value}};

pub struct Context<'ctx>(ContextRef<'ctx>);

#[derive(Clone, Copy)]
pub struct ContextRef<'ctx>(ContextImpl<'ctx>);


#[derive(Clone, Copy)]
pub struct ContextImpl<'me> {
    pub(crate) ptr: NonNull<LLVMContext>,

    // cache
    f32: FPTy<'me>,
    f64: FPTy<'me>,
    bool: BoolTy<'me>,
    unit: UnitTy<'me>,
    void: Void<'me>,
}


impl<'ctx> Context<'ctx> {
    pub fn new() -> Self {
        Self(ContextRef(ContextImpl::new()))
    }
}


impl<'me> ContextImpl<'me> {
    fn new() -> Self {
        let ptr = unsafe { LLVMContextCreate() };
        let ctx = NonNull::new(ptr).expect("failed to create an llvm context");

        let f32 = unsafe { LLVMFloatTypeInContext(ptr) };
        let f32 = NonNull::new(f32).expect("failed to create an f32 type");
        let f32 = unsafe { FPTy::new(Type::new(f32)) };

        let f64 = unsafe { LLVMDoubleTypeInContext(ptr) };
        let f64 = NonNull::new(f64).expect("failed to create an f64 type");
        let f64 = unsafe { FPTy::new(Type::new(f64)) };

        let bool = unsafe { LLVMIntTypeInContext(ptr, 1) };
        let bool = NonNull::new(bool).expect("failed to create a bool type");
        let bool = unsafe { BoolTy::new(Type::new(bool)) };

        let unit = unsafe { LLVMIntTypeInContext(ptr, 1) };
        let unit = NonNull::new(unit).expect("failed to create a unit type");
        let unit = unsafe { UnitTy::new(Type::new(unit)) };

        let void = unsafe { LLVMVoidTypeInContext(ptr) };
        let void = NonNull::new(void).expect("failed to create a void type");
        let void = unsafe { Void::new(Type::new(void)) };

        Self { ptr: ctx, f32, f64, bool, unit, void }
    }
    

    pub fn module(&self, name: &str) -> Module<'me> {
        assert!(!name.contains('\0'), "the module name can't contain null bytes");

        let pool = Arena::tls_get_temp();
        let name = sti::format_in!(&*pool, "{name}\0");

        let module = unsafe { LLVMModuleCreateWithNameInContext(name.as_ptr() as *const i8,
                                                                self.ptr.as_ptr()) };

        let module = NonNull::new(module).expect("failed to create a module");

        Module::new(module)
    }

        
    pub fn as_ctx_ref(&self) -> ContextRef<'me> {
        ContextRef(*self)
    }
}


// Types 
impl<'me> ContextImpl<'me> {
    pub fn integer(&self, size: u32) -> IntegerTy<'me> {
        let ptr = unsafe { LLVMIntTypeInContext(self.ptr.as_ptr(), size) };
        let ptr = NonNull::new(ptr).expect("failed to create an integer type");
        let ty = Type::new(ptr);

        unsafe { IntegerTy::new(ty) }
    }


    pub fn f32(&self) -> FPTy<'me> { self.f32 }
    pub fn f64(&self) -> FPTy<'me> { self.f64 }
    pub fn bool(&self) -> BoolTy<'me> { self.bool }
    pub fn unit(&self) -> UnitTy<'me> { self.unit }
    pub fn void(&self) -> Void<'me> { self.void }


    pub fn ptr(&self) -> PtrTy<'me> {
        let ptr = unsafe { LLVMPointerTypeInContext(self.ptr.as_ptr(), 0) };
        unsafe { PtrTy::new(Type::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn structure(&self, name: &str) -> StructTy<'me> {
        assert!(!name.contains('\0'));
        let pool = Arena::tls_get_temp();
        let name = format_in!(&*pool, "{name}\0");

        let ptr = unsafe { LLVMStructCreateNamed(self.ptr.as_ptr(),
                                                 name.as_ptr() as *const i8) };

        unsafe { StructTy::new(Type::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn union(&self, name: &str) -> UnionTy<'me> {
        assert!(!name.contains('\0'));
        let pool = Arena::tls_get_temp();
        let name = format_in!(&*pool, "{name}\0");

        let ptr = unsafe { LLVMStructCreateNamed(self.ptr.as_ptr(),
                                                 name.as_ptr() as *const i8) };

        unsafe { UnionTy::new(Type::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn array(&self, ty: Type<'me>, size: usize) -> ArrayTy<'me> {
        let ptr = unsafe { LLVMArrayType2(ty.llvm_ty().as_ptr(), size as u64) };

        unsafe { ArrayTy::new(Type::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn const_str(&self, str: &str) -> StringValue<'me> {
        let ptr = unsafe { LLVMConstStringInContext(self.ptr.as_ptr(),
                                                    str.as_ptr() as *const i8,
                                                    str.len() as u32,
                                                    1) };

        unsafe { StringValue::new(Value::new(NonNull::new(ptr).unwrap())) }

    }


    pub fn const_unit(&self) -> Unit<'me> {
        let ptr = unsafe { LLVMConstInt(LLVMIntTypeInContext(self.ptr.as_ptr(), 1), 0, 0) };
        let ptr = NonNull::new(ptr).unwrap();
        let ptr = Value::new(ptr);
        unsafe { Unit::new(ptr) }
    }


    pub fn const_array(&self, ty: Type<'me>, vals: &[Value<'me>]) -> Array<'me> {
        let pool = Arena::tls_get_rec();
        let mut vec = sti::vec::Vec::with_cap_in(&*pool, vals.len());
        for v in vals { assert_eq!(ty, v.ty()); vec.push(unsafe { v.llvm_val().as_ptr() }) }

        let ptr = unsafe { LLVMConstArray2(ty.llvm_ty().as_ptr(), vec.as_mut_ptr(), vec.len() as u64) };
        let ptr = NonNull::new(ptr).unwrap();
        let ptr = Value::new(ptr);
        unsafe { Array::new(ptr) }
    }


    pub fn const_int(&self, ty: IntegerTy<'me>, val: i64, sign_extended: bool) -> Integer<'me> {
        if val as u64 > 2u64.saturating_pow(ty.bit_size() as u32) {
            panic!("the constant ({val}) is out of bounds of the integer size ({})", ty.bit_size());
        }

        let ptr = unsafe { LLVMConstInt(ty.llvm_ty().as_ptr(), val as u64, sign_extended as i32) };
        let ptr = NonNull::new(ptr).expect("failed to build a const int");
        let ptr = Value::new(ptr);
        unsafe { Integer::new(ptr) }
    }


    pub fn const_f32(&self, val: f32) -> FP<'me> {
        let ptr = unsafe { LLVMConstReal(self.f32().llvm_ty().as_ptr(), val as f64) };
        let ptr = NonNull::new(ptr).expect("failed to build a const f32");
        let ptr = Value::new(ptr);
        unsafe { FP::new(ptr) }
    }


    pub fn const_f64(&self, val: f64) -> FP<'me> {
        let ptr = unsafe { LLVMConstReal(self.f64().llvm_ty().as_ptr(), val) };
        let ptr = NonNull::new(ptr).expect("failed to build a const f64");
        let ptr = Value::new(ptr);
        unsafe { FP::new(ptr) }
    }


    pub fn const_bool(&self, val: bool) -> Bool<'me> {
        let ptr = unsafe { LLVMConstInt(LLVMIntTypeInContext(self.ptr.as_ptr(), 1), val as u64, 0) };
        let ptr = NonNull::new(ptr).expect("failed to build a const bool");
        let ptr = Value::new(ptr);
        unsafe { Bool::new(ptr) }
    }


    pub fn const_struct(&self, ty: StructTy<'me>, fields: &[Value<'me>]) -> Struct<'me> {
        assert!(!ty.is_opaque(), "can't create a non-opaque type");
        assert_eq!(ty.fields_count(), fields.len());

        let arena = Arena::tls_get_rec();
        for (f, sf) in fields.iter().zip(ty.fields(&*arena)) {
            assert_eq!(f.ty(), sf);
        }


        let ptr = unsafe { LLVMConstNamedStruct(ty.llvm_ty().as_ptr(),
                    fields.as_ptr().cast_mut().cast(), fields.len() as u32) };

        unsafe { Struct::new(Value::new(NonNull::new(ptr).unwrap())) }
    }
}


impl<'ctx> Deref for Context<'ctx> {
    type Target = ContextImpl<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}


impl<'ctx> Deref for ContextRef<'ctx> {
    type Target = ContextImpl<'ctx>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


impl<'ctx> AsRef<ContextRef<'ctx>> for Context<'ctx> {
    fn as_ref(&self) -> &ContextRef<'ctx> {
        &self.0
    }
}


impl<'ctx> AsRef<ContextImpl<'ctx>> for Context<'ctx> {
    fn as_ref(&self) -> &ContextImpl<'ctx> {
        &self.0.0
    }
}


impl<'ctx> AsRef<ContextImpl<'ctx>> for ContextRef<'ctx> {
    fn as_ref(&self) -> &ContextImpl<'ctx> {
        &self.0
    }
}


impl<'me> Drop for Context<'me> {
    fn drop(&mut self) {
        unsafe { LLVMContextDispose(self.0.0.ptr.as_ptr()) }
    }
}
