use crate::mem::{NonNull, size_of, align_of};
use crate::num::ceil_to_multiple_pow2;


pub use core::alloc::Layout;


/// # safety:
/// - if `A: Clone`, `realloc` and `free` must be valid on all clones.
/// - if `A: Default`, `A` must be a "global allocator":
///     - all operations must be valid on all instances.
///     - `A` must be `Send + Sync`.
pub unsafe trait Alloc {
    /// allocates a block of memory.
    ///
    /// - if the call succeeds:
    ///     - the returned pointer refers to a live allocation.
    ///     - `layout` is the active layout of the returned memory block.
    ///
    #[inline]
    fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        if layout.size() != 0 {
            // layout.size() > 0.
            let result = unsafe { self.alloc_nonzero(layout) };

            if let Some(result) = result {
                debug_assert!(result.as_ptr() as usize % layout.align() == 0);
            }

            return result;
        }
        else {
            return Some(dangling(layout));
        }
    }

    /// allocates a block of memory.
    ///
    /// - if the call succeeds:
    ///     - the returned pointer refers to a live allocation.
    ///     - `layout` is the active layout of the returned memory block.
    ///
    /// # safety:
    /// - `layout.size() > 0`
    ///
    unsafe fn alloc_nonzero(&self, layout: Layout) -> Option<NonNull<u8>>;


    /// frees an allocation.
    ///
    /// - after the call, `ptr` is no longer a live allocation.
    ///
    /// # safety:
    /// - if `layout.size() > 0`:
    ///     - `ptr` must be a live allocation, allocated from this allocator.
    ///     - `layout` must be the active layout of the memory block.
    ///
    #[inline]
    unsafe fn free(&self, ptr: NonNull<u8>, layout: Layout) {
        if layout.size() != 0 {
            // invariants upheld by the caller, because `layout.size() > 0`.
            unsafe { self.free_nonzero(ptr, layout) }
        }
    }


    /// frees an allocation.
    ///
    /// - after the call, `ptr` is no longer a live allocation.
    ///
    /// # safety:
    /// - `ptr` must be a live allocation, allocated from this allocator.
    /// - `layout` must be the active layout of the memory block.
    /// - `layout.size() > 0`.
    ///
    unsafe fn free_nonzero(&self, ptr: NonNull<u8>, layout: Layout);


    /// attempts to resize an allocation.
    ///
    /// - returns whether resizing succeeded.
    /// - the allocation referenced by `ptr` always remains live.
    /// - if the call succeeds, `new_layout` is the active layout,
    ///   otherwise, `old_layout` remains the active layout.
    ///
    /// # safety:
    /// - if `old_layout.size() > 0`:
    ///     - `ptr` must be a live allocation, allocated from this allocator.
    ///     - `old_layout` must be the active layout of the memory block.
    /// - `old_layout.align() == new_layout.align()`.
    ///
    #[inline]
    unsafe fn try_realloc(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<(), ()> {
        debug_assert!(old_layout.align() == new_layout.align());

        if old_layout.size() == 0 || new_layout.size() == 0 {
            return Err(())
        }
        unsafe { self.try_realloc_nonzero(ptr, old_layout, new_layout) }
    }

    /// attempts to resize an allocation.
    ///
    /// - returns whether resizing succeeded.
    /// - the allocation referenced by `ptr` always remains live.
    /// - if the call succeeds, `new_layout` is the active layout,
    ///   otherwise, `old_layout` remains the active layout.
    ///
    /// # safety:
    /// - `ptr` must be a live allocation, allocated from this allocator.
    /// - `old_layout` must be the active layout of the memory block.
    /// - `old_layout.align() == new_layout.align()`.
    /// - `old_layout.size() > 0`.
    /// - `new_layout.size() > 0`.
    ///
    #[inline]
    unsafe fn try_realloc_nonzero(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<(), ()> {
        debug_assert!(old_layout.align() == new_layout.align());

        let _ = (ptr, old_layout, new_layout);
        Err(())
    }

    /// resize an allocation.
    ///
    /// - if the call succeeds:
    ///     - the returned pointer must be used for subsequent `Alloc` method calls.
    ///     - the active layout is `new_layout`.
    ///     - the new memory block's bytes in `0..min(old_layout.size(), new_layout.size())`
    ///       have the old memory block's values, other bytes are undefined.
    /// - if the call fails:
    ///     - `ptr` remains live.
    ///     - `old_layout` remains the active layout.
    ///     - the memory block referenced by `ptr` remains unchanged.
    ///
    /// # safety:
    /// - if old_layout.size() > 0:
    ///     - `ptr` must be a live allocation, allocated from this allocator.
    ///     - `old_layout` must be the active layout of the memory block.
    /// - `old_layout.align() == new_layout.align()`.
    ///
    unsafe fn realloc(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Option<NonNull<u8>> {
        unsafe { default_realloc(self, ptr, old_layout, new_layout) }
    }
}

#[inline]
pub unsafe fn default_realloc<A: Alloc + ?Sized>(alloc: &A, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Option<NonNull<u8>> {
    debug_assert_eq!(old_layout.align(), new_layout.align());

    // invariants upheld by the caller.
    if unsafe { alloc.try_realloc(ptr, old_layout, new_layout).is_ok() } {
        return Some(ptr);
    }
    else {
        let new_ptr = alloc.alloc(new_layout);

        if let Some(new_ptr) = new_ptr {
            unsafe {
                let min_size = old_layout.size().min(new_layout.size());
                if min_size > 0 {
                    // both allocations are live and at least `min_size` large.
                    // live allocations can't overlap.
                    crate::mem::copy_nonoverlapping(ptr.as_ptr(), new_ptr.as_ptr(), min_size);
                }

                // invariants upheld by the caller.
                alloc.free(ptr, old_layout);
            }
        }
        // else: failed. keep old allocation live.

        return new_ptr;
    }
}


// safe: a clone of `&A` refers to the same `A`.
unsafe impl<A: Alloc + ?Sized> Alloc for &A {
    #[inline(always)]
    fn alloc(&self, layout: Layout) -> Option<NonNull<u8>> {
        A::alloc(self, layout)
    }

    #[inline(always)]
    unsafe fn alloc_nonzero(&self, layout: Layout) -> Option<NonNull<u8>> {
        unsafe { A::alloc_nonzero(self, layout) }
    }


    #[inline(always)]
    unsafe fn free(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { A::free(self, ptr, layout) }
    }

    #[inline(always)]
    unsafe fn free_nonzero(&self, ptr: NonNull<u8>, layout: Layout) {
        unsafe { A::free_nonzero(self, ptr, layout) }
    }


    #[inline(always)]
    unsafe fn try_realloc(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<(), ()> {
        unsafe { A::try_realloc(self, ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn try_realloc_nonzero(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<(), ()> {
        unsafe { A::try_realloc_nonzero(self, ptr, old_layout, new_layout) }
    }

    #[inline(always)]
    unsafe fn realloc(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Option<NonNull<u8>> {
        unsafe { A::realloc(self, ptr, old_layout, new_layout) }
    }
}



#[inline]
pub fn dangling(layout: Layout) -> NonNull<u8> {
    // this is kinda sketchy.
    // taken from the unstable impl of `Layout::dangling`
    unsafe { NonNull::new_unchecked(crate::mem::transmute(layout.align())) }
}

#[inline]
pub fn alloc_ptr<T>(alloc: &impl Alloc) -> Option<NonNull<T>> {
    if let Some(ptr) = alloc.alloc(Layout::new::<T>()) {
        return Some(ptr.cast());
    }

    crate::hint::cold();
    return None;
}

#[inline]
pub fn alloc_ptr_with_extra<T>(alloc: &impl Alloc, extra: usize) -> Option<NonNull<T>> {
    if let Some(size) = crate::mem::size_of::<T>().checked_add(extra) {
    if let Ok(layout) = Layout::from_size_align(size, crate::mem::align_of::<T>()) {
    if let Some(ptr) = alloc.alloc(layout) {
        return Some(ptr.cast());
    }}}

    crate::hint::cold();
    return None;
}

#[inline]
pub fn alloc_array<T>(alloc: &impl Alloc, len: usize) -> Option<NonNull<T>> {
    if let Ok(layout) = Layout::array::<T>(len) {
    if let Some(ptr) = alloc.alloc(layout) {
        return Some(ptr.cast());
    }}

    crate::hint::cold();
    return None;
}

#[inline]
pub unsafe fn realloc_array<T>(alloc: &impl Alloc, ptr: NonNull<T>, old_len: usize, new_len: usize) -> Option<NonNull<T>> { unsafe {
    let old_layout = Layout::array::<T>(old_len).unwrap_unchecked();

    if let Ok(new_layout) = Layout::array::<T>(new_len) {
    if let Some(ptr) = alloc.realloc(ptr.cast(), old_layout, new_layout) {
        return Some(ptr.cast());
    }}

    crate::hint::cold();
    return None;
}}

#[inline]
pub unsafe fn free_array<T>(alloc: &impl Alloc, ptr: NonNull<T>, len: usize) { unsafe {
    let layout = Layout::array::<T>(len).unwrap_unchecked();
    alloc.free(ptr.cast(), layout);
}}

#[inline]
pub fn alloc_new<T>(alloc: &impl Alloc, value: T) -> Option<NonNull<T>> {
    if let Some(ptr) = alloc.alloc(Layout::new::<T>()) {
        let ptr = ptr.cast::<T>();
        unsafe { ptr.as_ptr().write(value) }
        return Some(ptr);
    }

    crate::hint::cold();
    return None;
}

/// #safety:
/// - safety requirements of `Alloc::free`.
#[inline]
pub unsafe fn free<T: ?Sized, A: Alloc>(alloc: &A, ptr: NonNull<T>) {
    unsafe {
        let layout = Layout::for_value(ptr.as_ref());
        alloc.free(ptr.cast(), layout);
    }
}

/// #safety:
/// - safety requirements of `Alloc::free`.
/// - `ptr.as_ref()` must be properly initialized.
#[inline]
pub unsafe fn drop_and_free<T: ?Sized, A: Alloc>(alloc: &A, ptr: NonNull<T>) {
    unsafe {
        let layout = Layout::for_value(ptr.as_ref());
        crate::mem::drop_in_place(ptr.as_ptr());
        alloc.free(ptr.cast(), layout);
    }
}



#[inline]
pub fn cat_join(a: Layout, b: Layout) -> Option<Layout> {
    let b_begin = ceil_to_multiple_pow2(a.size(), b.align());

    let new_size = b_begin.checked_add(b.size())?;
    let new_align = a.align().max(b.align());
    Layout::from_size_align(new_size, new_align).ok()
}

#[inline]
pub unsafe fn cat_next<T, U>(base: *const T, len: usize) -> *const U {
    unsafe { cat_next_bytes(base, len*size_of::<T>(), align_of::<U>()) }
}

#[inline]
pub unsafe fn cat_next_bytes<T, U>(base: *const T, base_size: usize, next_align: usize) -> *const U {
    let result = ceil_to_multiple_pow2(base as usize + base_size, next_align);
    return base.with_addr(result).cast();
}

#[inline]
pub unsafe fn cat_next_mut<T, U>(base: *mut T, len: usize) -> *mut U {
    unsafe { cat_next_mut_bytes(base, len*size_of::<T>(), align_of::<U>()) }
}

#[inline]
pub unsafe fn cat_next_mut_bytes<T, U>(base: *mut T, base_size: usize, next_align: usize) -> *mut U {
    let result = ceil_to_multiple_pow2(base as usize + base_size, next_align);
    return base.with_addr(result).cast();
}



#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Hash)]
pub struct GlobalAlloc;

unsafe impl Sync for GlobalAlloc {}
unsafe impl Send for GlobalAlloc {}

#[cfg(feature="std")]
// safe: GlobalAlloc is zst.
unsafe impl Alloc for GlobalAlloc {
    unsafe fn alloc_nonzero(&self, layout: Layout) -> Option<NonNull<u8>> {
        debug_assert!(layout.size() > 0);
        unsafe {
            return NonNull::new(std::alloc::alloc(layout));
        }
    }

    unsafe fn free_nonzero(&self, ptr: NonNull<u8>, layout: Layout) {
        debug_assert!(layout.size() > 0);
        unsafe {
            std::alloc::dealloc(ptr.as_ptr(), layout);
        }
    }

    unsafe fn realloc(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Option<NonNull<u8>> {
        debug_assert_eq!(old_layout.align(), new_layout.align());
        if old_layout.size() != 0 {
            if new_layout.size() != 0 {
                // allocation invariants upheld by the caller, because `old_layout.size() > 0`
                // new_size is non-zero. `Layout` guarantees rounding requirements.
                let result = unsafe { std::alloc::realloc(ptr.as_ptr(), old_layout, new_layout.size()) };
                NonNull::new(result)
            }
            else {
                // invariants upheld by the caller, because `old_layout.size() > 0`.
                unsafe { std::alloc::dealloc(ptr.as_ptr(), old_layout) };
                return Some(dangling(new_layout));
            }
        }
        else {
            return self.alloc(new_layout);
        }
    }
}

