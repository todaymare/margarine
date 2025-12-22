use std::path::Path;

use derive::ImmutableData;

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


    pub fn open<P: AsRef<Path>>(path: P, string_map: &mut StringMap) -> Result<Self, std::io::Error> {
        let path = std::fs::canonicalize(path)?;
        let new_path = path.with_extension("");
        let name = new_path.to_string_lossy();

        Self::open_ex(path, string_map.insert(&name), string_map)
    }


    pub fn open_ex<P: AsRef<Path>>(path: P, name: StringIndex, string_map: &mut StringMap) -> Result<Self, std::io::Error> {
        let data = std::fs::read_to_string(&path)?;
        let extension = match path.as_ref().extension() {
            Some(v) => {
                match v.to_str() {
                    Some("mar") => Extension::Mar,
                    Some(val) => Extension::Other(string_map.insert(val)),
                    _ => Extension::None,
                }
            },
            None => Extension::None,
        };

        Ok(Self::new(data, name, extension))
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
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default, Hash, ImmutableData)]
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


    pub fn as_str(mut self, files: &[FileData]) -> &str {
        let (file, base) = self.file(files);
        self = self.base(base);

        &file.read()[self.start as usize..self.end as usize]
    }


    #[inline(always)]
    pub const fn range(self) -> (u32, u32) {
        (self.start, self.end)
    }


    pub const fn base(self, num: u32) -> SourceRange {
        SourceRange::new(self.start - num, self.end - num)
    }


    pub const fn offset(self, num: u32) -> SourceRange {
        SourceRange::new(self.start + num, self.end + num)
    }


    ///
    /// returns the file and that file's offset
    ///
    #[inline(always)]
    pub fn file(self, files: &[FileData]) -> (&FileData, u32) {
        let mut start = 0;
        let mut end;

        for f in files {
            end = start + f.read().len() as u32;

            if self.start < end {
                //assert!(self.end <= end, "Range {}..{} exceeds file bounds {}..{}", self.start, self.end, start, end);
                return (f, start);
            }

            start = end;

        }

        panic!("the symbol index isn't within the bounds of the files");
    }
}
