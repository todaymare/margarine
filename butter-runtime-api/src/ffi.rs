use std::{marker::PhantomData, ptr::{null, null_mut}};

use proc_macros::margarine;
use wasmtime::{Instance, Memory, Store};

use crate::alloc::Allocable;

///
/// A pointer to wasm memory
///
#[margarine]
pub struct WasmPtr<T>(u32, PhantomData<T>);

impl<T> WasmPtr<T> {
    ///
    /// Returns a const reference to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_ptr(self, ctx: &Ctx) -> *const T {
        assert!(ctx.size() >= self.0);
        unsafe { ctx.base().add(self.0 as usize).cast() }
    }


    #[inline(always)]
    pub extern "C" fn as_ptr_ex(self, mem: &impl Allocable) -> *const T {
        unsafe { mem.data_ptr().add(self.0 as usize).cast() }
    }


    ///
    /// Returns a mutable pointer to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_mut(self, ctx: &Ctx) -> *mut T {
        self.as_ptr(ctx).cast_mut().cast()
    }


    #[inline(always)]
    pub extern "C" fn as_mut_ex(self, mem: &impl Allocable) -> *mut T {
        unsafe { mem.data_ptr().add(self.0 as usize).cast() }
    }


    ///
    /// Returns a u32 to the wasm memory
    ///
    #[inline(always)]
    pub const extern "C" fn as_u32(self) -> u32 {
        self.0
    }


    ///
    /// Constructs a new `WasmPtr` out of a `u32`
    ///
    /// This function is safe as other subsequent safe calls
    /// will check for the bounds and panic if out of bounds
    ///
    pub const extern "C" fn from_u32(i: u32) -> Self { Self(i, PhantomData) }
}


#[margarine]
pub struct Ptr<T>(i64, PhantomData<T>);


impl<T> Ptr<T> {
    pub extern "C" fn new(data: *mut T) -> Self  { Ptr (data as i64, PhantomData) }


    pub extern "C" fn as_ref<'a>(self) -> &'a T {
        unsafe { &*(self.0 as *const i64).cast() }
    }


    pub extern "C" fn as_mut<'a>(self) -> &'a mut T {
        unsafe { &mut *(self.0 as *mut i64).cast() }
    }
}

unsafe impl<T> Sync for Ptr<T> {}
unsafe impl<T> Send for Ptr<T> {}


#[repr(C)]
#[derive(Debug)]
pub struct Ctx {
    memory: SendPtr<Memory>,
    store: SendMutPtr<Store<()>>,
    instance: SendMutPtr<Instance>,
}

impl Ctx {
    pub const fn new() -> Self { Self { memory: SendPtr(null()), store: SendMutPtr(null_mut()), instance: SendMutPtr(null_mut()) } }

    pub fn set_mem(&mut self, ptr: &Memory) {
        assert!(self.memory.0.is_null());
        self.memory.0 = ptr;
    }

    pub fn set_store(&mut self, store: &mut Store<()>) { self.store.0 = store; }
    pub fn set_instance(&mut self, store: &mut Instance) { self.instance.0 = store; }

    pub fn base(&self) -> *const u8 {
        unsafe { (*self.memory.0).data_ptr(&*self.store.0) }
    }

    pub fn size(&self) -> u32 {
        unsafe { (*self.memory.0).data_size(&*self.store.0) }.try_into().unwrap_or(u32::MAX)
    }

    pub fn mem(&'_ self) -> &'_ Memory {
        unsafe { &*self.memory.0 }
    }

    pub fn store(&'_ self) -> &'_ mut Store<()> {
        unsafe { &mut *self.store.0 }
    }

    pub fn instance(&'_ self) -> &'_ mut Instance {
        unsafe { &mut *self.instance.0 }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct SendPtr<T: Send + Sync>(pub *const T);
unsafe impl<T: Send + Sync> Send for SendPtr<T> {}
unsafe impl<T: Send + Sync> Sync for SendPtr<T> {}

#[derive(Clone, Copy, Debug)]
pub struct SendMutPtr<T: Send + Sync>(pub *mut T);
unsafe impl<T: Send + Sync> Send for SendMutPtr<T> {}
unsafe impl<T: Send + Sync> Sync for SendMutPtr<T> {}


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
