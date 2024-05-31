use std::{ops::Deref, ptr::NonNull};

use llvm_sys::{core::{LLVMArrayType2, LLVMConstStringInContext, LLVMContextCreate, LLVMContextDispose, LLVMDoubleTypeInContext, LLVMFloatTypeInContext, LLVMIntTypeInContext, LLVMModuleCreateWithNameInContext, LLVMPointerTypeInContext, LLVMStructCreateNamed, LLVMVoidTypeInContext}, LLVMContext};
use sti::{arena::Arena, format_in};

use crate::{module::Module, tys::{array::ArrayTy, bool::BoolTy, fp::FPTy, integer::IntegerTy, ptr::PtrTy, strct::StructTy, union::UnionTy, unit::UnitTy, void::Void, Type}, values::{string::StringValue, Value}};

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
