///
/// A float wrapper that makes it
/// a hashable value
///
#[derive(Clone, Copy, Debug)]
pub struct HashableF64(pub f64);

impl std::hash::Hash for HashableF64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


impl PartialEq for HashableF64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}


impl Eq for HashableF64 {}


///
/// A float wrapper that makes it
/// a hashable value
///
#[derive(Clone, Copy, Debug)]
pub struct HashableF32(pub f32);

impl std::hash::Hash for HashableF32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


impl PartialEq for HashableF32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for HashableF32 {}
