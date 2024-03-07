use proc_macros::margarine;
use wasmtime::{Global, Instance, Memory, Store, Val};

use crate::{alloc::Allocable, ptr::{WasmPtr, SendPtr}};

#[repr(C)]
pub struct Ctx {
    memory: SendPtr<Memory>,
    store: SendPtr<Store<()>>,
    instance: SendPtr<Instance>,
    funcs: Vec<&'static str>
}

impl Ctx {
    pub const fn new(funcs: Vec<&str>) -> Self {
        Self {
            memory: SendPtr::null(),
            store: SendPtr::null(),
            instance: SendPtr::null(),
            funcs: unsafe { core::mem::transmute(funcs) }
        }
    }

    pub fn set_mem(&mut self, ptr: &Memory) {
        assert!(self.memory.is_null());
        self.memory = SendPtr::new(ptr);
    }

    pub fn set_store(&mut self, store: &mut Store<()>) { self.store = SendPtr::new(store); }
    pub fn set_instance(&mut self, store: &mut Instance) { self.instance = SendPtr::new(store); }

    fn mem(&'_ self) -> &'_ Memory {
        self.memory.as_ref()
    }

    fn store(&'_ self) -> &'_ mut Store<()> {
        unsafe { self.store.as_mut() }
    }

    fn instance(&'_ self) -> &'_ mut Instance {
        unsafe { self.instance.as_mut() }
    }

}


#[cfg(feature = "wenjin")]
impl Ctx {
    pub fn read_global(&self, str: &str) -> Value {
        let v = self.store().read_global(*self.instance(), str).unwrap();
        match v.value() {
            wenjin::Value::I32(v) => Value::I32(v),
            wenjin::Value::I64(v) => Value::I64(v),
            wenjin::Value::F32(v) => Value::F32(v),
            wenjin::Value::F64(v) => Value::F64(v),
        }
    }


    pub fn copy_mem<T: Copy>(&self, wasm_ptr: WasmPtr<T>) -> T {
        let ptr = self.store().memory_view(*self.mem()).data_ptr();
        unsafe { ptr.byte_add(wasm_ptr.as_u32() as usize).cast::<T>().read() }
    }


    pub fn read_mem<T>(&'_ self, wasm_ptr: WasmPtr<T>) -> &'_ T {
        let ptr = self.store().memory_view(*self.mem()).data_ptr();
        unsafe { &*ptr.byte_add(wasm_ptr.as_u32() as usize).cast::<T>() }
    }

    pub fn funcs(&'_ self) -> &'_ [&'_ str] {
        &self.funcs
    }
}


#[cfg(not(feature = "wenjin"))]
impl Ctx {
    pub fn read_global(&self, str: &str) -> Value {
        let v = self.instance().get_global(self.store(), str).unwrap();

        match v.get(self.store()) {
            wasmtime::Val::I32(v) => Value::I32(v),
            wasmtime::Val::I64(v) => Value::I64(v),
            wasmtime::Val::F32(v) => Value::F32(f32::from_bits(v)),
            wasmtime::Val::F64(v) => Value::F64(f64::from_bits(v)),
            _ => unimplemented!(),
        }
    }


    pub fn copy_mem<T: Copy>(&self, wasm_ptr: WasmPtr<T>) -> T {
        let ptr = self.data_ptr();
        unsafe { ptr.byte_add(wasm_ptr.as_u32() as usize).cast::<T>().read() }
    }


    pub fn read_mem<T>(&'_ self, wasm_ptr: WasmPtr<T>) -> &'_ T {
        let ptr = self.data_ptr();
        unsafe { &*ptr.byte_add(wasm_ptr.as_u32() as usize).cast::<T>() }
    }

    pub fn funcs(&'_ self) -> &'_ [&'_ str] {
        &self.funcs
    }
}


#[cfg(feature = "wenjin")]
impl Allocable for Ctx {
    fn data_ptr(&mut self) -> *mut u8 {
        self.store().memory_view(*self.mem()).data_ptr()
    }

    fn data_size(&mut self) -> usize {
        self.store().memory_view(*self.mem()).size()
    }

    fn size(&mut self) -> usize {
        self.data_size() / (64 * 1024)
    }

    fn grow(&mut self, delta: usize) -> bool {
        self.store().grow_memory(*self.mem(), delta);
        true
    }
}


#[cfg(not(feature = "wenjin"))]
impl Allocable for Ctx {
    fn data_ptr(&self) -> *mut u8 {
        self.mem().data_ptr(self.store())
    }

    fn data_size(&self) -> usize {
        self.mem().data_size(self.store())
    }

    fn size(&self) -> usize {
        self.mem().size(self.store()).try_into().unwrap()
    }

    fn grow(&mut self, delta: usize) -> bool {
        self.mem().grow(self.store(), delta.try_into().unwrap()).is_ok()
    }
}

unsafe impl Send for Ctx {}
unsafe impl Sync for Ctx {}


pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}


impl Value {
    #[inline(always)]
    pub fn u32(self) -> u32 { self.i32() as u32 }
    #[inline(always)]
    pub fn i32(self) -> i32 {
        match self {
            Value::I32(v) => v,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn u64(self) -> u64 { self.i64() as u64 }
    #[inline(always)]
    pub fn i64(self) -> i64 {
        match self {
            Value::I64(v) => v,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn f32(self) -> f32 {
        match self {
            Value::F32(v) => v,
            _ => unreachable!(),
        }
    }

    #[inline(always)]
    pub fn f64(self) -> f64 {
        match self {
            Value::F64(v) => v,
            _ => unreachable!(),
        }
    }
    
}


#[margarine]
pub struct Str {
    len: u64,
    ptr: *const u8,
}


impl Str {
    pub fn new(str: &'static str) -> Self {
        Self {
            len: str.len() as u64,
            ptr: str.as_ptr(),
        }
    }
    pub fn read<'a>(self) -> &'a str {
        let slice = unsafe { std::slice::from_raw_parts(self.ptr, self.len.try_into().unwrap()) };
        let str = std::str::from_utf8(slice).expect("invalid pointer");
        str
    }
}
