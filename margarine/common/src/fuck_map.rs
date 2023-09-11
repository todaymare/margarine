use sti::{hash::{HashMap, DefaultSeed}, prelude::{Alloc, GlobalAlloc}};
use std::hash::Hash;

pub type FxHashBuilder = std::hash::BuildHasherDefault<sti::hash::fxhash::FxHasher64>;


pub struct FuckMap<K: Eq + Hash, V, A: Alloc = GlobalAlloc> {
    map: HashMap<K, V, DefaultSeed, A>,
}


impl<K: Eq + Hash, V> FuckMap<K, V, GlobalAlloc> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }


    pub fn with_capacity(cap: usize) -> Self {
        Self::with_capacity_in(cap, GlobalAlloc)
    }
}


impl<K: Eq + Hash, V, A: Alloc> FuckMap<K, V, A> {
    pub fn new_in(alloc: A) -> Self {
        Self::with_capacity_in(0, alloc)
    }


    pub fn with_capacity_in(cap: usize, alloc: A) -> Self {
        Self { map: HashMap::with_cap_in(alloc, cap) }
    }


    pub fn copy_in<A2: Alloc>(&self, alloc: A2) -> FuckMap<K, V, A2>
    where K: Copy, V: Copy {
        FuckMap { map: self.map.copy_in(alloc) }
    }


    pub fn move_into<A2: Alloc>(self, alloc: A2) -> FuckMap<K, V, A2> {
        FuckMap { map: self.map.move_into(alloc) }
    }
}


impl<K: Eq + Hash, V, A: Alloc> std::ops::Deref for FuckMap<K, V, A> {
    type Target = HashMap<K, V, DefaultSeed, A>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}


impl<K: Eq + Hash, V, A: Alloc> std::ops::DerefMut for FuckMap<K, V, A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}


impl<K: Eq + Hash + std::fmt::Debug, V: std::fmt::Debug, A: Alloc> std::fmt::Debug for FuckMap<K, V, A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.map)
    }
}


impl<K: Eq + Hash + Clone, V: Clone, A: Alloc + Clone> Clone for FuckMap<K, V, A> {
    fn clone(&self) -> Self {
        Self { map: self.map.clone() }
    }
}


impl<K: Eq + Hash, V> Default for FuckMap<K, V, GlobalAlloc> {
    fn default() -> Self {
        Self::new()
    }
}

