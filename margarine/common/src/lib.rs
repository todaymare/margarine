use std::{path::Path, ops::{Deref, DerefMut}, sync::{OnceLock, Mutex}, collections::HashMap};
use istd::index_map;
use sti::arena::Arena;


#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Debug, Hash)]
pub struct SymbolIndex(usize);


pub struct SymbolMap {
    arena: Arena,
    map: HashMap<&'static str, SymbolIndex>,
    vec: Vec<&'static str>,
}


impl SymbolMap {
    #[inline(always)]
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    
    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            map: HashMap::with_capacity(cap),
            vec: Vec::with_capacity(cap),
            arena: Arena::new(),
        }
    }


    #[inline(always)]
    pub fn insert(&mut self, value: &str) -> SymbolIndex {
        if let Some(key) = self.map.get(value) {
            return *key
        }

        let string = unsafe { std::mem::transmute::<&str, &'static str>(self.arena.alloc_str(value)) };
        let index = SymbolIndex(self.vec.len());
        self.vec.push(string);
        self.map.insert(string, index);
        index
    }
    

    #[inline(always)]
    pub fn get<'a>(&'a self, index: SymbolIndex) -> &'a str {
        &self.vec[index.0]
    }


    #[inline(always)]
    pub fn len(&self) -> usize { self.vec.len() }


    #[inline(always)]
    pub fn capacity(&self) -> usize { self.vec.capacity().min(self.map.capacity()) }


    #[inline(always)]
    pub fn is_empty(&self) -> bool { self.vec.is_empty() }
}


impl core::ops::Index<SymbolIndex> for SymbolMap {
    type Output = str;

    fn index<'a>(&'a self, index: SymbolIndex) -> &'a Self::Output {
        &self.vec[index.0]
    }
}


impl std::fmt::Debug for SymbolMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SymbolMap {{ {:?} {:?} }}", self.map, self.vec)
    }
}


impl PartialEq for SymbolMap {
    fn eq(&self, other: &Self) -> bool {
        self.vec == other.vec
    }
}


impl Drop for SymbolMap {
    fn drop(&mut self) {
        self.map.clear();
    }
}


/// A single (immutable) unit of a file
pub struct FileData {
    data: String,
    name: SymbolIndex,
}


impl FileData {
    pub fn new(data: String, name: SymbolIndex) -> Self { 
        let data = data.replace('\t', "    ").replace('\r', "");
        Self { data, name } 
    }


    pub fn open<P: AsRef<Path>>(path: P, symbol_map: &mut SymbolMap) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(&path)?;
        let path = path.as_ref().with_extension("");
        let name = path.to_string_lossy();

        Ok(Self::new(data, symbol_map.insert(&name)))
    }


    #[inline(always)]
    pub fn read(&self) -> &str { &self.data }
    #[inline(always)]
    pub fn name(&self) -> SymbolIndex { self.name }
}


/// Represents the source range of
/// something in byte offset from its
/// respected file data.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct SourceRange {
    start: usize,
    end  : usize,
    file : SymbolIndex,
}


impl SourceRange {
    pub fn new(start: usize, end: usize, file: SymbolIndex) -> Self {
        Self {
            start,
            end,
            file,
        }
    }


    #[inline(always)]
    pub fn range(self) -> (usize, usize) {
        (self.start, self.end)
    }


    #[inline(always)]
    pub fn start(self) -> usize {
        self.start
    }


    #[inline(always)]
    pub fn end(self) -> usize {
        self.end
    }


    #[inline(always)]
    pub fn file(self) -> SymbolIndex {
        self.file
    }
}


pub trait Slice: Deref {
    fn as_slice(&self) -> &<Self as Deref>::Target;
    fn as_mut(&mut self) -> &mut <Self as Deref>::Target;
}


impl<A, T: Deref<Target = [A]> + DerefMut> Slice for T {
    fn as_slice(&self) -> &<Self as Deref>::Target {
        self.deref()
    }

    fn as_mut(&mut self) -> &mut <Self as Deref>::Target {
        self.deref_mut()
    }
}


#[derive(PartialEq, Clone, Copy, Debug)]
pub struct HashableF64(pub f64);


impl std::hash::Hash for HashableF64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state)
    }
}


impl Eq for HashableF64 {}


