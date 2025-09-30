use std::alloc::Layout;

use crate::Reg;

pub struct Alloc {
    ptrs: Vec<AllocatedObject>,
}


pub struct AllocatedObject {
    size: usize, // size in bytes
    is_leaked: bool,
    is_struct: bool,
    ptr : *mut u8,
}


impl Alloc {
    pub fn new() -> Self {
        Self {
            ptrs: vec![]
        }
    }


    pub fn alloc_struct(&mut self, size: usize) -> &mut [Reg] {
        todo!()
    }
}

