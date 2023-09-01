use std::marker::PhantomData;

use sti::{prelude::Arena, arena::ArenaStats, hash::{HashMap, HashFn, fxhash::FxHasher32, HashMapF}};

use crate::fuck_map::FuckMap;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Debug, Hash)]
pub struct StringIndex(u32);


pub struct StringMap {
    arena: Arena,
    map: sti::hash::HashMapF<HashStr<'static>, StringIndex, HashStrHashFn>,
    // map: FuckMap<&'static str, StringIndex>,
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
            map: HashMapF::fwith_cap(cap),
            // map: FuckMap::with_capacity(cap),
            vec: Vec::with_capacity(cap),
            arena: Arena::new(),
        }
    }


    #[inline(always)]
    pub fn insert(&mut self, value: &str) -> StringIndex {
        let key = HashStr::new(unsafe { std::mem::transmute(value) });
        *self.map.get_or_insert_with_key(&key, |_| {
            debug_assert!(self.vec.len() < u32::MAX as usize);

            let alloc = self.arena.alloc_str(value);

            //
            // This is fine to do since arena is
            // owned by StringMap and thus as long
            // as the API doesn't give out `'static str`s
            // around carelessly it should be fine
            //
            let string = unsafe { std::mem::transmute::<&str, &'static str>(alloc) };

            let index = StringIndex(self.vec.len() as u32);
            self.vec.push(string);
            (HashStr::with_hash(string, key.hash()), index)
        })
    }


    #[inline(always)]
    pub fn get<'a>(&'a self, index: StringIndex) -> &'a str {
        &self.vec[index.0 as usize]
    }


    #[inline(always)]
    pub fn len(&self) -> usize { self.vec.len() }


    #[inline(always)]
    pub fn capacity(&self) -> usize { self.vec.capacity().min(self.map.cap()) }


    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.vec.is_empty() }


    #[inline(always)]
    pub fn arena_stats(&self) -> ArenaStats {
        self.arena.stats()
    }
}


impl std::fmt::Debug for StringMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StringMap")
            .field("vec", &self.vec)
            .finish()
    }
}


impl PartialEq for StringMap {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}


impl Drop for StringMap {
    fn drop(&mut self) {
        // self.map.clear();
    }
}


#[derive(Clone, Copy)]
pub struct HashStr<'a> {
    ptr: *const u8,
    len: u32,
    hash: u32,
    phantom: PhantomData<&'a str>,
}


impl<'a> HashStr<'a> {
    pub fn new(str: &'a str) -> Self {
        let ptr = str.as_ptr();
        let len = str.len().try_into().unwrap();
        let hash = FxHasher32::hash_bytes(str.as_bytes());
        Self { ptr, len, hash, phantom: PhantomData }
    }

    
    #[inline(always)]
    pub fn with_hash(str: &'a str, hash: u32) -> Self {
        let ptr = str.as_ptr();
        let len = str.len().try_into().unwrap();
        Self { ptr, len, hash, phantom: PhantomData }
    }


    #[inline(always)]
    pub fn hash(&self) -> u32 { self.hash }


    #[inline(always)]
    pub fn as_str(&self) -> &'a str {
        unsafe {
            core::str::from_utf8_unchecked(
                core::slice::from_raw_parts(
                    self.ptr, self.len as usize))
        }
    }
}


impl<'a> core::ops::Deref for HashStr<'a> {
    type Target = str;

    #[inline(always)]
    fn deref(&self) -> &Self::Target { self.as_str() }
}


impl<'a> PartialEq for HashStr<'a> {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.as_str() == other.as_str()
    }
}


impl<'a> Eq for HashStr<'a> {}


struct HashStrHashFn;

impl<'a> HashFn<HashStr<'a>> for HashStrHashFn {
    type Seed = ();
    type Hash = u32;

    const DEFAULT_SEED: () = ();

    #[inline(always)]
    fn hash_with_seed(_: (), value: &HashStr<'a>) -> u32 {
        value.hash
    }
}