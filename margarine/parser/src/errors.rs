use std::fmt::Write;

use common::{source::SourceRange, string_map::StringIndex};
use errors::ErrorType;
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
    },


    FileDoesntExist {
        source: SourceRange,
        path: StringIndex,
    },

    HashMismatch {
        source_hash: SourceRange,
        source_extern: SourceRange,
        expected: StringIndex,
        actual: StringIndex,
    },

    InvalidHash {
        source: SourceRange,
    },

    ExternalFileError {
        source: SourceRange,
        url: StringIndex,
        operation: &'static str,
        reason: StringIndex,
    },


    RepoDoesntExist {
        source: SourceRange,
        path: StringIndex,
    },

    TooManyEnumVariants(SourceRange),

    InvalidCfg {
        source: SourceRange,
        expected: &'static str,
    },

    MissingCfgEnvironment {
        source: SourceRange,
        name: StringIndex,
    },
}


impl ErrorType<()> for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter, _: &mut ()) {
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
                    let mut first = true;
                    for i in expected.iter() {
                        if !first {
                            let _ = write!(message, ", ");
                        }
                        first = false;
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


            Error::FileDoesntExist { source, path } => {
                let msg = format!("unable to find file '{}'", fmt.string(*path));
                fmt.error(&msg)
                    .highlight(*source);
            }


            Error::RepoDoesntExist { source, path } => {
                let msg = format!("unable to find git repository '{}'", fmt.string(*path));
                fmt.error(&msg)
                    .highlight(*source);
            }


            Error::TooManyEnumVariants(source) => {
                fmt.error("too many enum variants")
                    .highlight_with_note(*source, "enums cannot have more than 65535 variants");
            }


            Error::InvalidCfg { source, expected } => {
                fmt.error("invalid attribute usage")
                    .highlight_with_note(*source, expected);
            }


            Error::MissingCfgEnvironment { source, name } => {
                let msg = format!("cfg environment variable '{}' is not defined", fmt.string(*name));
                fmt.error("missing cfg environment variable")
                    .highlight_with_note(*source, &msg);
            }

            Error::HashMismatch { source_hash, source_extern, expected, actual } => {
                let note = format!(
                    "but downloaded resource has '{}'",
                    fmt.string(*actual),
                );

                let mut err = fmt.error("resource hash mismatch");
                err
                    .highlight_with_note(*source_hash, "expected this SHA-256 digest");
                err
                    .highlight_with_note(*source_extern, &note);
            }

            Error::InvalidHash { source } => {
                fmt.error("invalid resource hash")
                    .highlight_with_note(*source, "expected exactly 64 hexadecimal characters");
            }

            Error::ExternalFileError { source, url, operation, reason } => {
                let msg = format!("unable to {operation} '{}'", fmt.string(*url));
                let reason = fmt.string(*reason).to_string();
                fmt.error(&msg)
                    .highlight_with_note(*source, &reason);
            }
        }
    }
}
