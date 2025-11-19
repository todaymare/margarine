use crate::ext::FromIn;
use crate::alloc::{Alloc, GlobalAlloc, Layout};
use crate::mem::{NonNull, PhantomData, MaybeUninit, ManuallyDrop, size_of, align_of};
use crate::key::Key;
use crate::slice::{KSlice, KIter, KIterMut};


pub type Vec<V, A = GlobalAlloc> = KVec<u32, V, A>;

pub type ZVec<V, A = GlobalAlloc> = KVec<usize, V, A>;

#[derive(PartialEq, Eq, Hash)]
pub struct KVec<K: Key, V, A: Alloc = GlobalAlloc> {
    alloc: A,
    cap: K,
    len: K,
    ptr: NonNull<V>,
    phantom: PhantomData<V>,
}

impl<K: Key, V> KVec<K, V, GlobalAlloc> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self::new_in(GlobalAlloc)
    }

    #[inline(always)]
    pub fn with_cap(cap: usize) -> Self {
        Self::with_cap_in(GlobalAlloc, cap)
    }

    #[inline(always)]
    pub fn from_value(n: usize, value: V) -> Self
    where V: Clone{
        Self::from_value_in(GlobalAlloc, n, value)
    }

    #[inline(always)]
    pub fn from_array<const N: usize>(values: [V; N]) -> Self {
        Self::from_array_in(GlobalAlloc, values)
    }

    #[inline(always)]
    pub fn from_slice(values: &[V]) -> Self
    where V: Clone {
        Self::from_slice_in(GlobalAlloc, values)
    }
}

