use common::source::SourceRange;
use errors::ErrorType;

#[derive(Clone, Debug)]
pub enum Error {
    InvalidCharacter {
        character: char,
        position: SourceRange,
    },

    UnterminatedString(SourceRange),

    CorruptUnicodeEscape(SourceRange),

    InvalidUnicodeCharacter(SourceRange),

    NumberTooLarge(SourceRange),

    TooManyDots(SourceRange),
}


impl ErrorType<()> for Error {
    fn display(&self, fmt: &mut errors::fmt::ErrorFormatter, _: &mut ()) {
        match self {
            Error::InvalidCharacter { character, position } => {
                fmt.error("invalid character")
                    .highlight_with_note(*position, &format!("'{}'", character));
            },

            
            Error::UnterminatedString(pos) => {
                fmt.error("unterminated string")
                    .highlight(*pos);
            },
        
            
            Error::CorruptUnicodeEscape(pos) => {
                fmt.error("corrupt unicode escape")
                    .highlight_with_note(
                        *pos, 
                        &format!("unicode escapes are formatted like '\\u{{..}}")
                    )
            },

            
            Error::InvalidUnicodeCharacter(pos) => {
                fmt.error("invalid unicode character")
                    .highlight_with_note(
                        *pos,
                        "..is not a valid unicode character"
                    )
            },

            
            Error::NumberTooLarge(pos) => {
                fmt.error("number is too big for current context")
                    .highlight(*pos)
            },

            
            Error::TooManyDots(pos) => {
                fmt.error("too many dots")
                    .highlight(*pos)
            },
        }
    }
}
