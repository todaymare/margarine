use std::marker::PhantomData;

use crate::TypeId;

#[derive(Clone, Copy, Debug)]
pub struct Code {
    start: *const u8,

    #[cfg(debug_assertions)]
    end: *const u8,
}


impl Code {
    ///
    /// # Safety
    /// - `start` and `end` must be within a valid u8 sequence
    /// - `end` must be bigger or equal to `start`
    ///
    pub unsafe fn from_raw(
        start: *const u8,

        #[cfg(debug_assertions)]
        end: *const u8,
    ) -> Self {
        debug_assert!(start <= end);
        
        Self {
            start,
            #[cfg(debug_assertions)]
            end,
            
        }
    }


    ///
    /// # Safety
    /// The given slice must outlive self
    ///
    pub unsafe fn from_slice(val: &[u8]) -> Self {
        Self::from_raw(
            val.as_ptr(),

            #[cfg(debug_assertions)]
            unsafe { val.as_ptr().add(val.len()) },
        )
        
    }
}


#[derive(Debug, Clone, Copy)]
pub struct ProgramCounter {
    code: Code,
    counter: *const u8,
}


impl ProgramCounter {
    pub fn new(code: Code) -> Self {
        Self {
            code,
            counter: code.start,
        }
    }


    ///
    /// Increments the program counter
    /// by 1, returning the old value
    ///
    /// # Safety
    /// The next byte must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn next(&mut self) -> u8 {
        #[cfg(debug_assertions)]
        assert!(self.counter <= self.code.end);

        let val = unsafe { *self.counter };
        unsafe { self.counter = self.counter.add(1) };
        
        val
    }

    ///
    /// Returns an array of the next N
    /// elements and increments the program
    /// counter by N
    ///
    /// # Safety
    /// The next N bytes must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn next_n<const N: usize>(&mut self) -> [u8; N] {
        #[cfg(debug_assertions)]
        assert!(unsafe { self.counter.add(N) } <= self.code.end);

        core::array::from_fn(|_| self.next())
    }


    ///
    /// Jumps to the given offset using the
    /// `Code`'s start given when creating `Self`
    ///
    /// # Safety
    /// `self.code.start + offset` must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn jump(&mut self, offset: usize) {
        #[cfg(debug_assertions)]
        {
            assert!(self.code.start.add(offset) <= self.code.end);
            assert!(self.code.start.add(offset) >= self.code.start);
        }
        self.counter = unsafe { self.code.start.add(offset) };
    }


    ///
    /// Skips `n` amount of bytes
    ///
    /// # Safety
    /// The next `n` bytes must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn skip(&mut self, n: usize) {
        self.counter = unsafe { self.counter.add(n) };
        #[cfg(debug_assertions)]
        assert!(self.counter <= self.code.end);
    }


    ///
    /// Reads the next 2 bytes as a u16
    ///
    /// # Safety
    /// The next 2 bytes must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn next_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.next_n::<2>())
    }


    ///
    /// Reads the next 4 bytes as a u32
    ///
    /// # Safety
    /// The next 4 bytes must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.next_n::<4>())
    }


    ///
    /// Reads the next bytes as a string
    /// The string format is the following
    /// - `u32` for the `length` in bytes
    /// - The next `length` bytes are treated
    ///   as UTF-8
    ///
    /// # Safety
    /// - The next 4+`length` bytes must be within the range of `self.code`
    /// - The given string must be valid UTF-8
    ///
    #[inline(always)]
    pub unsafe fn next_str<'a>(&mut self) -> &'a str {
        let size = self.next_u32();

        #[cfg(debug_assertions)]
        assert!(unsafe { self.counter.add(size as usize)} <= self.code.end);

        let slice = unsafe { core::slice::from_raw_parts(self.counter, size as usize) };
        let string = unsafe { core::str::from_utf8_unchecked(slice) };

        self.counter = unsafe { self.counter.add(size as usize) };
        
        string
    }

    ///
    /// Reads the next bytes as a `TypeId`
    ///
    /// # Safety
    /// The next 4 bytes must be within the range of `self.code`
    ///
    #[inline(always)]
    pub unsafe fn next_type(&mut self) -> TypeId {
        TypeId(self.next_u32())
    }
}