impl<K: Key, V, A: Alloc> KVec<K, V, A> {
    #[inline]
    pub const fn new_in(alloc: A) -> Self {
        Self {
            alloc,
            cap: K::MIN,
            len: K::MIN,
            ptr: NonNull::dangling(),
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn with_cap_in(alloc: A, cap: usize) -> Self {
        let mut this = Self::new_in(alloc);
        unsafe { this.set_cap(cap) };
        return this;
    }


    pub fn from_value_in(alloc: A, n: usize, value: V) -> Self
    where V: Clone {
        let mut this = Self::with_cap_in(alloc, n);

        unsafe {
            crate::asan::unpoison_ref(this.uninit_slice_mut());

            for i in 0..n {
                this.ptr.as_ptr().add(i).write(value.clone());
            }

            this.len = K::from_usize_unck(n);
        }

        return this;
    }

    #[inline]
    pub fn from_array_in<const N: usize>(alloc: A, values: [V; N]) -> Self {
        let mut this = Self::with_cap_in(alloc, values.len());

        unsafe {
            crate::asan::unpoison_ref(this.uninit_slice_mut());

            let values = ManuallyDrop::new(values);
            crate::mem::copy_nonoverlapping(
                values.as_ptr(),
                this.as_mut_ptr(),
                values.len());

            this.len = K::from_usize_unck(values.len());
        }

        return this;
    }

    #[inline]
    pub fn from_slice_in(alloc: A, values: &[V]) -> Self
    where V: Clone {
        let mut this = Self::with_cap_in(alloc, values.len());

        unsafe {
            crate::asan::unpoison_ref(this.uninit_slice_mut());

            for i in 0..values.len() {
                this.ptr.as_ptr().add(i).write(values[i].clone());
            }

            this.len = K::from_usize_unck(values.len());
        }

        return this;
    }


    #[inline(always)]
    pub fn alloc(&self) -> &A { &self.alloc }

    #[inline(always)]
    pub fn cap(&self) -> usize { self.cap.usize() }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len.usize() }

    #[inline(always)]
    pub fn klen(&self) -> K { self.len }

    #[inline(always)]
    pub fn as_ptr(&self) -> *const V { self.ptr.as_ptr() }

    #[inline(always)]
    pub fn as_mut_ptr(&mut self) -> *mut V { self.ptr.as_ptr() }

    #[inline]
    pub fn as_slice(&self) -> &[V] { unsafe {
        crate::slice::from_raw_parts(self.ptr.as_ptr(), self.len.usize())
    }}

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] { unsafe {
        crate::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len.usize())
    }}

    #[inline(always)]
    pub fn as_kslice(&self) -> &KSlice<K, V> {
        unsafe { KSlice::new_unck(self.as_slice()) }
    }

    #[inline(always)]
    pub fn as_mut_kslice(&mut self) -> &mut KSlice<K, V> {
        unsafe { KSlice::new_mut_unck(self.as_mut_slice()) }
    }

    /// - note: these values are asan poisoned.
    ///   you need to unpoison them before writing.
    ///   and when you're done with the unsafe op,
    ///   make sure the values returned by this function
    ///   are poisoned again.
    ///   the simplest way would be:
    ///   ```code
    ///     asan::unpoison_ref(v.uninit_slice_mut());
    ///     // do something.
    ///     asan::poison_ref(v.uninit_slice_mut());
    ///   ```
    #[inline]
    pub fn uninit_slice_mut(&mut self) -> &mut [MaybeUninit<V>] { unsafe {
        crate::slice::from_raw_parts_mut(
            self.ptr.as_ptr().add(self.len.usize()).cast(), 
            self.cap.usize() - self.len.usize())
    }}

    #[inline]
    pub fn check_idx(&self, idx: usize) -> Option<K> {
        if idx < self.len() {
            Some(unsafe { K::from_usize_unck(idx) })
        }
        else { None }
    }


    pub const CAP_MAX: usize = {
        let v_size = size_of::<V>();
        let size_max = isize::MAX as usize - (align_of::<V>() - 1);
        let cap_max = size_max / if v_size > 0 { v_size } else { 1 };
        let cap_max = if cap_max <= K::MAX_USIZE { cap_max } else { K::MAX_USIZE };
        cap_max
    };

    pub const GROW_MIN_CAP: usize = {
        if size_of::<V>() == 0 {
            Self::CAP_MAX
        }
        else if size_of::<V>() <= 256 {
            let cap = 16 / size_of::<V>();
            if cap < 4 { 4 } else { cap }
        }
        else { 1 }
    };


    /// set the vector's capacity.
    ///
    /// # safety:
    /// - `self.len <= new_cap`.
    ///
    unsafe fn set_cap(&mut self, new_cap: usize) {
        assert!(self.len.usize() <= new_cap);

        if new_cap == self.cap.usize() {
            return;
        }

        // ensure we don't overflow K or the max allocation size.
        assert!(new_cap <= Self::CAP_MAX);

        crate::asan::unpoison_ref(self.uninit_slice_mut());

        let new_ptr = unsafe {
            let old_layout = Layout::array::<V>(self.cap.usize()).unwrap_unchecked();
            let new_layout = Layout::array::<V>(new_cap.usize()).unwrap_unchecked();
            self.alloc.realloc(self.ptr.cast(), old_layout, new_layout).unwrap().cast()
        };

        self.ptr = new_ptr;
        self.cap = unsafe { K::from_usize_unck(new_cap) };

        crate::asan::poison_ref(self.uninit_slice_mut());
    }

    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap());
        self.len = unsafe { K::from_usize_unck(new_len) };
    }

    pub fn trim_exact(&mut self) {
        // `new_cap >= self.len`.
        unsafe { self.set_cap(self.len.usize()) }
    }

    pub fn reserve(&mut self, min_cap: usize) {
        let new_cap = min_cap;
        if new_cap > self.cap.usize() {
            let new_cap =
                if size_of::<V>() > 0 {
                    // can't overflow, cause `self.cap <= isize::MAX/sizeof(T)`.
                    new_cap.max(2*self.cap.usize())
                }
                else { new_cap };

            let new_cap = new_cap.max(Self::GROW_MIN_CAP);

            // `new_cap > self.cap >= self.len`.
            unsafe { self.set_cap(new_cap) };
        }
    }

    pub fn reserve_exact(&mut self, cap: usize) {
        if cap > self.cap.usize() {
            // `min_cap > self.cap >= self.len`.
            unsafe { self.set_cap(cap) };
        }
    }

    pub fn reserve_more(&mut self, extra: usize) {
        self.reserve(self.len.usize().checked_add(extra).unwrap());
    }
    
    #[cold]
    fn reserve_one_more(&mut self) {
        self.reserve_more(1);
    }



    pub fn resize(&mut self, new_len: usize, default: V)
    where V: Clone {
        if new_len <= self.len.usize() {
            self.truncate(new_len);
        }
        else {
            self.reserve(new_len);
            debug_assert!(new_len <= self.cap.usize());

            unsafe {
                let ptr = self.ptr.as_ptr().add(self.len.usize());
                let len = new_len - self.len.usize();
                crate::asan::unpoison_ptr_len(ptr, len);

                for i in 0..len {
                    ptr.add(i).write(default.clone());
                }

                self.len = K::from_usize_unck(new_len);
            }
        }
    }

    pub fn truncate(&mut self, new_len: usize) {
        assert!(new_len <= self.len.usize());
        if new_len == self.len.usize() {
            return;
        }

        unsafe {
            let ptr = self.ptr.as_ptr().add(new_len);
            let len = self.len.usize() - new_len;

            crate::mem::drop_in_place(
                crate::slice::from_raw_parts_mut(ptr, len));

            crate::asan::poison_ptr_len(ptr, len);

            self.len = K::from_usize_unck(new_len);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }


    #[inline]
    pub fn push(&mut self, value: V) -> K {
        if self.len == self.cap {
            self.reserve_one_more();
        }

        let idx = self.len;
        unsafe {
            let ptr = self.ptr.as_ptr().add(idx.usize());
            crate::asan::unpoison_ptr(ptr);

            ptr.write(value);

            self.len = idx.add(1);
        }
        return idx;
    }

    pub fn extend_from_slice(&mut self, values: &[V])
    where V: Clone {
        self.reserve_more(values.len());

        unsafe {
            let ptr = self.ptr.as_ptr().add(self.len.usize());
            crate::asan::unpoison_ptr_len(ptr, values.len());

            for i in 0..values.len() {
                ptr.add(i).write(values[i].clone());
            }

            self.len = self.len.add(values.len());
        }
    }

    pub fn insert(&mut self, idx: K, value: V) {
        assert!(idx <= self.len);

        if self.len == self.cap {
            self.reserve_one_more();
        }

        unsafe {
            let ptr = self.ptr.as_ptr().add(idx.usize());

            let to_move = self.len.usize() - idx.usize();
            crate::asan::unpoison_ptr(ptr.add(to_move));

            if to_move > 0 {
                crate::mem::copy(ptr, ptr.add(1), to_move);
            }

            ptr.write(value);
            self.len = self.len.add(1);
        }
    }

    pub fn insert_from_slice(&mut self, idx: K, values: &[V]) {
        assert!(idx <= self.len);

        self.reserve_more(values.len());

        unsafe {
            let ptr = self.ptr.as_ptr().add(idx.usize());

            let to_move = self.len.usize() - idx.usize();
            crate::asan::unpoison_ptr_len(ptr.add(to_move), values.len());

            if to_move > 0 {
                crate::mem::copy(ptr, ptr.add(values.len()), to_move);
            }

            crate::mem::copy_nonoverlapping(values.as_ptr(), ptr, values.len());

            self.len = self.len.add(values.len());
        }
    }


    #[inline]
    pub fn pop(&mut self) -> Option<V> {
        if self.len.usize() > 0 { unsafe {
            let last = self.len.sub(1);

            let ptr = self.ptr.as_ptr().add(last.usize());
            let result = ptr.read();
            crate::asan::poison_ptr(ptr);

            self.len = last;
            return Some(result);
        }}
        else { None }
    }

    pub fn remove_swap(&mut self, idx: K) -> V { unsafe {
        assert!(idx < self.len);

        let ptr = self.ptr.as_ptr().add(idx.usize());
        let result = ptr.read();

        let last = self.ptr.as_ptr().add(self.len.usize() - 1);
        if ptr < last {
            ptr.write(last.read());
        }
        crate::asan::poison_ptr(last);

        self.len = self.len.sub(1);

        return result;
    }}


    #[inline]
    pub fn take(&mut self) -> Self where A: Clone {
        crate::mem::replace(self, Self::new_in(self.alloc.clone()))
    }

    #[inline]
    pub fn leak<'a>(self) -> &'a KSlice<K, V>
    where A: 'a { unsafe {
        KSlice::new_mut_unck(self.leak_slice())
    }}

    #[inline]
    pub fn leak_slice<'a>(self) -> &'a mut [V]
    where A: 'a { unsafe {
        let this = core::mem::ManuallyDrop::new(self);
        crate::slice::from_raw_parts_mut(this.ptr.as_ptr(), this.len())
    }}


    pub fn clone_in<B: Alloc>(&self, alloc: B) -> KVec<K, V, B>
    where V: Clone {
        let mut this = KVec::with_cap_in(alloc, self.len.usize());
        this.extend_from_slice(&self);
        return this;
    }
}

