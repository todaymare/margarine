use std::marker::PhantomData;

use proc_macros::margarine;

use crate::{alloc::Allocable, ffi::Ctx};

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
    pub extern "C" fn as_ptr(self, mem: &impl Allocable) -> *const T {
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
    pub extern "C" fn as_mut(self, ctx: &impl Allocable) -> *mut T {
        self.as_ptr(ctx).cast_mut().cast()
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
    #[inline(always)]
    pub const extern "C" fn from_u32(i: u32) -> Self { Self(i, PhantomData) }
}


#[derive(Debug)]
#[repr(C)]
pub struct SendPtr<T>(*mut T);

impl<T> SendPtr<T> {
    #[inline(always)]
    pub const fn new(ptr: &T) -> Self {
        Self::from_raw(ptr)
    }


    #[inline(always)]
    pub const fn null() -> Self {
        Self::from_raw(core::ptr::null())
    }


    #[inline(always)]
    pub const fn from_raw(ptr: *const T) -> Self {
        Self::from_raw_mut(ptr.cast_mut())
    }


    #[inline(always)]
    pub const fn from_raw_mut(ptr: *mut T) -> Self {
        Self(ptr)
    }

    
    #[inline(always)]
    pub fn is_null(self) -> bool { self.0.is_null() }


    #[inline(always)]
    pub fn as_ref<'a>(self) -> &'a T {
        debug_assert!(!self.0.is_null());
        unsafe { &*self.0 }
    }


    #[inline(always)]
    pub unsafe fn as_mut<'a>(self) -> &'a mut T {
        debug_assert!(!self.0.is_null());
        unsafe { &mut *self.0 }
    }
}

unsafe impl<T> Send for SendPtr<T> {}
unsafe impl<T> Sync for SendPtr<T> {}

impl<T: Send + Sync> PartialEq for SendPtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}


impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        Self::from_raw_mut(self.0)
    }
}

impl<T> Copy for SendPtr<T> {}

