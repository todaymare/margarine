use std::{path::Path, ops::{Deref, DerefMut}, sync::{OnceLock, Mutex}, collections::HashMap};
use istd::index_map;


index_map!(SymbolMap, SymbolIndex, String);


impl SymbolMap {
    pub fn const_str(&mut self, str: &'static str) -> SymbolIndex {    
        static MAP: OnceLock<Mutex<HashMap<&'static str, SymbolIndex>>> = OnceLock::new();

        let mut lock = MAP.get_or_init(|| Mutex::new(HashMap::with_capacity(1))).lock().unwrap();
        *lock.entry(str).or_insert(self.insert(str.to_string()))
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
        let name = path.to_string_lossy().to_string();

        Ok(Self::new(data, symbol_map.insert(name)))
    }


    #[inline(always)]
    pub fn read(&self) -> &str { &self.data }
    #[inline(always)]
    pub fn name(&self) -> SymbolIndex { self.name }
}


/// Represents the source range of
/// something in byte offset from its
/// respected file data.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
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
