use std::path::Path;

use crate::string_map::{StringIndex, StringMap};

///
/// A single (immutable) unit of a file
///
#[derive(Clone, Debug)]
pub struct FileData {
    data: String,
    name: StringIndex,
}


impl FileData {
    pub fn new(data: String, name: StringIndex) -> Self { 
        let data = data.replace('\t', "    ").replace('\r', "");
        Self { data, name } 
    }


    pub fn open<P: AsRef<Path>>(path: P, symbol_map: &mut StringMap) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(&path)?;
        let path = path.as_ref().with_extension("");
        let name = path.to_string_lossy();

        Ok(Self::new(data, symbol_map.insert(&name)))
    }


    #[inline(always)]
    pub fn read(&self) -> &str { &self.data }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
}


///
/// Represents the source range of
/// something in byte offset from its
/// respected file data.
///
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash)]
pub struct SourceRange {
    start: u32,
    end  : u32,
}


impl SourceRange {
    pub fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
        }
    }


    #[inline(always)]
    pub fn range(self) -> (u32, u32) {
        (self.start, self.end)
    }


    #[inline(always)]
    pub fn start(self) -> u32 {
        self.start
    }


    #[inline(always)]
    pub fn end(self) -> u32 {
        self.end
    }


    #[inline(always)]
    pub fn file(self, files: &[FileData]) -> &FileData {
        let mut start = 0;
        let mut end;

        for f in files {
            end = start + f.read().len() as u32;

            if self.start <= end {
                assert!(self.end <= end);
                return f;
            }

            start = end;

        }

        panic!("the symbol index isn't within the bounds of the files");
    }
}
