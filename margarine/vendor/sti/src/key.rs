use crate::num::{NonZeroU32, NonMaxU32};


pub unsafe trait Key: Copy + PartialEq + PartialOrd {
    const MIN: Self;
    const MAX: Self;
    const MAX_USIZE: usize;

    unsafe fn from_usize_unck(value: usize) -> Self;

    fn usize(self) -> usize;

    unsafe fn add(self, delta: usize) -> Self;
    unsafe fn sub(self, delta: usize) -> Self;
}

unsafe impl Key for u8 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn usize(self) -> usize { self as usize }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self { unsafe { self.unchecked_add(delta as Self) } }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self { unsafe { self.unchecked_sub(delta as Self) } }
}

unsafe impl Key for u16 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn usize(self) -> usize { self as usize }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self { unsafe { self.unchecked_add(delta as Self) } }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self { unsafe { self.unchecked_sub(delta as Self) } }
}

unsafe impl Key for u32 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn usize(self) -> usize { self as usize }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self { unsafe { self.unchecked_add(delta as Self) } }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self { unsafe { self.unchecked_sub(delta as Self) } }
}

unsafe impl Key for u64 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn usize(self) -> usize { self as usize }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self { unsafe { self.unchecked_add(delta as Self) } }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self { unsafe { self.unchecked_sub(delta as Self) } }
}

unsafe impl Key for usize {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = Self::MAX as usize;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self { value as Self }

    #[inline(always)]
    fn usize(self) -> usize { self as usize }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self { unsafe { self.unchecked_add(delta as Self) } }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self { unsafe { self.unchecked_sub(delta as Self) } }
}

unsafe impl Key for NonZeroU32 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = u32::MAX as usize - 1;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self {
        unsafe { NonZeroU32::new_unchecked((value as u32).unchecked_add(1)) }
    }

    #[inline(always)]
    fn usize(self) -> usize {
        unsafe { self.get().unchecked_sub(1) as usize }
    }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self {
        unsafe { NonZeroU32::new_unchecked(self.get().unchecked_add(delta as u32)) }
    }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self {
        unsafe { NonZeroU32::new_unchecked(self.get().unchecked_sub(delta as u32)) }
    }
}

unsafe impl Key for NonMaxU32 {
    const MIN: Self = Self::MIN;
    const MAX: Self = Self::MAX;
    const MAX_USIZE: usize = u32::MAX as usize - 1;

    #[inline(always)]
    unsafe fn from_usize_unck(value: usize) -> Self {
        unsafe { NonMaxU32::new_unck(value as u32) }
    }

    #[inline(always)]
    fn usize(self) -> usize {
        self.get() as usize
    }

    #[inline(always)]
    unsafe fn add(self, delta: usize) -> Self {
        unsafe { NonMaxU32::new_unck(self.get().unchecked_add(delta as u32)) }
    }

    #[inline(always)]
    unsafe fn sub(self, delta: usize) -> Self {
        unsafe { NonMaxU32::new_unck(self.get().unchecked_sub(delta as u32)) }
    }
}


#[macro_export]
macro_rules! define_key {
    ($vis:vis $name:ident ( $ty_vis:vis $ty:ty )) => {
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        #[repr(transparent)]
        $vis struct $name($ty_vis $ty);

        unsafe impl $crate::key::Key for $name {
            const MIN: Self = Self(<$ty as $crate::key::Key>::MIN);
            const MAX: Self = Self(<$ty as $crate::key::Key>::MAX);
            const MAX_USIZE: usize = <$ty as $crate::key::Key>::MAX_USIZE;

            #[inline(always)]
            unsafe fn from_usize_unck(value: usize) -> Self {
                unsafe { Self($crate::key::Key::from_usize_unck(value)) }
            }

            #[inline(always)]
            fn usize(self) -> usize {
                $crate::key::Key::usize(self.0)
            }

            #[inline(always)]
            unsafe fn add(self, delta: usize) -> Self {
                unsafe { Self(self.0.add(delta)) }
            }

            #[inline(always)]
            unsafe fn sub(self, delta: usize) -> Self {
                unsafe { Self(self.0.sub(delta)) }
            }
        }

        impl $crate::fmt::Debug for $name {
            fn fmt(&self, f: &mut $crate::fmt::Formatter) -> $crate::fmt::Result {
                write!(f, "{}({})", core::stringify!($name), self.0)
            }
        }
    };
}


pub struct KeyGen<K: Key> {
    pub next: K,
}

impl<K: Key> KeyGen<K> {
    #[inline]
    pub fn new() -> Self {
        Self { next: K::MIN }
    }

    #[inline]
    pub fn from_initial(next: K) -> Self {
        Self { next }
    }


    #[inline]
    pub fn len(&self) -> usize {
        self.next.usize()
    }


    #[inline]
    pub fn next(&mut self) -> K {
        assert!(self.next.usize() < K::MAX_USIZE);
        let result = self.next;
        self.next = unsafe { self.next.add(1) };
        return result;
    }
}


pub struct KRange<K: Key> {
    pub begin: K,
    pub end: K,
}

impl<K: Key> KRange<K> {
    #[inline]
    pub const fn new(begin: K, end: K) -> Self {
        Self { begin, end }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.end.usize().checked_sub(self.begin.usize()).unwrap_or(0)
    }
}

impl<K: Key + core::fmt::Debug> core::fmt::Debug for KRange<K> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{:?}..{:?}", self.begin, self.end)
    }
}


impl<K: Key> Iterator for KRange<K> {
    type Item = K;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.begin < self.end {
            let result = self.begin;
            self.begin = unsafe { self.begin.add(1) };
            return Some(result);
        }
        else { None }
    }

    #[inline]
    fn nth(&mut self, i: usize) -> Option<Self::Item> {
        if i < self.len() { unsafe {
            let result = self.begin.add(i);
            self.begin = result.add(1);
            return Some(result);
        }}
        else { None }
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.begin < self.end {
            Some(unsafe { self.end.sub(1) })
        }
        else { None }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_basic() {
        crate::define_key!(MyKey(NonMaxU32));

        let mut vec: crate::prelude::KVec<MyKey, u8> = crate::vec::KVec::new();

        let min = vec.push(42);
        assert_eq!(min, MyKey::MIN);

        assert_eq!(vec.klen(), MyKey(NonMaxU32::new(1).unwrap()));
    }
}

