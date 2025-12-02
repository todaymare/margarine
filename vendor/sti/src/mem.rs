pub use core::{
    cell::{Cell, UnsafeCell},
    marker::{PhantomData, Unpin},
    mem::{
        MaybeUninit, ManuallyDrop,
        transmute, transmute_copy,
        take, replace, swap, forget,
        size_of, size_of_val, align_of, offset_of,
    },
    ptr::{
        NonNull,
        write_bytes, copy, copy_nonoverlapping,
        drop_in_place,
    },
    pin::{Pin, pin},
};


pub fn as_bytes_maybe_uninit<T: ?Sized>(value: &T) -> &[MaybeUninit<u8>] { unsafe {
    return crate::slice::from_raw_parts(
        value as *const T as *const MaybeUninit<u8>,
        size_of_val(value));
}}

pub unsafe fn as_bytes<T: ?Sized>(value: &T) -> &[u8] { unsafe {
    return crate::slice::from_raw_parts(
        value as *const T as *const u8,
        size_of_val(value));
}}

pub unsafe fn ref_from_bytes<T>(bytes: &[u8]) -> &T { unsafe {
    debug_assert!(bytes.as_ptr().cast::<T>().is_aligned());
    debug_assert!(bytes.len() == size_of::<T>());
    return &*bytes.as_ptr().cast::<T>();
}}

pub unsafe fn slice_from_bytes<T>(bytes: &[u8]) -> &[T] { unsafe {
    debug_assert!(bytes.as_ptr().cast::<T>().is_aligned());
    debug_assert!(bytes.len() % size_of::<T>() == 0);
    return core::slice::from_raw_parts(
        bytes.as_ptr().cast::<T>(),
        bytes.len() / size_of::<T>());
}}

