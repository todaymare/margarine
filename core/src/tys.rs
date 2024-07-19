use std::{alloc::Layout, marker::PhantomData, mem::{align_of, size_of}};

#[repr(C)]
#[derive(Debug)]
pub struct Rc<T> {
    ptr: *mut RcData<T>,
}


#[repr(C)]
struct RcData<T> {
    count: u64,
    val: T,
}


impl<T> Rc<T> {
    pub fn new(val: T) -> Self {
        let data : RcData<T> = RcData { count: 1, val };

        let layout = Layout::from_size_align(size_of::<RcData<T>>() + size_of::<T>(), align_of::<RcData<T>>()).unwrap();

        let alloc = unsafe { std::alloc::alloc(layout) };

        unsafe { alloc.cast::<RcData<T>>().write(data) };
        
        Rc { ptr: alloc.cast() }
    }
}

impl<T> Rc<T> {
    pub fn read_ptr<'a>(&'a self) -> *const T {
        unsafe { &(*self.ptr).val }
    }
}


impl<T: Copy> Rc<T> {
    pub fn inc(&mut self) {
        debug_assert!(!self.ptr.is_null());

        unsafe { (*self.ptr).count += 1 };
    }


    pub fn dec(&mut self) {
        unsafe { (*self.ptr).count -= 1 };
        if unsafe { (*self.ptr).count } == 0 {
            self.free();
        }
    }


    pub fn count(&self) -> u64 {
        unsafe { self.ptr.read().count }
    }


    pub fn read<'a>(&'a self) -> T {
        unsafe { self.ptr.read().val }
    }


    pub fn free(&mut self) {
        let layout = Layout::for_value(unsafe { &*self.ptr });
        unsafe { core::ptr::drop_in_place(self.ptr) };
        unsafe { std::alloc::dealloc(self.ptr.cast(), layout) };
    }
}


#[repr(C)]
pub struct Str {
    ptr: Rc<u8>,
    len: u32,
}

impl Str {
    pub fn read<'a>(&'a self) -> &'a str {
        let slice = unsafe { core::slice::from_raw_parts(self.ptr.read_ptr().cast(), self.len as usize) };
        core::str::from_utf8(slice).unwrap()
    }
}
