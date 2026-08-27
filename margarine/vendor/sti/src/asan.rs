#![allow(unexpected_cfgs)]

#[cfg(asan)]
mod target {
    use crate::ffi::c_void;

    extern "C" {
        fn __asan_poison_memory_region(addr: *const c_void, size: usize);
        fn __asan_unpoison_memory_region(addr: *const c_void, size: usize);
    }

    #[inline]
    pub fn poison(addr: *const u8, size: usize) {
        unsafe { __asan_poison_memory_region(addr as *const c_void, size) }
    }

    #[inline]
    pub fn unpoison(addr: *const u8, size: usize) {
        unsafe { __asan_unpoison_memory_region(addr as *const c_void, size) }
    }


    #[inline]
    pub fn poison_ptr<T>(ptr: *const T) {
        poison(ptr.cast(), crate::mem::size_of::<T>())
    }

    #[inline]
    pub fn unpoison_ptr<T>(ptr: *const T) {
        unpoison(ptr.cast(), crate::mem::size_of::<T>())
    }


    #[inline]
    pub fn poison_ptr_len<T>(ptr: *const T, len: usize) {
        poison(ptr.cast(), crate::mem::size_of::<T>()*len)
    }

    #[inline]
    pub fn unpoison_ptr_len<T>(ptr: *const T, len: usize) {
        unpoison(ptr.cast(), crate::mem::size_of::<T>()*len)
    }


    #[inline]
    pub fn poison_ref<T: ?Sized>(r: &T) {
        poison(r as *const _ as *const u8, crate::mem::size_of_val(r))
    }

    #[inline]
    pub fn unpoison_ref<T: ?Sized>(r: &T) {
        unpoison(r as *const _ as *const u8, crate::mem::size_of_val(r))
    }
}

#[cfg(not(asan))]
#[allow(unused_variables)]
mod target {
    #[inline(always)]
    pub fn poison(addr: *const u8, size: usize) {}

    #[inline(always)]
    pub fn unpoison(addr: *const u8, size: usize) {}


    #[inline(always)]
    pub fn poison_ptr<T>(ptr: *const T) {}

    #[inline(always)]
    pub fn unpoison_ptr<T>(ptr: *const T) {}


    #[inline(always)]
    pub fn poison_ptr_len<T>(ptr: *const T, len: usize) {}

    #[inline(always)]
    pub fn unpoison_ptr_len<T>(ptr: *const T, len: usize) {}


    #[inline(always)]
    pub fn poison_ref<T: ?Sized>(r: &T) {}

    #[inline(always)]
    pub fn unpoison_ref<T: ?Sized>(r: &T) {}
}

pub use target::*;

