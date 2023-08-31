use sti::{prelude::Arena, arena::ArenaStats};

use crate::fuck_map::FuckMap;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Debug, Hash)]
pub struct StringIndex(u32);


pub struct StringMap {
    arena: Arena,
    map: FuckMap<&'static str, StringIndex>,
    vec: Vec<&'static str>,
}


impl StringMap {
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    
    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: FuckMap::with_capacity(cap),
            vec: Vec::with_capacity(cap),
            arena: Arena::new(),
        }
    }


    #[inline(always)]
    pub fn insert(&mut self, value: &str) -> StringIndex {
        if let Some(key) = self.map.get(value) {
            return *key
        }

        let string = unsafe { std::mem::transmute::<&str, &'static str>(self.arena.alloc_str(value)) };

        debug_assert!(self.vec.len() < u32::MAX as usize);
        
        let index = StringIndex(self.vec.len() as u32);
        self.vec.push(string);
        self.map.insert(string, index);
        index
    }
    

    #[inline(always)]
    pub fn get<'a>(&'a self, index: StringIndex) -> &'a str {
        &self.vec[index.0 as usize]
    }


    #[inline(always)]
    pub fn len(&self) -> usize { self.vec.len() }


    #[inline(always)]
    pub fn capacity(&self) -> usize { self.vec.capacity().min(self.map.capacity()) }


    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.vec.is_empty() }


    #[inline(always)]
    pub fn arena_stats(&self) -> ArenaStats {
        self.arena.stats()
    }
}


impl std::fmt::Debug for StringMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymbolMap {{ {:?} {:?} }}", self.map, self.vec)
    }
}


impl PartialEq for StringMap {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}


impl Drop for StringMap {
    fn drop(&mut self) {
        self.map.clear();
    }
}


