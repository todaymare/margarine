pub mod fmt;
 
use std::fmt::Write;

use common::{string_map::StringMap, source::FileData};
use fmt::ErrorFormatter;
use sti::define_key;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum ErrorId {
    Lexer(LexerError),
    Parser(ParserError),
    Sema(SemaError),
}


define_key!(u32, pub LexerError);
define_key!(u32, pub ParserError);
define_key!(u32, pub SemaError);


pub struct ErrorCode(u32);

pub trait ErrorType<T> {
    fn display(&self, fmt: &mut ErrorFormatter, data: &T);
}


pub fn display<T>(errors: &[impl ErrorType<T>], string_map: &StringMap, file: &[FileData], data: &T) -> String {
    let mut string = String::new();
    for e in errors.iter() {
        if !string.is_empty() {
            let _ = writeln!(string);
        }

        let mut fmt = ErrorFormatter::new(&mut string, string_map, file);
        e.display(&mut fmt, data);
    }

    string
}
