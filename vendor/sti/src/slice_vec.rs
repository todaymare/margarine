use crate::mem::MaybeUninit;


// @todo: kslicevec + from raw parts.
pub struct SliceVec<'a, T> {
    buffer: &'a mut [MaybeUninit<T>],
    len: usize,
}

impl<'a, T> SliceVec<'a, T> {
    #[inline]
    pub const fn new(buffer: &'a mut [MaybeUninit<T>]) -> Self {
        Self {
            buffer,
            len: 0,
        }
    }

    #[inline]
    pub fn cap(&self) -> usize {
        return self.buffer.len();
    }

    #[inline]
    pub fn len(&self) -> usize {
        return self.len;
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        return self.buffer.len() - self.len;
    }


    #[inline]
    pub unsafe fn set_len(&mut self, new_len: usize) {
        debug_assert!(new_len <= self.cap());
        self.len = new_len;
    }

    #[inline]
    pub fn clear(&mut self) {
        unsafe {
            crate::mem::drop_in_place(
                crate::slice::from_raw_parts_mut(
                    self.buffer.as_mut_ptr(), self.len));

            self.len = 0;
        }
    }


    #[must_use]
    #[inline]
    pub fn extend_from_slice(&mut self, values: &[T]) -> usize
    where T: Clone {
        let n = values.len().min(self.remaining());

        unsafe {
            let ptr = self.buffer.as_mut_ptr().add(self.len);
            for i in 0..n {
                ptr.add(i).write(MaybeUninit::new(values[i].clone()));
            }
            self.len += n;
        }

        return n;
    }


    #[inline]
    pub fn as_slice(&self) -> &[T] { unsafe {
        crate::slice::from_raw_parts(self.buffer.as_ptr().cast(), self.len)
    }}

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] { unsafe {
        crate::slice::from_raw_parts_mut(self.buffer.as_mut_ptr().cast(), self.len)
    }}
}

impl<'a, T> Drop for SliceVec<'a, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { crate::mem::drop_in_place(self.as_mut_slice()) }
    }
}


impl<'a, T> crate::ops::Deref for SliceVec<'a, T> {
    type Target = [T];

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'a, T> crate::ops::DerefMut for SliceVec<'a, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}


impl<'a, T: crate::fmt::Debug> crate::fmt::Debug for SliceVec<'a, T> {
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.as_slice().fmt(f)
    }
}