unsafe impl<K: Key, V: Sync, A: Alloc + Sync> Sync for KVec<K, V, A> {}
unsafe impl<K: Key, V: Send, A: Alloc + Send> Send for KVec<K, V, A> {}


impl<K: Key, V, A: Alloc> Drop for KVec<K, V, A> {
    fn drop(&mut self) { unsafe {
        // drop values.
        crate::mem::drop_in_place(
            crate::slice::from_raw_parts_mut(
                self.ptr.as_ptr(), self.len.usize()));

        // `self.cap` is always valid for `Layout::array`.
        let layout = Layout::array::<V>(self.cap.usize()).unwrap_unchecked();

        // asan::poison is done by alloc.free.

        // `self.data` is an allocation iff `self.cap > 0`.
        self.alloc.free(self.ptr.cast(), layout);
    }}
}


impl<K: Key, V, A: Alloc> crate::ops::Deref for KVec<K, V, A> {
    type Target = KSlice<K, V>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_kslice()
    }
}

impl<K: Key, V, A: Alloc> crate::ops::DerefMut for KVec<K, V, A> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_kslice()
    }
}


impl<K: Key, V: Clone, A: Alloc + Clone> Clone for KVec<K, V, A> {
    fn clone(&self) -> Self {
        self.clone_in(self.alloc.clone())
    }
}


