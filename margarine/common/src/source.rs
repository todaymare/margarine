use std::path::Path;

use crate::string_map::{StringIndex, StringMap};

///
/// A single (immutable) unit of a file
///
#[derive(Clone, Debug)]
pub struct FileData {
    data: String,
    name: StringIndex,
    extension: Extension
}


impl FileData {
    pub fn new(data: String, name: StringIndex, extension: Extension) -> Self { 
        let data = data.replace('\t', "    ").replace('\r', "");
        Self { data, name, extension } 
    }


    pub fn open<P: AsRef<Path>>(path: P, symbol_map: &mut StringMap) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(&path)?;
        let new_path = path.as_ref().with_extension("");
        let name = new_path.to_string_lossy();

        let extension = match path.as_ref().extension() {
            Some(v) => {
                match v.to_str() {
                    Some("mar") => Extension::Mar,
                    Some(val) => Extension::Other(symbol_map.insert(val)),
                    _ => Extension::None,
                }
            },
            None => Extension::None,
        };

        Ok(Self::new(data, symbol_map.insert(&name), extension))
    }


    #[inline(always)]
    pub fn read(&self) -> &str { &self.data }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }

    #[inline(always)]
    pub fn extension(&self) -> Extension { self.extension }
}


#[derive(Clone, Copy, Debug)]
pub enum Extension {
    Mar,
    None,
    Other(StringIndex),
}


impl Extension {
    pub fn read<'a>(&self, string_map: &'a StringMap) -> &'a str {
        match self {
            Extension::Mar => "mar",
            Extension::None => "",
            Extension::Other(v) => string_map.get(*v),
        }
    }
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
    pub const MAX : SourceRange = SourceRange::new(u32::MAX, u32::MAX);
    pub const ZERO : SourceRange = SourceRange::new(0, 0);

    pub const fn new(start: u32, end: u32) -> Self {
        Self {
            start,
            end,
        }
    }


    #[inline(always)]
    pub const fn range(self) -> (u32, u32) {
        (self.start, self.end)
    }


    #[inline(always)]
    pub const fn start(self) -> u32 {
        self.start
    }


    #[inline(always)]
    pub const fn end(self) -> u32 {
        self.end
    }


    #[inline(always)]
    pub fn file(self, files: &[FileData]) -> (&FileData, u32) {
        let mut start = 0;
        let mut end;

        for f in files {
            end = start + f.read().len() as u32;

            if self.start <= end {
                assert!(self.end <= end);
                return (f, start);
            }

            start = end;

        }

        panic!("the symbol index isn't within the bounds of the files");
    }
}
