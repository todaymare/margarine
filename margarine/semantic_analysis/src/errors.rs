use common::{source::SourceRange, string_map::StringIndex};
use errors::ErrorType;
use parser::nodes::{BinaryOperator, UnaryOperator};
use sti::keyed::{KSlice, KVec};

use crate::{Type, TypeId, TypeSymbolKind, TypeSymbol};

#[derive(Clone, Debug)]
pub enum Error {
    NameIsAlreadyDefined {
        source: SourceRange,
        name: StringIndex,
    },

    UnknownType(StringIndex, SourceRange),

    FunctionBodyAndReturnMismatch {
        header: SourceRange,
        item: SourceRange,
        return_type: Type,
        body_type: Type,
    },

    InvalidType {
        source: SourceRange,
        found: Type,
        expected: Type,
    },

    DuplicateField {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    DuplicateArg {
        declared_at: SourceRange,
        error_point: SourceRange,
    },

    VariableValueAndHintDiffer {
        value_type: Type,
        hint_type: Type,
        source: SourceRange,
    },

    VariableNotFound {
        name: StringIndex,
        source: SourceRange,
    },

    InvalidBinaryOp {
        operator: BinaryOperator,
        lhs: Type,
        rhs: Type,
        source: SourceRange,
    },

    InvalidUnaryOp {
        operator: UnaryOperator,
        rhs: Type,
        source: SourceRange,
    },

    IfBodyAndElseMismatch {
        body: (SourceRange, Type),
        else_block: (SourceRange, Type),
    },
    
    Bypass,
}


impl<'a> ErrorType<KVec<TypeId, TypeSymbol<'a>>> for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter, types: &KVec<TypeId, TypeSymbol<'a>>) {
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

            
            Error::InvalidType { source, found, expected } => {
                let msg = format!("expected a value of type '{}' but found '{}'",
                    expected.to_string(types, fmt.string_map()),
                    found.to_string(types, fmt.string_map()),
                );
                
                fmt.error("invalid type")
                    .highlight_with_note(
                        *source,
                        &msg,
                    )
            },

            
            Error::FunctionBodyAndReturnMismatch { header, item, return_type, body_type } => {
                let msg = format!("the function returns '{}'",
                    return_type.to_string(types, fmt.string_map()),
                );
                
                let msg2 = format!("but the body returns '{}'",
                    body_type.to_string(types, fmt.string_map()),
                );

                let mut err = fmt.error("function's return type and the body mismatch");
                err.highlight_with_note(*header, &msg);
                err.highlight_with_note(*item, &msg2);
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

            
            Error::VariableValueAndHintDiffer { value_type, hint_type, source } => {
                let msg = format!("the value is '{}' but the hint is '{}'",
                    value_type.to_string(types, fmt.string_map()),
                    hint_type.to_string(types, fmt.string_map()),
                );
                
                fmt
                    .error("variable type & hint differ in types")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::VariableNotFound { name, source } => {
                let msg = format!("no variable named '{}'", fmt.string_map().get(*name));
                fmt.error("variable not found")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::InvalidBinaryOp { operator, lhs, rhs, source } => {
                let msg = format!("can't apply the binary op '{}' between the types '{}' and '{}'",
                    operator,
                    lhs.to_string(types, fmt.string_map()),
                    rhs.to_string(types, fmt.string_map()),
                );

                fmt.error("invalid binary operation")
                    .highlight_with_note(*source, &msg)
            },

            
            Error::InvalidUnaryOp { operator, rhs, source } => {                
                let msg = format!("can't apply the unary op '{}' on type '{}'",
                    operator,
                    rhs.to_string(types, fmt.string_map()),
                );

                fmt.error("invalid binary operation")
                    .highlight_with_note(*source, &msg)
            },
            
            
            Error::IfBodyAndElseMismatch { body, else_block } => {
                let msg = format!("the main branch returns '{}'", 
                    body.1.to_string(types, fmt.string_map()));
                
                let msg2 = format!("but the else branch returns '{}'", 
                    else_block.1.to_string(types, fmt.string_map()));

                let mut err = fmt.error("if branches differ in types");
                err.highlight_with_note(body.0, &msg);
                err.highlight_with_note(else_block.0, &msg2);
            },
            
            
            Error::Bypass => (),
        }
    }
}
