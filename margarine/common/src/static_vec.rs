use core::mem::MaybeUninit;


pub struct StaticVec<T, const N: usize> {
    pub(crate) values: [MaybeUninit<T>; N],
    len: usize,
}

impl<T, const N: usize> StaticVec<T, N> {
    #[inline(always)]
    pub const fn new() -> Self {
        Self {
            values: unsafe {
                // cf MaybeUninit::uninit_array()
                MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init()
            },
            len: 0,
        }
    }

    #[inline(always)]
    pub fn cap(&self) -> usize { N }

    #[inline(always)]
    pub fn len(&self) -> usize { self.len }


    #[inline(always)]
    pub fn push(&mut self, value: T) -> Result<&mut T, T> {
        if self.len < N {
            return Ok(unsafe { self.push_unck(value) })
        }

        Err(value)
    }


    /// # safety:
    /// - `self.len() < self.cap()`
    #[inline(always)]
    pub unsafe fn push_unck(&mut self, value: T) -> &mut T {
        debug_assert!(self.len < N);

        let result = unsafe {
            self.values.get_unchecked_mut(self.len)
            .write(value)
        };
        self.len += 1;
        return result;
    }


    #[inline(always)]
    pub fn as_slice(&self) -> &[T] {
        unsafe {
            core::slice::from_raw_parts(
                self.values.as_ptr() as *const T,
                self.len)
        }
    }

    #[inline(always)]
    pub fn as_slice_mut(&mut self) -> &mut [T] {
        unsafe {
            core::slice::from_raw_parts_mut(
                self.values.as_ptr() as *mut T,
                self.len)
        }
    }
}

impl<T, const N: usize> Drop for StaticVec<T, N> {
    #[inline(always)]
    fn drop(&mut self) {
        unsafe {
            let slice = self.as_slice_mut();
            core::ptr::drop_in_place(slice);
        }
    }
}

impl<T, const N: usize> core::ops::Deref for StaticVec<T, N> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> core::ops::DerefMut for StaticVec<T, N> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_slice_mut()
    }
}

impl<T, const N: usize, I> core::ops::Index<I> for StaticVec<T, N>
where I: core::slice::SliceIndex<[T]>{
    type Output = I::Output;

    #[inline(always)]
    fn index(&self, index: I) -> &Self::Output {
        core::ops::Index::index(&**self, index)
    }
}

impl<T, const N: usize, I> core::ops::IndexMut<I> for StaticVec<T, N>
where I: core::slice::SliceIndex<[T]>{
    #[inline(always)]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        core::ops::IndexMut::index_mut(&mut **self, index)
    }
}

