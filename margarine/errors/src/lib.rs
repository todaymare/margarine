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

pub trait ErrorType {
    fn display(&self, fmt: &mut ErrorFormatter);
}


pub fn display(errors: &[impl ErrorType], string_map: &StringMap, file: &[FileData]) -> String {
    let mut string = String::new();
    for e in errors.iter() {
        if !string.is_empty() {
            let _ = writeln!(string);
        }

        let mut fmt = ErrorFormatter::new(&mut string, string_map, file);
        e.display(&mut fmt);
    }

    string
}
