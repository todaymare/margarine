pub use lexer::lex;
pub use parser::parse;
pub use semantic_analysis::semantic_analysis;
// pub use ir::convert;
// pub use codegen::codegen;
pub use common::source::{FileData, Extension};
pub use common::string_map::StringMap;
pub use common::DropTimer;
pub use wasm::*;
pub use errors::display;
// pub use ::runtime::*;

