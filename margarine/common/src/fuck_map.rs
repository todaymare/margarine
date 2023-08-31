use std::collections::HashMap;

pub type FxHashBuilder = std::hash::BuildHasherDefault<sti::hash::fxhash::FxHasher64>;


pub struct FuckMap<K, V> {
    map: HashMap<K, V, FxHashBuilder>
}


impl<K, V> FuckMap<K, V> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }


    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(cap, FxHashBuilder::default()),
        }
    }
}


impl<K, V> std::ops::Deref for FuckMap<K, V> {
    type Target = HashMap<K, V, FxHashBuilder>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}


impl<K, V> std::ops::DerefMut for FuckMap<K, V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.map
    }
}


impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for FuckMap<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.map)
    }
}


impl<K: Clone, V: Clone> Clone for FuckMap<K, V> {
    fn clone(&self) -> Self {
        Self { map: self.map.clone() }
    }
}


impl<K, V> Default for FuckMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}


impl<K, V> IntoIterator for FuckMap<K, V> {
    type Item = <HashMap<K, V> as IntoIterator>::Item;

    type IntoIter = <HashMap<K, V> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

