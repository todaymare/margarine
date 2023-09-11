use common::{source::SourceRange, string_map::StringIndex};
use errors::ErrorType;

use crate::typed_ast::Type;

#[derive(Clone, Debug)]
pub enum Error {
    NameIsAlreadyDefined {
        source: SourceRange,
        name: StringIndex,
    },

    UnknownType(StringIndex, SourceRange),

    FunctionBodyAndReturnMismatch {
        source: SourceRange,
        return_type: Type,
        body_type: Type,
    },

    DuplicateField {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    DuplicateArg {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    Bypass,
}


impl ErrorType for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter) {
        match self {
            Error::NameIsAlreadyDefined { source, name } => {
                let name = fmt.string(*name).to_string();
                fmt.error("name is already defined")
                    .highlight_with_note(
                        *source,
                        &format!("there's already a symbol with the name '{name}'"),
                    )
            },

            
            Error::UnknownType(name, pos) => {
                let name = fmt.string(*name).to_string();
                fmt.error("unknown type")
                    .highlight_with_note(
                        *pos,
                        &format!("there's no type named '{name}'"),
                    )
            },

            
            Error::FunctionBodyAndReturnMismatch { .. } => {
                todo!()
            },

            
            Error::DuplicateField { declared_at, error_point } => {
                let mut error = fmt.error("duplicate field");
                error
                    .highlight_with_note(*declared_at, "the field is declared here");
                error.highlight_with_note(*error_point, "..but it's redeclared here");
            
            },

            
            Error::DuplicateArg { declared_at, error_point } => {
                let mut error = fmt.error("duplicate argument");
                error
                    .highlight_with_note(*declared_at, "the argument is declared here");
                error.highlight_with_note(*error_point, "..but it's redeclared here");
            
            },

            
            Error::Bypass => (),
        }
    }
}
