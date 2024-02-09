use std::{ptr::null, marker::PhantomData};

use proc_macros::margarine;

///
/// A pointer to wasm memory
///
#[margarine]
pub struct WasmPtr(u32);

impl WasmPtr {
    ///
    /// Returns a const reference to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_ptr(self, ctx: &Ctx) -> *const u8 {
        assert!(ctx.size >= self.0);
        unsafe { ctx.base.0.add(self.0 as usize) }
    }


    ///
    /// Returns a mutable pointer to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_mut(self, ctx: &Ctx) -> *mut u8 {
        self.as_ptr(ctx).cast_mut()
    }


    ///
    /// Constructs a new `WasmPtr` out of a `u32`
    ///
    /// This function is safe as other subsequent safe calls
    /// will check for the bounds and panic if out of bounds
    ///
    pub extern "C" fn from_u32(i: u32) -> Self { Self(i) }
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
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    base: SendPtr,
    size: u32,
}

impl Ctx {
    pub const fn new() -> Self { Self { base: SendPtr(null()), size: 0 } }

    pub fn set_base(&mut self, ptr: *const u8) {
        assert!(self.base.0.is_null());
        assert!(!ptr.is_null());
        self.base.0 = ptr;
    }

    pub fn set_size(&mut self, len: u32) {
        self.size = len;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct SendPtr(*const u8);
unsafe impl Send for SendPtr {}
unsafe impl Sync for SendPtr {}


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