impl<K: Key + crate::fmt::Debug, V: crate::fmt::Debug, A: Alloc> crate::fmt::Debug for KVec<K, V, A> {
    fn fmt(&self, f: &mut crate::fmt::Formatter<'_>) -> crate::fmt::Result {
        self.as_kslice().fmt(f)
    }
}


impl<K: Key, V, A: Alloc + Default> Default for KVec<K, V, A> {
    #[inline]
    fn default() -> Self {
        Self::new_in(A::default())
    }
}


impl<K: Key, V, A: Alloc, I: Iterator<Item = V>> FromIn<I, A> for KVec<K, V, A> {
    #[inline]
    fn from_in(alloc: A, iter: I) -> Self {
        let (min_len, max_len) = iter.size_hint();
        let cap = max_len.unwrap_or(min_len);

        let mut result = KVec::with_cap_in(alloc, cap);
        for v in iter {
            result.push(v);
        }
        return result;
    }
}

impl<K: Key, V> FromIterator<V> for KVec<K, V, GlobalAlloc> {
    fn from_iter<T: IntoIterator<Item = V>>(iter: T) -> Self {
        Self::from_in(GlobalAlloc, iter.into_iter())
    }
}

impl<K: Key, V, A: Alloc> Extend<V> for KVec<K, V, A> {
    #[inline]
    fn extend<I: IntoIterator<Item = V>>(&mut self, iter: I) {
        let iter = iter.into_iter();

        let (min_len, max_len) = iter.size_hint();
        let len = max_len.unwrap_or(min_len);

        self.reserve_more(len);
        for v in iter {
            self.push(v);
        }
    }
}

