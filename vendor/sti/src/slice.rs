use crate::mem::{NonNull, PhantomData};
use crate::key::{Key, KRange};


pub use core::slice::*;


pub type S<V> = Slice<V>;
pub type KS<K, V> = KSlice<K, V>;

pub type Slice<V> = KSlice<u32, V>;

pub struct KSlice<K: Key, V> {
    phantom: PhantomData<K>,
    values: [V],
}

impl<K: Key, V> KSlice<K, V> {
    #[inline]
    pub fn new(slice: &[V]) -> Option<&Self> {
        if slice.len() <= K::MAX_USIZE {
            Some(unsafe { &*(slice as *const [V] as *const Self) })
        }
        else { None }
    }

    #[inline]
    pub unsafe fn new_unck(slice: &[V]) -> &Self {
        debug_assert!(slice.len() <= K::MAX_USIZE);
        unsafe { &*(slice as *const [V] as *const Self) }
    }

    #[inline]
    pub fn new_mut(slice: &mut [V]) -> Option<&mut Self> {
        if slice.len() <= K::MAX_USIZE {
            Some(unsafe { &mut *(slice as *mut [V] as *mut Self) })
        }
        else { None }
    }

    #[inline]
    pub unsafe fn new_mut_unck(slice: &mut [V]) -> &mut Self {
        debug_assert!(slice.len() <= K::MAX_USIZE);
        unsafe { &mut *(slice as *mut [V] as *mut Self) }
    }


    #[inline(always)]
    pub fn klen(&self) -> K {
        unsafe { K::from_usize_unck(self.values.len()) }
    }

    #[inline]
    pub fn krange(&self) -> KRange<K> {
        KRange { begin: K::MIN, end: self.klen() }
    }

    #[inline(always)]
    pub fn as_slice(&self) -> &[V] {
        &self.values
    }

    #[inline(always)]
    pub fn as_mut_slice(&mut self) -> &mut [V] {
        &mut self.values
    }


    #[inline]
    pub fn kget(&self, idx: K) -> Option<&V> {
        return self.get(idx.usize());
    }

    #[inline]
    pub fn kget_mut(&mut self, idx: K) -> Option<&mut V> {
        return self.get_mut(idx.usize());
    }


