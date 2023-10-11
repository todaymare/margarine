use crate::Data;

#[repr(C)]
pub struct RegisterStack {
    values: Box<[Data]>,
    bottom: usize,

    /// Note: This is not the same as `values.len()` as this
    /// is used for function boundaries
    top: usize,
}


impl RegisterStack {
    ///
    /// Creates a new `RegisterStack` with 0 capacity.
    /// Check `Self::with_capacity` for more information.
    ///
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    
    ///
    /// Creates a new `RegisterStack` with enough capacity to
    /// fit `capacity` elements, all new registers are set
    /// to the return value of `Data::new_unit`.
    ///
    #[inline(always)]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: vec![Data::new_unit(); capacity].into_boxed_slice(),
            bottom: 0,
            top: 0,
        }
    }

    
    ///
    /// Sets the register at `reg` to `data` while accounting
    /// for the bottom boundary, so the resulting index of the
    /// register would be `self.bottom + reg`.
    ///
    /// # Panics (In Debug Mode)
    /// In debug mode if `self.bottom + reg` is *NOT* less than
    /// `self.top` this function will panic.
    ///
    /// # Safety
    /// If `self.bottom + reg` is out of bounds this function will
    /// cause undefined behaviour.
    ///
    #[inline(always)]
    pub unsafe fn set_reg(&mut self, reg: u8, data: Data) {
        debug_assert!((self.bottom + reg as usize) < self.top);
        *unsafe { self.values.get_unchecked_mut(self.bottom + reg as usize) } = data;
    }

    
    ///
    /// Gets the value of the register at `reg` while accounting 
    /// for the bottom boundary, so the resulting index of the
    /// register would be `self.bottom + reg`.
    ///
    /// # Panics (In Debug Mode)
    /// In debug mode if `self.bottom + reg` is *NOT* less than
    /// `self.top` this function will panic.
    ///
    /// # Safety
    /// If `self.bottom + reg` is out of bounds this function will
    /// cause undefined behaviour.
    ///
    #[inline(always)]
    pub unsafe fn reg(&self, reg: u8) -> Data {
        debug_assert!((self.bottom + reg as usize) < self.top);
        *unsafe { self.values.get_unchecked(self.bottom+reg as usize) }
    }
    
    
    ///
    /// Creates a mutable reference to the value at `reg` while 
    /// accounting for the bottom boundary, so the resulting index
    /// of the register would be `self.bottom + reg`
    ///
    /// # Panics (In Debug Mode)
    /// In debug mode if `self.bottom + reg` is *NOT* less than
    /// `self.top` this function will panic.
    ///
    /// # Safety
    /// If `self.bottom + reg` is out of bounds this function will
    /// cause undefined behaviour.
    ///
    #[inline(always)]
    pub unsafe fn reg_mut(&mut self, reg: u8) -> &mut Data {
        debug_assert!((self.bottom + reg as usize) < self.top);
        unsafe { self.values.get_unchecked_mut(self.bottom+reg as usize) }
    }


    ///
    /// Increases the top boundary by `amount`
    ///
    /// # Panics (In Debug Mode)
    /// In debug mode if `self.top + amount` is bigger than
    /// `self.values.len()` this function will panic
    ///
    #[inline(always)]
    pub fn push(&mut self, amount: usize) {
        debug_assert!(self.top + amount <= self.values.len());

        let Some(result) = self.top.checked_add(amount)
        else { panic!("result of `self.top + amount` is out of `usize` bounds") };

        self.top = result;
    }

    
    ///
    /// Decreases the top boundary by `amount`
    ///
    /// # Panics (In Debug Mode)
    /// In debug mode if `self.top - amount` is less than
    /// `self.bottom` this function will panic
    ///
    #[inline(always)]
    pub fn pop(&mut self, amount: usize) {
        debug_assert!(self.top - amount >= self.bottom);

        let Some(result) = self.top.checked_sub(amount)
        else { panic!("result of `self.top - amount` is out of `usize` bounds") };

        self.top = result;
    }
}

