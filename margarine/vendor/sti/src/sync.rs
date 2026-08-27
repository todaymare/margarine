
pub struct AssertSync<T>(T);

impl<T> AssertSync<T> {
    #[inline]
    pub const unsafe fn new(value: T) -> Self {
        Self(value)
    }

    #[inline]
    pub const fn get(&self) -> &T {
        &self.0
    }
}

unsafe impl<T> Sync for AssertSync<T> {}