    #[inline]
    pub fn kiter(&self) -> KIter<K, V> {
        KIter {
            idx: K::MIN,
            len: self.klen(),
            ptr: unsafe { NonNull::new_unchecked(self.as_ptr() as *mut V) },
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn kiter_mut(&mut self) -> KIterMut<K, V> {
        KIterMut {
            idx: K::MIN,
            len: self.klen(),
            ptr: unsafe { NonNull::new_unchecked(self.as_mut_ptr()) },
            phantom: PhantomData,
        }
    }
}

impl<K: Key, V> crate::ops::Deref for KSlice<K, V> {
    type Target = [V];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<K: Key, V> crate::ops::DerefMut for KSlice<K, V> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}


impl<K: Key + crate::fmt::Debug, V: crate::fmt::Debug> crate::fmt::Debug for KSlice<K, V> {
    fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {
        f.debug_map().entries(self.kiter()).finish()
    }
}


pub trait KSliceIndex<K: Key, V> {
    type Output: ?Sized;

    fn index(self, slice: &KSlice<K, V>) -> &Self::Output;
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output;
}

impl<K: Key, V> KSliceIndex<K, V> for K {
    type Output = V;

    #[inline(always)]
    fn index(self, slice: &KSlice<K, V>) -> &Self::Output {
        &slice.values[self.usize()]
    }

    #[inline(always)]
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output {
        &mut slice.values[self.usize()]
    }
}

impl<K: Key, V> KSliceIndex<K, V> for crate::ops::Range<K> {
    type Output = [V];

    #[inline(always)]
    fn index(self, slice: &KSlice<K, V>) -> &Self::Output {
        &slice.values[self.start.usize()..self.end.usize()]
    }

    #[inline(always)]
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output {
        &mut slice.values[self.start.usize()..self.end.usize()]
    }
}

impl<K: Key, V> KSliceIndex<K, V> for crate::ops::RangeTo<K> {
    type Output = [V];

    #[inline(always)]
    fn index(self, slice: &KSlice<K, V>) -> &Self::Output {
        &slice.values[..self.end.usize()]
    }

    #[inline(always)]
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output {
        &mut slice.values[..self.end.usize()]
    }
}

impl<K: Key, V> KSliceIndex<K, V> for crate::ops::RangeFrom<K> {
    type Output = [V];

    #[inline(always)]
    fn index(self, slice: &KSlice<K, V>) -> &Self::Output {
        &slice.values[self.start.usize()..]
    }

    #[inline(always)]
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output {
        &mut slice.values[self.start.usize()..]
    }
}

impl<K: Key, V> KSliceIndex<K, V> for crate::ops::RangeFull {
    type Output = [V];

    #[inline(always)]
    fn index(self, slice: &KSlice<K, V>) -> &Self::Output {
        &slice.values[..]
    }

    #[inline(always)]
    fn index_mut(self, slice: &mut KSlice<K, V>) -> &mut Self::Output {
        &mut slice.values[..]
    }
}

// @todo: other indices. including `RevIndex(I)`.


impl<K: Key, V, X: KSliceIndex<K, V>> crate::ops::Index<X> for KSlice<K, V> {
    type Output = X::Output;

    #[inline(always)]
    fn index(&self, index: X) -> &Self::Output {
        index.index(self)
    }
}

impl<K: Key, V, X: KSliceIndex<K, V>> crate::ops::IndexMut<X> for KSlice<K, V> {
    #[inline(always)]
    fn index_mut(&mut self, index: X) -> &mut Self::Output {
        index.index_mut(self)
    }
}

impl<'a, K: Key, V> IntoIterator for &'a KSlice<K, V> {
    type IntoIter = KIter<'a, K, V>;
    type Item = (K, &'a V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return self.kiter();
    }
}

impl<'a, K: Key, V> IntoIterator for &'a mut KSlice<K, V> {
    type IntoIter = KIterMut<'a, K, V>;
    type Item = (K, &'a mut V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return self.kiter_mut();
    }
}


#[derive(Clone)]
pub struct KIter<'a, K: Key, V> {
    idx: K, // <= len
    len: K,
    ptr: NonNull<V>,
    phantom: PhantomData<&'a [V]>,
}

impl<'a, K: Key, V> Iterator for KIter<'a, K, V> {
    type Item = (K, &'a V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let idx = self.idx;
            let val = unsafe { &*self.ptr.as_ptr().add(idx.usize()) };
            self.idx = unsafe { idx.add(1) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn nth(&mut self, i: usize) -> Option<Self::Item> {
        if i < self.len.usize() - self.idx.usize() {
            self.idx = unsafe { self.idx.add(i) };

            let idx = self.idx;
            let val = unsafe { &*self.ptr.as_ptr().add(idx.usize()) };
            self.idx = unsafe { idx.add(1) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.idx < self.len {
            let idx = unsafe { self.len.sub(1) };
            let val = unsafe { &*self.ptr.as_ptr().add(idx.usize()) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.usize() - self.idx.usize();
        (len, Some(len))
    }
}


pub struct KIterMut<'a, K: Key, V> {
    len: K,
    ptr: NonNull<V>,
    idx: K, // <= len
    phantom: PhantomData<(K, &'a mut V)>,
}

impl<'a, K: Key, V> Iterator for KIterMut<'a, K, V> {
    type Item = (K, &'a mut V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len {
            let idx = self.idx;
            let val = unsafe { &mut *self.ptr.as_ptr().add(idx.usize()) };
            self.idx = unsafe { idx.add(1) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn nth(&mut self, i: usize) -> Option<Self::Item> {
        if i < self.len.usize() - self.idx.usize() {
            self.idx = unsafe { self.idx.add(i) };

            let idx = self.idx;
            let val = unsafe { &mut *self.ptr.as_ptr().add(idx.usize()) };
            self.idx = unsafe { idx.add(1) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn last(self) -> Option<Self::Item> {
        if self.idx < self.len {
            let idx = unsafe { self.len.sub(1) };
            let val = unsafe { &mut *self.ptr.as_ptr().add(idx.usize()) };
            return Some((idx, val));
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len.usize() - self.idx.usize();
        (len, Some(len))
    }
}

