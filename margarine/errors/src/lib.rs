pub mod fmt;
 
use common::{string_map::StringMap, source::{FileData, SourceRange}};
use fmt::ErrorFormatter;
use sti::define_key;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ErrorId {
    Lexer((u32, LexerError)),
    Parser((u32, ParserError)),
    Sema(SemaError),
}


define_key!(pub LexerError(pub u32));
define_key!(pub ParserError(pub u32));
define_key!(pub SemaError(pub u32));

pub trait ErrorType<T> {
    fn display(&self, fmt: &mut ErrorFormatter, data: &mut T);
}


impl From<SemaError> for ErrorId {
    fn from(value: SemaError) -> Self {
        Self::Sema(value)
    }
}


pub fn display<T>(
    e: &impl ErrorType<T>,
    string_map: &StringMap,
    file: &[FileData],
    data: &mut T,
) -> (String, Option<SourceRange>) {
    let mut string = String::new();
    let primary_range = {
        let mut fmt = ErrorFormatter::new(&mut string, string_map, file);
        e.display(&mut fmt, data);
        fmt.primary_range
    };

    (string, primary_range)
}