impl<K: Key, V, A: Alloc> IntoIterator for KVec<K, V, A> {
    type IntoIter = IntoIter<K, V, A>;
    type Item = (K, V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        let vec = ManuallyDrop::new(self);
        return IntoIter {
            cap: vec.cap,
            ptr: vec.ptr,
            len: vec.len,
            idx: K::MIN,
            alloc: unsafe { crate::ptr::read(&vec.alloc) },
        };
    }
}

impl<'a, K: Key, V, A: Alloc> IntoIterator for &'a KVec<K, V, A> {
    type IntoIter = KIter<'a, K, V>;
    type Item = (K, &'a V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return self.kiter();
    }
}

impl<'a, K: Key, V, A: Alloc> IntoIterator for &'a mut KVec<K, V, A> {
    type IntoIter = KIterMut<'a, K, V>;
    type Item = (K, &'a mut V);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        return self.kiter_mut();
    }
}


pub struct IntoIter<K: Key, V, A: Alloc> {
    cap: K,
    ptr: NonNull<V>,
    len: K,
    idx: K, // <= cap
    alloc: A,
}

impl<K: Key, V, A: Alloc> IntoIter<K, V, A> {
    #[inline]
    fn len(&self) -> usize {
        self.len.usize() - self.idx.usize()
    }
}

impl<K: Key, V, A: Alloc> Drop for IntoIter<K, V, A> {
    fn drop(&mut self) { unsafe {
        // drop values.
        crate::mem::drop_in_place(
            crate::slice::from_raw_parts_mut(
                self.ptr.as_ptr().add(self.idx.usize()), self.len()));

        // `self.cap` is always valid for `Layout::array`.
        let layout = Layout::array::<V>(self.cap.usize()).unwrap_unchecked();

        // asan::poison is done by alloc.free.

        // `self.data` is an allocation iff `self.cap > 0`.
        self.alloc.free(self.ptr.cast(), layout);
    }}
}

impl<K: Key, V, A: Alloc> Iterator for IntoIter<K, V, A> {
    type Item = (K, V);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx < self.len { unsafe {
            let idx = self.idx;

            let ptr = self.ptr.as_ptr().add(idx.usize());
            let val = ptr.read();
            crate::asan::poison_ptr(ptr);

            self.idx = idx.add(1);

            return Some((idx, val));
        }}
        else { None }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}



#[macro_export]
macro_rules! vec_in {
    ($alloc:expr) => {
        $crate::vec::Vec::new_in($alloc)
    };

    ($alloc:expr, $elem:expr; $n:expr) => {
        $crate::vec::Vec::from_value_in($alloc, $n, $elem)
    };

    ($alloc:expr; $($x:expr),+ $(,)?) => {
        $crate::vec::Vec::from_array_in($alloc, [$($x),+])
    };
}

