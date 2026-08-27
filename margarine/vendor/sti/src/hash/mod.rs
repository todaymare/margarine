pub use core::hash::{Hash, Hasher};


pub trait HashFn<T: ?Sized, Hash> {
    fn hash(&self, value: &T) -> Hash;
}


pub mod fxhash;


pub mod hash_map;
pub use hash_map::HashMap;

