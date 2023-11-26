///
/// A float wrapper that makes it
/// a hashable value
///
#[derive(Clone, Copy, Debug)]
pub struct NonNaNF64(f64);

impl NonNaNF64 {
    pub fn new(val: f64) -> Self {
        assert!(!val.is_nan());
        Self(val)
    }

    pub fn inner(self) -> f64 { self.0 }
}

impl std::hash::Hash for NonNaNF64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


impl PartialEq for NonNaNF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}


impl Eq for NonNaNF64 {}


///
/// A float wrapper that makes it
/// a hashable value
///
#[derive(Clone, Copy, Debug)]
pub struct NonNaNF32(pub f32);

impl std::hash::Hash for NonNaNF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


impl PartialEq for NonNaNF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for NonNaNF32 {}
