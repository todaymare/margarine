pub use core::num::*;


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NonMaxU32(NonZeroU32);

impl NonMaxU32 {
    pub const MIN: NonMaxU32 = NonMaxU32(NonZeroU32::MAX);
    pub const MAX: NonMaxU32 = NonMaxU32(NonZeroU32::MIN);

    #[inline(always)]
    pub const fn new(value: u32) -> Option<NonMaxU32> {
        match NonZeroU32::new(!value) {
            Some(nz) => Some(NonMaxU32(nz)),
            None => None,
        }
    }

    #[inline(always)]
    pub const unsafe fn new_unck(value: u32) -> NonMaxU32 {
        unsafe { NonMaxU32(NonZeroU32::new_unchecked(!value)) }
    }

    #[inline(always)]
    pub const fn get(self) -> u32 {
        !self.0.get()
    }
}

impl crate::cmp::PartialOrd for NonMaxU32 {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.get().partial_cmp(&other.get())
    }
}

impl crate::cmp::Ord for NonMaxU32 {
    #[inline(always)]
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.get().cmp(&other.get())
    }
}

impl crate::hash::Hash for NonMaxU32 {
    #[inline(always)]
    fn hash<H: crate::hash::Hasher>(&self, state: &mut H) {
        self.get().hash(state)
    }
}

impl crate::fmt::Debug for NonMaxU32 {
    #[inline]
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.get().fmt(f)
    }
}

impl crate::fmt::Display for NonMaxU32 {
    #[inline]
    fn fmt(&self, f: &mut crate::fmt::Formatter) -> crate::fmt::Result {
        self.get().fmt(f)
    }
}

impl From<NonMaxU32> for usize {
    #[inline(always)]
    fn from(value: NonMaxU32) -> Self {
        value.get() as usize
    }
}


#[inline]
pub const fn ceil_to_multiple_pow2(x: usize, n: usize) -> usize {
    debug_assert!(n.is_power_of_two());
    (x + (n-1)) & !(n-1)
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_max_u32() {
        let zero = NonMaxU32::new(0).unwrap();
        assert_eq!(zero, NonMaxU32::MIN);
        assert_eq!(zero.get(), 0);

        let max = NonMaxU32::new(u32::MAX - 1).unwrap();
        assert_eq!(max, NonMaxU32::MAX);
        assert_eq!(max.get(), u32::MAX - 1);

        assert!(NonMaxU32::new(u32::MAX).is_none());
    }

    #[test]
    fn non_max_u32_ord() {
        use core::cmp::Ordering;

        let zero = NonMaxU32::new(0).unwrap();
        let one = NonMaxU32::new(1).unwrap();

        assert!(zero.partial_cmp(&zero).unwrap() == Ordering::Equal);
        assert!(zero.partial_cmp(&one).unwrap() == Ordering::Less);
        assert!(one.partial_cmp(&zero).unwrap() == Ordering::Greater);
        assert!(one.partial_cmp(&one).unwrap() == Ordering::Equal);

        assert!(zero.cmp(&zero) == Ordering::Equal);
        assert!(zero.cmp(&one) == Ordering::Less);
        assert!(one.cmp(&zero) == Ordering::Greater);
        assert!(one.cmp(&one) == Ordering::Equal);
    }
}

