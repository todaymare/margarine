use crate::mem::{NonNull, Cell, size_of};
use crate::alloc::{Alloc, GlobalAlloc, Layout};


/// minimum block size.
/// - the arena will never allocate blocks smaller than this.
pub const BLOCK_SIZE_MIN: usize = 1024;

/// default maximum block size.
/// - by default, the arena will never allocate blocks larger than this.
pub const BLOCK_SIZE_DEFAULT_MAX: usize = 16*1024*1024;

/// maximum block size.
const BLOCK_SIZE_MAX: usize = usize::MAX/4 + 1;
const ALLOC_SIZE_MAX: usize = BLOCK_SIZE_MAX - size_of::<BlockHeader>();

const BLOCK_ALIGN: usize = 16;


pub struct Arena {
    /// - block != NonNull::dangling() iff cap != 0
    block: Cell<NonNull<BlockHeader>>,

    /// - cap <= BLOCK_SIZE_MAX
    /// - cap >= size_of::<BlockHeader>() || cap == 0
    cap: Cell<usize>,

    /// - used <= self.cap
    /// - self.used >= size_of::<BlockHeader>() || cap == 0
    used: Cell<usize>,

    /// hint for the minimum block size.
    pub block_size_min: Cell<usize>,

    /// hint for the maximum block size.
    pub block_size_max: Cell<usize>,
}

unsafe impl Send for Arena {}

struct BlockHeader {
    prev: NonNull<BlockHeader>,
    prev_cap: usize,
}


impl Arena {
    pub const fn new() -> Arena {
        Arena {
            block: Cell::new(NonNull::dangling()),
            cap: Cell::new(0),
            used: Cell::new(0),
            block_size_min: Cell::new(BLOCK_SIZE_MIN),
            block_size_max: Cell::new(BLOCK_SIZE_DEFAULT_MAX),
        }
    }


    #[inline]
    pub fn alloc_ptr<T>(&self) -> NonNull<T> {
        crate::alloc::alloc_ptr::<T>(self).unwrap()
    }

    #[inline]
    pub fn alloc_new<T>(&self, value: T) -> &mut T {
        unsafe { crate::alloc::alloc_new(self, value).unwrap().as_mut() }
    }

    #[inline]
    pub fn alloc_str<'a>(&'a self, value: &str) -> &'a str {
        unsafe {
            let bytes = crate::alloc::alloc_array(self, value.len()).unwrap();
            core::ptr::copy_nonoverlapping(value.as_ptr(), bytes.as_ptr(), value.len());
            core::str::from_utf8_unchecked(
                core::slice::from_raw_parts(bytes.as_ptr(), value.len()))
        }
    }


    /// # safety:
    /// - `layout.size() > 0`.
    #[cold]
    unsafe fn alloc_slow_path(&self, layout: Layout) -> Option<NonNull<u8>> {
        assert!(layout.size() > 0);

        let padded_size = layout.size().checked_add(layout.align() - 1)?;
        if padded_size > ALLOC_SIZE_MAX {
            return None;
        }

        // geometric growth.
        let new_cap = {
            // can't overflow cause `cap <= BLOCK_SIZE_MAX`.
            let new_cap = 2*self.cap.get();
            let new_cap = new_cap.max(self.block_size_min.get());
            let new_cap = new_cap.min(self.block_size_max.get());
            let new_cap = new_cap.max(BLOCK_SIZE_MIN);
            let new_cap = new_cap.min(BLOCK_SIZE_MAX);
            // can't overflow cause size/align we checked to be within limits above.
            let new_cap = new_cap.max(size_of::<BlockHeader>() + padded_size);
            new_cap
        };
        assert!(new_cap <= BLOCK_SIZE_MAX);

        // allocate block.
        crate::static_assert!(align_of::<BlockHeader>() <= BLOCK_ALIGN);
        let block_layout = unsafe { Layout::from_size_align_unchecked(new_cap, BLOCK_ALIGN) };
        let block: NonNull<BlockHeader> = GlobalAlloc.alloc(block_layout)?.cast();
        assert!(block.is_aligned());

        // save current state.
        unsafe {
            block.as_ptr().write(BlockHeader {
                prev: self.block.get(),
                prev_cap: self.cap.get(),
            });
        }

        // requested allocation.
        let (result, new_used) = unsafe {
            let used = size_of::<BlockHeader>();
            let aligned_used = crate::num::ceil_to_multiple_pow2(used, layout.align());
            let new_used = aligned_used + layout.size();
            assert!(new_used <= new_cap);

            // `block` is always `NonNull`, so any valid derived pointer is also `NonNull`.
            let ptr = NonNull::new_unchecked(block.as_ptr().cast::<u8>().add(aligned_used));

            (ptr, new_used)
        };
        assert!(new_used <= new_cap);

        crate::asan::poison(
            unsafe { block.as_ptr().cast::<u8>().add(new_used) },
            new_cap - new_used);

        // attach new block.
        self.block.set(block);
        self.cap.set(new_cap);
        self.used.set(new_used);

        return Some(result);
    }


    pub fn reset(&mut self) {
        self.reset_core(false);
    }

    pub fn reset_all(&mut self) {
        self.reset_core(true);
    }

    pub fn reset_core(&self, including_first: bool) {
        if self.cap.get() == 0 {
            return;
        }

        let mut block = self.block.get();
        let mut cap = self.cap.get();

        crate::asan::unpoison(block.as_ptr().cast(), cap);

        while cap > 0 {
            let header = unsafe { block.as_ptr().read() };

            if header.prev_cap == 0 && !including_first {
                break;
            }

            unsafe {
                let block_layout = Layout::from_size_align_unchecked(cap, BLOCK_ALIGN);
                GlobalAlloc.free(block.cast(), block_layout);
            }

            block = header.prev;
            cap = header.prev_cap;
        }

        let used = if cap > 0 { size_of::<BlockHeader>() } else { 0 };
        assert!(used <= cap);

        crate::asan::poison(
            unsafe { block.as_ptr().cast::<u8>().add(used) },
            cap - used);

        self.block.set(block);
        self.cap.set(cap);
        self.used.set(used);
    }
}



