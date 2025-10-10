pub mod fmt;
 
use std::fmt::Write;

use common::{string_map::StringMap, source::FileData};
use fmt::ErrorFormatter;
use sti::define_key;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ErrorId {
    Lexer((u32, LexerError)),
    Parser((u32, ParserError)),
    Sema(SemaError),
    Bypass,
}


define_key!(pub LexerError(pub u32));
define_key!(pub ParserError(pub u32));
define_key!(pub SemaError(pub u32));

pub trait ErrorType<T> {
    fn display(&self, fmt: &mut ErrorFormatter, data: &mut T);
}


pub fn display<T>(e: &impl ErrorType<T>, string_map: &StringMap, file: &[FileData], data: &mut T) -> String {
    let mut string = String::new();
    if !string.is_empty() {
        let _ = writeln!(string);
    }

    let mut fmt = ErrorFormatter::new(&mut string, string_map, file);
    e.display(&mut fmt, data);

    string
}
