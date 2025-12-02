use crate::mem::MaybeUninit;
use crate::key::Key;
use crate::slice::KSlice;


pub type StaticVec<V, const N: usize> = KStaticVec<usize, V, N>;

pub struct KStaticVec<K: Key, V, const N: usize> {
    values: [MaybeUninit<V>; N],
    len: K,
}

impl<K: Key, V, const N: usize> KStaticVec<K, V, N> {
    #[inline]
    pub const fn new() -> Self {
        Self {
            values: unsafe {
                // cf MaybeUninit::uninit_array()
                MaybeUninit::<[MaybeUninit<V>; N]>::uninit().assume_init()
            },
            len: K::MIN,
        }
    }

    #[inline(always)]
    pub fn cap(&self) -> usize { N }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len.usize() }

    #[inline(always)]
    pub fn klen(&self) -> K { self.len }


    #[inline]
    pub fn push(&mut self, value: V) -> Result<K, V> {
        if self.len.usize() < N {
            return Ok(unsafe { self.push_unck(value) })
        }
        else {
            return Err(value);
        }
    }

    #[inline]
    pub fn push_strict(&mut self, value: V) -> K {
        if self.len.usize() < N {
            return unsafe { self.push_unck(value) };
        }
        else {
            panic!("StaticVec overflow");
        }
    }

    /// # safety:
    /// - `self.len() < self.cap()`
    #[inline]
    pub unsafe fn push_unck(&mut self, value: V) -> K { unsafe {
        debug_assert!(self.len.usize() < N);

        let result = self.len;

        self.values.get_unchecked_mut(self.len.usize()).write(value);

        self.len = K::from_usize_unck(self.len.usize() + 1);
        return result;
    }}


    #[inline]
    pub fn extend_from_slice(&mut self, values: &[V]) -> usize
    where V: Clone {
        let n = values.len().min(self.cap().usize() - self.len.usize());

        unsafe {
            let ptr = self.values.as_mut_ptr().add(self.len.usize());
            for i in 0..n {
                ptr.add(i).write(MaybeUninit::new(values[i].clone()));
            }
            self.len = K::from_usize_unck(self.len.usize() + n);
        }

        return n;
    }

    #[inline]
    pub fn pop(&mut self) -> Option<V> {
        if self.len.usize() > 0 { unsafe {
            let last = self.len.sub(1);

            let ptr = self.values.as_mut_ptr().add(last.usize());
            let result = ptr.read().assume_init();

            self.len = last;
            return Some(result);
        }}
        else { None }
    }


    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap());
        self.len = unsafe { K::from_usize_unck(new_len) };
    }

    pub fn truncate(&mut self, new_len: usize) {
        assert!(new_len <= self.len.usize());
        if new_len == self.len.usize() {
            return;
        }

        unsafe {
            let ptr = self.values.as_mut_ptr().add(new_len).cast::<V>();
            let len = self.len.usize() - new_len;

            crate::mem::drop_in_place(
                crate::slice::from_raw_parts_mut(ptr, len));

            self.len = K::from_usize_unck(new_len);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.truncate(0);
    }


    #[inline]
    pub fn as_slice(&self) -> &[V] { unsafe {
        crate::slice::from_raw_parts(self.values.as_ptr().cast(), self.len.usize())
    }}

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [V] { unsafe {
        crate::slice::from_raw_parts_mut(self.values.as_mut_ptr().cast(), self.len.usize())
    }}

    #[inline(always)]
    pub fn as_kslice(&self) -> &KSlice<K, V> {
        unsafe { KSlice::new_unck(self.as_slice()) }
    }

    #[inline(always)]
    pub fn as_mut_kslice(&mut self) -> &mut KSlice<K, V> {
        unsafe { KSlice::new_mut_unck(self.as_mut_slice()) }
    }

    #[inline]
    pub fn uninit_slice_mut(&mut self) -> &mut [MaybeUninit<V>] { unsafe {
        crate::slice::from_raw_parts_mut(
            self.values.as_mut_ptr().add(self.len().usize()), 
            self.cap().usize() - self.len().usize())
    }}
}

impl<K: Key, V, const N: usize> Drop for KStaticVec<K, V, N> {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::mem::drop_in_place(self.as_mut_slice()) }
    }
}


impl<K: Key, V, const N: usize> crate::ops::Deref for KStaticVec<K, V, N> {
    type Target = KSlice<K, V>;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_kslice()
    }
}

impl<K: Key, V, const N: usize> crate::ops::DerefMut for KStaticVec<K, V, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_kslice()
    }
}


impl<K: Key + crate::fmt::Debug, V: crate::fmt::Debug, const N: usize> crate::fmt::Debug for KStaticVec<K, V, N> {
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.as_kslice().fmt(f)
    }
}


impl<K: Key, const N: usize> crate::fmt::Write for KStaticVec<K, u8, N> {
    fn write_str(&mut self, s: &str) -> crate::fmt::Result {
        if s.len() <= self.cap().usize() - self.len().usize() { unsafe {
            crate::mem::copy_nonoverlapping(
                s.as_ptr(),
                self.values.as_mut_ptr().cast::<u8>().add(self.len().usize()),
                s.len());

            self.len = K::from_usize_unck(self.len().usize() + s.len());

            return Ok(());
        }}
        else { Err(crate::fmt::Error) }
    }
}