#[derive(Clone, Copy, Debug)]
pub struct ArenaStats {
    pub blocks: isize,
    pub allocated: isize,
    pub used: isize,
}

impl crate::ops::Sub<ArenaStats> for ArenaStats {
    type Output = ArenaStats;

    fn sub(self, rhs: ArenaStats) -> Self::Output {
        ArenaStats {
            blocks:    self.blocks    - rhs.blocks,
            allocated: self.allocated - rhs.allocated,
            used:      self.used      - rhs.used,
        }
    }
}

impl Arena {
    pub fn stats(&self) -> ArenaStats {
        let mut result = ArenaStats {
            blocks: 0,
            allocated: 0,
            used: self.used.get() as isize,
        };

        let mut block = self.block.get();
        let mut cap = self.cap.get();
        while cap != 0 {
            result.allocated += cap as isize;
            result.blocks += 1;

            let header = unsafe { block.as_ptr().read() };

            block = header.prev;
            cap = header.prev_cap;
            result.used += header.prev_cap as isize;
        }

        return result;
    }
}


// safe: Arena is not Clone.
unsafe impl Alloc for Arena {
    unsafe fn alloc_nonzero(&self, layout: Layout) -> Option<NonNull<u8>> {
        debug_assert!(layout.size() > 0);

        let cap = self.cap.get();
        let used = self.used.get();

        // can't overflow cause `used <= BLOCK_SIZE_MAX`.
        // and `align <= 2^63`.
        let aligned_used = crate::num::ceil_to_multiple_pow2(used, layout.align());

        // can't overflow cause `used <= BLOCK_SIZE_MAX`.
        // and `size <= isize::MAX`.
        if aligned_used + layout.size() <= cap {
            let result = unsafe {
                let block = self.block.get();

                // `block.add(cap)` is the end of the allocation,
                // and `ptr + layout.size() <= cap`.
                let ptr = block.as_ptr().cast::<u8>().add(aligned_used);

                // `block` is always `NonNull`, so any valid derived pointer is also `NonNull`.
                NonNull::new_unchecked(ptr)
            };
            debug_assert!(result.as_ptr() as usize & (layout.align() - 1) == 0);

            self.used.set(aligned_used + layout.size());

            crate::asan::unpoison(result.as_ptr(), layout.size());

            return Some(result);
        }
        else {
            return unsafe { self.alloc_slow_path(layout) };
        }
    }

