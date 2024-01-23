use std::fmt::Write;

use common::source::SourceRange;
use errors::{ErrorType};
use lexer::TokenKind;

#[derive(Clone, Debug)]
pub enum Error {
    ExpectedLiteralString {
        source: SourceRange,
        token: TokenKind,
    },
    
    ExpectedLiteralBool {
        source: SourceRange,
        token: TokenKind,
    },
    
    ExpectedIdentifier {
        source: SourceRange,
        token: TokenKind,
    },

    UnexpectedToken(SourceRange),

    ExpectedXFoundY {
        source: SourceRange,
        found: TokenKind,
        expected: TokenKind,
    },

    ExpectedXFoundYMulti {
        source: SourceRange,
        found: TokenKind,
        expected: &'static [TokenKind],
    },

    DeclarationOnlyBlock {
        source: SourceRange,
    }
}


impl ErrorType<()> for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter, _: &()) {
        match self {
            Error::ExpectedLiteralString { source, token } => {
                fmt.error("expected literal")
                    .highlight_with_note(
                        *source,
                        &format!("expected a string literal, found '{token:?}'"),
                    )
            },


            Error::ExpectedLiteralBool { source, token } => {
                fmt.error("expected literal")
                    .highlight_with_note(
                        *source,
                        &format!("expected a boolean literal, found '{token:?}'"),
                    )
            },
            
            
            Error::ExpectedIdentifier { source, token } => {
                fmt.error("expected identifier")
                    .highlight_with_note(
                        *source,
                        &format!("expected an identifier, found '{token:?}'"),
                    )
            },

            
            Error::UnexpectedToken(source) => {
                fmt.error("unexpected token")
                    .highlight(*source)
            },

            
            Error::ExpectedXFoundY { source, found, expected } => {
                fmt.error("expected a different token")
                    .highlight_with_note(
                        *source,
                        &format!("expected {expected:?}, found '{found:?}'"),
                    )
            },

            
            Error::ExpectedXFoundYMulti { source, found, expected } => {
                let message = {
                    let mut message = String::new();
                    for i in expected.iter() {
                        if message.is_empty() {
                            let _ = write!(message, ", ");
                        }

                        let _ = write!(message, "'{i:?}'");
                    }

                    message
                };

                
                fmt.error("expected a different token")
                    .highlight_with_note(
                        *source,
                        &format!("expected {message}, found '{found:?}'"),
                    )
            },


            Error::DeclarationOnlyBlock { source } => {
                fmt.error("this block only allows declarations")
                    .highlight(*source);
            },
        }
    }
}