#[macro_export]
macro_rules! vec {
    () => {
        $crate::vec::Vec::new()
    };

    ($elem:expr; $n:expr) => {
        $crate::vec::Vec::from_value($n, $elem)
    };

    ($($x:expr),+ $(,)?) => {
        $crate::vec::Vec::from_array([$($x),+])
    };
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_basic() {
        let v: Vec<bool> = vec!();
        assert_eq!(**v, []);
        assert_eq!(v.cap(), 0);

        let v = vec![1, 2, 3];
        assert_eq!(**v, [1, 2, 3]);
        assert_eq!(v.cap(), 3);

        /*
        let v = vec!["hi".to_string(); 2];
        assert_eq!(**v, ["hi".to_string(), "hi".to_string()]);
        assert_eq!(v.cap(), 2);

        let v = Vec::from_slice(&["hi".to_string(), "ho".to_string()]);
        assert_eq!(**v, ["hi".to_string(), "ho".to_string()]);
        assert_eq!(v.cap(), 2);
        */


        let mut v = vec![1, 2, 3, 4];
        assert_eq!(v.remove_swap(1), 2);
        assert_eq!(**v, [1, 4, 3]);
        assert_eq!(v.remove_swap(2), 3);
        assert_eq!(**v, [1, 4]);


        let mut arena = crate::arena::Arena::new();
        {
            let v: Vec<bool, _> = vec_in!(&arena);
            assert_eq!(**v, []);

            let v = vec_in!(&arena; 1, 2, 3);
            assert_eq!(**v, [1, 2, 3]);

            /*
            let v = vec_in!(&arena, "hi".to_string(); 2);
            assert_eq!(**v, ["hi".to_string(), "hi".to_string()]);

            let v = Vec::from_slice_in(&arena, &["hi".to_string(), "ho".to_string()]);
            assert_eq!(**v, ["hi".to_string(), "ho".to_string()]);
            assert_eq!(v.cap(), 2);
            */
        }
        arena.reset();
    }

    #[test]
    fn vec_from_iter() {
        use crate::ext::CopyIt;

        let a = Vec::from_iter(6..9);
        assert_eq!(a.len(), 3);
        assert_eq!(a.cap(), 3);
        assert_eq!(**a, [6, 7, 8]);

        let b = Vec::from_iter(a.copy_map_it(|x| x - 5));
        assert_eq!(b.len(), 3);
        assert_eq!(b.cap(), 3);
        assert_eq!(**b, [1, 2, 3]);


        let mut arena = crate::arena::Arena::new();
        {
            let aa = Vec::from_in(&arena, a.copy_it());
            assert_eq!(aa.len(), 3);
            assert_eq!(aa.cap(), 3);
            assert_eq!(**aa, [6, 7, 8]);

            let bb = Vec::from_in(&arena, aa.copy_it().rev());
            assert_eq!(bb.len(), 3);
            assert_eq!(bb.cap(), 3);
            assert_eq!(**bb, [8, 7, 6]);
        }
        arena.reset();
    }

    #[test]
    fn vec_extend() {
        let mut v = vec![1, 2, 3];
        assert_eq!(**v, [1, 2, 3]);

        v.extend([4, 5]);
        assert_eq!(**v, [1, 2, 3, 4, 5]);
        assert_eq!(v.cap(), 2*3);

        let mut v = vec!["hi"; 6];
        v.extend_from_slice(&["ho", "ha"]);
        assert_eq!(v.cap(), 2*6);
        let mut vs = v.kiter();
        for i in 0..6 {
            assert_eq!(vs.next().unwrap(), (i, &"hi"));
        }
        assert_eq!(vs.next().unwrap(), (6, &"ho"));
        assert_eq!(vs.next().unwrap(), (7, &"ha"));
        assert_eq!(vs.next(), None);
    }

    #[test]
    fn vec_resize() {
        let mut v = vec![1, 2, 3];

        v.resize(2, 69);
        assert_eq!(**v, [1, 2]);

        v.resize(7, 69);
        assert_eq!(**v, [1, 2, 69, 69, 69, 69, 69]);
    }

    #[test]
    fn vec_clone() {
        let mut v = vec![1];
        assert_eq!(v.cap(), 1);

        let v2 = v.clone();
        assert_eq!(v2.cap(), v2.len());
        assert_eq!(**v2, [1]);
        drop(v2);

        v.push(2);
        assert_eq!(v.cap(), 4); // grow min cap.

        let v3 = v.clone();
        assert_eq!(v3.cap(), v3.len());
        assert_eq!(**v3, [1, 2]);
        drop(v3);

        v.push(3);
        assert_eq!(v.cap(), 4);

        let v4 = v.clone();
        assert_eq!(v4.cap(), v4.len());
        assert_eq!(**v4, [1, 2, 3]);
        drop(v4);
    }

    #[test]
    fn vec_drop() {
        use crate::mem::Cell;

        struct Dropper<'a> {
            ticket: u32,
            counter: &'a Cell<u32>
        }

        impl<'a> Drop for Dropper<'a> {
            fn drop(&mut self) {
                assert_eq!(self.counter.get(), self.ticket);
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter = Cell::new(0);
        let d = |ticket: u32| {
            Dropper { ticket, counter: &counter }
        };

        // basic drop.
        let mut v = Vec::new();
        v.push(d(0));
        v.push(d(1));
        v.push(d(2));
        drop(v);
        assert_eq!(counter.get(), 3);

        // truncate.
        counter.set(0);
        let mut v = Vec::new();
        v.push(d(2));
        v.push(d(0));
        v.push(d(1));
        v.truncate(1);
        drop(v);
        assert_eq!(counter.get(), 3);
    }

    #[test]
    fn vec_insert() {
        let mut v = Vec::new();
        v.push(69);
        v.push(420);
        v.push(31298);
        v.insert(0, 0);
        assert_eq!(**v, [0, 69, 420, 31298]);
        v.insert(1, 1);
        assert_eq!(**v, [0, 1, 69, 420, 31298]);
        v.push(4389);
        v.insert(2, 2);
        assert_eq!(**v, [0, 1, 2, 69, 420, 31298, 4389]);
        v.insert(3, 3);
        assert_eq!(**v, [0, 1, 2, 3, 69, 420, 31298, 4389]);
        v.push(574);
        v.push(12398);
        v.insert(v.klen(), 4);
        assert_eq!(**v, [0, 1, 2, 3, 69, 420, 31298, 4389, 574, 12398, 4]);
    }


    #[test]
    fn vec_insert_from_slice() {
        let mut v = Vec::new();
        v.push(69);
        v.push(31298);
        v.push(4389);
        v.insert_from_slice(0, &[0, 1, 2, 3]);
        assert_eq!(**v, [0, 1, 2, 3, 69, 31298, 4389]);
        v.push(574);
        v.push(12398);
        v.insert_from_slice(v.klen(), &[6, 7, 8]);
        assert_eq!(**v, [0, 1, 2, 3, 69, 31298, 4389, 574, 12398, 6, 7, 8]);
        v.push(9);
        v.push(10);
        v.insert_from_slice(4, &[4, 5]);
        assert_eq!(**v, [0, 1, 2, 3, 4, 5, 69, 31298, 4389, 574, 12398, 6, 7, 8, 9, 10]);
    }

    #[test]
    fn vec_into_iter() {
        use crate::mem::Cell;

        struct Dropper<'a> {
            ticket: u32,
            counter: &'a Cell<u32>
        }

        impl<'a> Drop for Dropper<'a> {
            fn drop(&mut self) {
                assert_eq!(self.counter.get(), self.ticket);
                self.counter.set(self.counter.get() + 1);
            }
        }

        let counter = Cell::new(0);
        let d = |ticket: u32| {
            Dropper { ticket, counter: &counter }
        };

        let mut v = Vec::new();
        v.push(d(0));
        v.push(d(1));
        v.push(d(2));
        v.push(d(3));
        v.push(d(4));
        v.push(d(5));


        let mut iter = v.into_iter();
        iter.next().unwrap();
        iter.next().unwrap();
        iter.next().unwrap();
        assert_eq!(counter.get(), 3);

        drop(iter);
        assert_eq!(counter.get(), 6);


        // no item
        let v : Vec<Dropper> = Vec::new();
        assert!(v.into_iter().next().is_none());

        // one item
        let mut v : Vec<i32> = Vec::new();
        v.push(69);

        let mut iter = v.into_iter();
        assert_eq!(iter.next(), Some((0, 69)));
        assert_eq!(iter.next(), None);
    }
}

