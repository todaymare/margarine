use std::mem::size_of;

use crate::ffi::WasmPtr;
use wasmtime::{Memory, Store};

#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct MemoryBlock {
    ptr: *mut (),
}

type Word = usize;

static mut ALLOC : Walloc = Walloc {
    free_lists: Vec::new(),
};


struct Walloc {
    free_lists: Vec<WasmPtr<Block>>,
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
struct Block {
    next: WasmPtr<Block>,
    used_n_size: usize,

    // user data
    data: usize,
}


impl Block {
    fn size(&self) -> usize { 
        self.used_n_size & !1
    }

    fn is_used(&self) -> bool {
        (self.used_n_size & 1) != 0
    }

    fn set_used(ptr: *mut Self, b: bool) {
        if b {
            unsafe { (*ptr).used_n_size |= 1; }
        } else {
            unsafe { (*ptr).used_n_size &= !1; }
        }
    }
}


fn ptr_to_wptr<T>(memory: &Memory, store: &mut Store<()>, wasm_ptr: *const T) -> WasmPtr<T> {
    let ptr = wasm_ptr as usize;
    let dptr = memory.data_ptr(store) as usize;
    WasmPtr::from_u32((ptr - dptr) as u32)
}



///
/// Returns a `MemoryBlock` which contains the pointer
/// to a memory block at least `size` bytes.
/// 
/// The `size` field of the `MemoryBlock` indicates the
/// actual size of the memory allocated
///
pub fn walloc(memory: &Memory, store: &mut Store<()>, size: usize) -> WasmPtr<()> {
    let size = align_to(size, size_of::<Word>());
    
    // Search for a block
    if let Some(block) = find_block(memory, store, size) {
        let block = try_split(memory, store, block, size);
        Block::set_used(block.as_mut_ex(memory, store), true);

        let data = unsafe { &mut (*block.as_mut_ex(memory, store)).data } as *mut usize as *mut u8;
        return ptr_to_wptr(memory, store, data.cast())
    }

    // If not found, allocate
    let block = request_memory(memory, store, size).expect("Out of memory");

    unsafe {
        (*block.as_mut_ex(memory, store)).used_n_size = size;
        Block::set_used(block.as_mut_ex(memory, store), true);
    }


    let block = block.as_mut_ex(memory, store);
    let data = unsafe { &mut (*block).data } as *mut usize as *mut u8;
    ptr_to_wptr(memory, store, data.cast())
}


///
/// Frees a previously allocated block
///
pub fn free(memory: &Memory, store: &mut Store<()>, ptr: WasmPtr<()>) {
    let ptrb = get_header(ptr);

    let mut curr = ptrb;
    while curr.as_u32() != 0 {
        let currp = curr.as_ptr_ex(memory, store);
        unsafe {
            if !(*currp).is_used() {
                let size = alloc_size((*currp).size());
                (*ptrb.as_mut_ex(memory, store)).used_n_size += size;
                curr = WasmPtr::from_u32(curr.as_u32() + size as u32);
                continue
            }
        }
        break
    }

    Block::set_used(ptrb.as_mut_ex(memory, store), false);
    unsafe { ALLOC.free_lists[get_bucket((*ptrb.as_ptr_ex(memory, store)).size())] = ptrb };
}


fn find_block(memory: &Memory, store: &mut Store<()>, size: usize) -> Option<WasmPtr<Block>> {
    let mut bucket = get_bucket(size);
    let len = unsafe { ALLOC.free_lists.len() };
    let mut left = len;
    while left != 0 {
        let mut curr = unsafe { ALLOC.free_lists[bucket] };

        while curr.as_u32() != 0 {
            let currp = curr.as_ptr_ex(memory, store);

            unsafe {
                if (*currp).is_used() || (*currp).size() < size {
                    curr = (*currp).next;
                    continue;
                }

                ALLOC.free_lists[get_bucket(size)] = (*currp).next;
                return Some(curr);
            }
        }

        bucket = (bucket + 1) % len;
        left -= 1;

    }
    None
}


fn get_bucket(size: usize) -> usize {
    let bucket = size / size_of::<Word>() - 1;
    if unsafe { ALLOC.free_lists.len() } <= bucket {
        unsafe { ALLOC.free_lists.resize(bucket + 1, WasmPtr::from_u32(0)) }
    }

    bucket
}


fn get_header(ptr: WasmPtr<()>) -> WasmPtr<Block> {
    WasmPtr::from_u32(ptr.as_u32() + size_of::<Word>() as u32 - size_of::<Block>() as u32)
}


fn align_to(n: usize, alignment: usize) -> usize {
  return (n + alignment - 1) & !(alignment - 1);
}


fn alloc_size(size: usize) -> usize {
    size + size_of::<Block>() - size_of::<Word>()
}


fn try_split(memory: &Memory, store: &mut Store<()>, ptr: WasmPtr<Block>, size: usize) -> WasmPtr<Block> {
    unsafe {
        let ptrp = ptr.as_mut_ex(memory, store);
        debug_assert!(!(*ptrp).is_used());

        if (*ptrp).size() > size + size_of::<Block>() {
            let nptr = ptrp.cast::<u8>().add(alloc_size(size));
            let nptr = nptr.cast::<Block>();
            let nsize = (*ptrp).size() - alloc_size(size);


            {
                let bucket = get_bucket(nsize);
                let ptr = ALLOC.free_lists[bucket];
                nptr.write(Block {
                    next: ptr, data: 0,
                    used_n_size: size,
                });

                ALLOC.free_lists[bucket] = ptr_to_wptr(memory, store, nptr);
            }

            (*ptrp).used_n_size = size;
        }
    }

    ptr
}

static mut PAGE : WasmPtr<u8> = WasmPtr::from_u32(0);

pub fn set_heap_start(ptr: WasmPtr<u8>) {
    unsafe { PAGE = ptr }
}

fn request_memory(memory: &Memory, store: &mut Store<()>, size: usize) -> Option<WasmPtr<Block>> {
    static mut PAGE_BOUNDARY : WasmPtr<u8> = WasmPtr::from_u32(0);
    
    unsafe {
    if PAGE.as_u32() + size as u32 >= PAGE_BOUNDARY.as_u32() {
        match memory.grow(&mut *store, 1) {
            Ok(s) => {
                PAGE_BOUNDARY = WasmPtr::from_u32((s * 64 * 1024) as u32);
                return request_memory(memory, store, size)
            }
            Err(_) => return None,
        }
    }

    let ptr = WasmPtr::from_u32(PAGE.as_u32());
    PAGE = WasmPtr::from_u32(PAGE.as_u32() + size as u32);
    Some(ptr)
    }
}