    #[inline]
    unsafe fn free_nonzero(&self, ptr: NonNull<u8>, layout: Layout) {
        crate::asan::poison(ptr.as_ptr(), layout.size());
    }

    unsafe fn try_realloc_nonzero(&self, ptr: NonNull<u8>, old_layout: Layout, new_layout: Layout) -> Result<(), ()> {
        debug_assert!(old_layout.size() > 0);
        debug_assert!(new_layout.size() > 0);
        debug_assert!(old_layout.align() == new_layout.align());

        let old_size = old_layout.size();
        let new_size = new_layout.size();

        let ptr = ptr.as_ptr() as usize;
        let block = self.block.get().as_ptr() as usize;

        let alloc_end = ptr + old_size;
        let used_end = block + self.used.get();
        if alloc_end != used_end {
            return Err(())
        }

        let block_end = block + self.cap.get();
        let block_rem = block_end - ptr;
        if new_size > block_rem {
            return Err(())
        }

        self.used.set(self.used.get() - old_size + new_size);

        debug_assert!(self.used.get() <= self.cap.get());

        return Ok(());
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    fn get_base(arena: &Arena) -> Option<usize> {
        if arena.cap.get() != 0 {
            Some(arena.block.get().as_ptr() as usize)
        }
        else { None }
    }

    #[test]
    fn arena_general() {
        struct Foo {
            name: &'static str,
            age:  u8,
        }

        // init.
        let mut arena = Arena::new();
        assert!(get_base(&arena).is_none());

        // zst does nothing.
        arena.alloc_new(());
        assert!(get_base(&arena).is_none());

        // first allocation allocates block.
        let first = arena.alloc_ptr::<u8>().as_ptr() as usize;
        assert!(get_base(&arena).is_some());
        assert_eq!(first, get_base(&arena).unwrap() + size_of::<BlockHeader>());

        arena.reset();

        // resetting results in equivalent behavior.
        for _ in 0..2 {
            assert_eq!(arena.stats().blocks, 1);

            // first alloc is base.
            let first = arena.alloc_ptr::<u8>().as_ptr() as usize;
            assert_eq!(first, get_base(&arena).unwrap() + size_of::<BlockHeader>());

            // zst does nothing.
            arena.alloc_new([(); 1_000_000]);

            // next alloc is after first alloc.
            let second = arena.alloc_ptr::<u8>().as_ptr() as usize;
            assert_eq!(second, first + 1);

            let third = arena.alloc_ptr::<[u64; 2]>().as_ptr() as usize;
            assert_eq!(third, second + 7);

            // alloc_new sets value.
            let foo = arena.alloc_new(Foo { name: "bar", age: 42 });
            assert!(foo.name == "bar");
            assert!(foo.age  == 42);

            let foo = foo as *mut Foo as usize;
            assert_eq!(foo, third + size_of::<[u64; 2]>());

            let hi = arena.alloc_str("hi");
            assert_eq!(hi, "hi");
            assert_eq!(hi.as_ptr() as usize, foo + size_of::<Foo>());

            let last = arena.alloc_ptr::<u8>().as_ptr() as usize;
            assert_eq!(last, hi.as_ptr() as usize + hi.len());

            assert_eq!(arena.stats().blocks, 1);
            assert_eq!(arena.stats().used as usize, last + 1 - get_base(&arena).unwrap());

            // reset
            arena.reset();
        }

        // arena is reset.
        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().used as usize, size_of::<BlockHeader>());

        let first = arena.alloc_ptr::<u8>().as_ptr() as usize;
        assert_eq!(first, get_base(&arena).unwrap() + size_of::<BlockHeader>());
        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().used as usize, size_of::<BlockHeader>() + 1);
    }

    #[test]
    fn arena_block_size() {
        let mut arena = Arena::new();

        assert_eq!(arena.stats().allocated, 0);
        assert_eq!(arena.stats().blocks, 0);
        assert_eq!(arena.stats().used, 0);

        let block_size = 1024;
        arena.block_size_max.set(block_size);

        arena.alloc_ptr::<u8>();

        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().allocated as usize, block_size);
        assert_eq!(arena.stats().used as usize, size_of::<BlockHeader>() + 1);

        arena.reset();

        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().allocated as usize, block_size);
        assert_eq!(arena.stats().used as usize, size_of::<BlockHeader>());

        crate::alloc::alloc_array::<u8>(&arena, block_size - size_of::<BlockHeader>()).unwrap();

        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().allocated as usize, block_size);
        assert_eq!(arena.stats().used as usize, block_size);

        arena.alloc_ptr::<u8>();

        assert_eq!(arena.stats().blocks, 2);
        assert_eq!(arena.stats().allocated as usize, 2*block_size);
        assert_eq!(arena.stats().used as usize, block_size + size_of::<BlockHeader>() + 1);

        arena.reset();

        assert_eq!(arena.stats().blocks, 1);
        assert_eq!(arena.stats().allocated as usize, block_size);
        assert_eq!(arena.stats().used as usize, size_of::<BlockHeader>());

        arena.reset_all();

        assert_eq!(arena.stats().blocks, 0);
        assert_eq!(arena.stats().allocated, 0);
        assert_eq!(arena.stats().used, 0);
    }

    #[test]
    fn arena_realloc() {
        let arena = Arena::new();

        let layout_1 = Layout::from_size_align(1, 1).unwrap();
        let ptr = arena.alloc(layout_1).unwrap();

        let layout_2 = Layout::from_size_align(9, 1).unwrap();
        let ok = unsafe { arena.try_realloc(ptr, layout_1, layout_2) };
        assert!(ok.is_ok());

        // can't ever resize to `0`.
        let layout_cant = Layout::from_size_align(0, 1).unwrap();
        let err = unsafe { arena.try_realloc(ptr, layout_2, layout_cant) };
        assert!(err.is_err());

        let layout_3 = Layout::from_size_align(5, 1).unwrap();
        let ok = unsafe { arena.try_realloc(ptr, layout_2, layout_3) };
        assert!(ok.is_ok());

        let ok = unsafe { arena.try_realloc(ptr, layout_3, layout_3) };
        assert!(ok.is_ok());

        // new alloc.
        let new_ptr = arena.alloc(Layout::from_size_align(2, 2).unwrap()).unwrap();
        assert!(new_ptr.as_ptr() as usize == ptr.as_ptr() as usize + 6);

        let layout_cant = Layout::from_size_align(5, 1).unwrap();
        let err = unsafe { arena.try_realloc(ptr, layout_3, layout_cant) };
        assert!(err.is_err());

        let layout_cant = Layout::from_size_align(0, 1).unwrap();
        let err = unsafe { arena.try_realloc(ptr, layout_3, layout_cant) };
        assert!(err.is_err());

        let layout_cant = Layout::from_size_align(1, 1).unwrap();
        let err = unsafe { arena.try_realloc(ptr, layout_3, layout_cant) };
        assert!(err.is_err());
    }

    /* broken
    #[test]
    fn arena_realloc_edge_cases() {
        let backing = Arena::new();
        backing.min_block_size.set(1024);

        let arena = Arena::new();
        arena.min_block_size.set(128);
        arena.max_block_size.set(128);

        let layout_1 = Layout::from_size_align(1, 1).unwrap(); // must be 1,1.
        let layout_8 = Layout::from_size_align(8, 1).unwrap();

        // no block.
        assert_eq!(arena.stats().total_allocated, 0);
        let err = unsafe { arena.try_realloc(NonNull::new(&mut 1u8).unwrap(), layout_1, layout_8) };
        assert!(err.is_err());

        let before = backing.alloc_ptr::<[u8; 128]>().cast::<u8>();

        // with block.
        let ptr = arena.alloc_ptr::<u8>();
        let stats = arena.stats();
        assert_eq!(stats.total_allocated, 128);
        assert_eq!(stats.num_blocks, 1);

        let after = backing.alloc_ptr::<[u8; 128]>().cast::<u8>();
        assert_eq!(after.as_ptr() as usize - before.as_ptr() as usize, 128);

        // out of bounds pointers.
        let err = unsafe { arena.try_realloc(before, layout_1, layout_8) };
        assert!(err.is_err());
        let err = unsafe { arena.try_realloc(after, layout_1, layout_8) };
        assert!(err.is_err());
        let err = unsafe { arena.try_realloc(NonNull::new(after.as_ptr().add(128)).unwrap(), layout_1, layout_8) };
        assert!(err.is_err());

        // can't resize beyond end of block.
        let layout_too_full = Layout::from_size_align(128 - size_of::<BlockHeader>() + 1, 1).unwrap();
        let err = unsafe { arena.try_realloc(ptr, layout_1, layout_too_full) };
        assert!(err.is_err());

        // can resize to end of block.
        let layout_full = Layout::from_size_align(128 - size_of::<BlockHeader>(), 1).unwrap();
        let ok = unsafe { arena.try_realloc(ptr, layout_1, layout_full) };
        assert!(ok.is_ok());
    }
    */

    /*
    #[test]
    fn arena_save_restore() {
        let arena = Arena::new();
        arena.min_block_size.set(64);
        arena.max_block_size.set(64);

        let save_0 = arena.save();
        assert_eq!(save_0.used_end, align_of::<BlockHeader>());

        unsafe { arena.restore(save_0.clone()) };
        assert_eq!(arena.stats().total_allocated, 0);
        assert_eq!(arena.stats().num_blocks, 0);
        assert_eq!(arena.current_block_used(), 0);

        let a = arena.alloc_ptr::<[u8; 12]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 12);

        // same block save.
        let save_1 = arena.save();
        assert_eq!(save_1.used_end, a + 12);

        // restore immediately.
        unsafe { arena.restore(save_1.clone()) }
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 12);

        arena.alloc_ptr::<u32>();
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 12 + 4);

        // restore same block.
        unsafe { arena.restore(save_1.clone()) }
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 12);

        // allocate rest of block.
        arena.alloc_ptr::<[u8; 64 - size_of::<BlockHeader>() - 12]>();
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), 64);

        // force another block.
        let b = arena.alloc_ptr::<[u8; 8]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 128);
        assert_eq!(arena.stats().num_blocks, 2);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 8);

        let save_2 = arena.save();
        assert_eq!(save_2.used_end, b + 8);

        // and an oversided block.
        let c = arena.alloc_ptr::<[u8; 128]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 128 + size_of::<BlockHeader>()+128);
        assert_eq!(arena.stats().num_blocks, 3);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>()+128);

        let save_3 = arena.save();
        assert_eq!(save_3.used_end, c + 128);

        // and another normal block.
        let d = arena.alloc_ptr::<[u8; 8]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 128 + size_of::<BlockHeader>()+128 + 64);
        assert_eq!(arena.stats().num_blocks, 4);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 8);

        let save_4 = arena.save();
        assert_eq!(save_4.used_end, d + 8);

        // restore skipping 2 blocks.
        unsafe { arena.restore(save_2) }
        assert_eq!(arena.stats().total_allocated, 128);
        assert_eq!(arena.stats().num_blocks, 2);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 8);

        // restore save_1 again.
        unsafe { arena.restore(save_1) }
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 12);

        // restore save_0 again.
        unsafe { arena.restore(save_0) }
        assert_eq!(arena.stats().total_allocated, 0);
        assert_eq!(arena.stats().num_blocks, 0);
        assert_eq!(arena.current_block_used(), 0);
    }
    */

    /* broken
    #[test]
    fn arena_save_restore_contiguous() {
        let backing = Arena::new();

        #[allow(deprecated)]
        let arena = Arena::new_in(&backing);
        arena.min_block_size.set(64);
        arena.max_block_size.set(64);

        arena.alloc_ptr::<[u8; 64-size_of::<BlockHeader>()]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), 64);

        let save = arena.save();

        arena.alloc_ptr::<[u8; 8]>().as_ptr() as usize;
        assert_eq!(arena.stats().total_allocated, 128);
        assert_eq!(arena.stats().num_blocks, 2);
        assert_eq!(arena.current_block_used(), size_of::<BlockHeader>() + 8);

        // check blocks contiguous.
        assert_eq!(save.used_end, arena.block.get().as_ptr() as usize);

        // correctly frees current block.
        unsafe { arena.restore(save) }
        assert_eq!(arena.stats().total_allocated, 64);
        assert_eq!(arena.stats().num_blocks, 1);
        assert_eq!(arena.current_block_used(), 64);
    }
    */
}

