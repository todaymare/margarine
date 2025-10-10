use std::ops::Deref;

use sti::alloc::Alloc;

///
/// A dynamically allocated non-resizable buffer
///
pub struct Buffer<T, A: Alloc> {
    internal: sti::vec::Vec<T, A>,
}


impl<T, A: Alloc> Buffer<T, A> {
    pub fn new(alloc: A, cap: usize) -> Self {
        Self {
            internal: sti::vec::Vec::with_cap_in(alloc, cap),
        }
    }


    /// Push an item into the buffer
    /// Panics if the value can't fit in the buffer
    pub fn push(&mut self, val: T) {
        if self.push_checked(val).is_some() { panic!("push out of bounds") };
    }


    /// Push an item into the buffer
    /// Returns `Some(val)` if the value can't fit in the buffer
    pub fn push_checked(&mut self, val: T) -> Option<T> {
        if self.internal.len() == self.internal.cap() { return Some(val) }
        self.internal.push(val);
        None
    }


    pub fn len(&self) -> usize {
        self.internal.len()
    }


    /// Leaks the buffer
    /// The returned slice isn't guaranteed to be
    /// as big as the initial buffer
    pub fn leak<'a>(self) -> &'a mut [T] where A: 'a {
        self.internal.leak_slice()
    }

    pub fn extend_from_slice<'a>(&mut self, vals: &'a [T]) -> Option<&'a [T]> where T: Copy {
        for (i, val) in vals.iter().enumerate() {
            if self.push_checked(*val).is_some() {
                return Some(&vals[i..])
            }
        }

        None
    }
}


impl<T, A: Alloc> Deref for Buffer<T, A> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &*self.internal
    }
}
