use std::ops::{Deref, DerefMut};

use errors::{Error, CompilerError, ErrorCode, ErrorBuilder, CombineIntoError};
use common::{string_map::{StringMap, StringIndex}, source::{SourceRange, FileData}, hashables::HashableF64};
use sti::reader::Reader;

mod tests;


/// A wrapper around `Vec<Token>` with
/// with the guarantee that it wont be
/// empty.
#[derive(Debug)]
pub struct TokenList {
    vec: Vec<Token>,
}


impl TokenList {
    /// # Panics
    /// if the `vec` is empty
    pub fn new(vec: Vec<Token>) -> Self {
        assert!(!vec.is_empty());
        Self {
            vec,
        }
    }
}


impl Deref for TokenList {
    type Target = [Token];

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}


impl DerefMut for TokenList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}



#[derive(Debug, PartialEq)]
pub struct Token {
    token_kind: TokenKind,
    source_range: SourceRange,
}


impl Token {
    #[inline(always)]
    pub fn kind(&self) -> TokenKind {
        self.token_kind
    }


    #[inline(always)]
    pub fn range(&self) -> SourceRange {
        self.source_range
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TokenKind {
    /// '('
    LeftParenthesis,
    /// ')'
    RightParenthesis,

    /// '<'
    LeftAngle,
    /// '>'
    RightAngle,

    /// '{'
    LeftBracket,
    /// '}'
    RightBracket,

    /// '['
    LeftSquare,
    /// ']'
    RightSquare,

    /// '%'
    Percent,
    /// '/'
    Slash,
    /// '+'
    Plus,
    /// '-'
    Minus,
    /// '*'
    Star,
    /// ':'
    Colon,
    /// '::'
    DoubleColon,
    /// ','
    Comma,
    /// '.'
    Dot,
    /// '!'
    Bang,
    /// '='
    Equals,
    /// '_'
    Underscore,
    /// '@'
    At,
    /// '?'
    QuestionMark,
    /// '&'
    Ampersand,
    /// '~'
    SquigglyDash,

    Literal(Literal),
    Keyword(Keyword),
    Identifier(StringIndex),

    /// '<='
    LesserEquals,
    /// '>='
    GreaterEquals,
    /// '=='
    EqualsTo,
    /// '!='
    NotEqualsTo,
    /// '||'
    LogicalOr,
    /// '&&'
    LogicalAnd,

    /// '+='
    AddEquals,
    /// '-='
    SubEquals,
    /// '*='
    MulEquals,
    /// '/='
    DivEquals,
    /// '%='
    RemEquals,

    /// '<<'
    BitshiftLeft,
    /// '>>'
    BitshiftRight,
    /// '|'
    BitwiseOr,
    /// '^'
    BitwiseXor,

    /// '=>'
    Arrow,
    
    EndOfFile,
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Literal {
    Integer(i64),
    Float(HashableF64),
    String(StringIndex),
    Bool(bool),
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Keyword {
    System,
    Fn,
    Struct,
    Component,
    Resource,
    Impl,
    Extern,
    Using,
    Mut,
    Mod,
    Enum,
    Match,
    If,
    Else,
    Let,
    Loop,
    While,
    Return,
    Break,
    Continue,
}


pub fn lex(
    file: &FileData,
    string_map: &mut StringMap
) -> Result<TokenList, Error> {
    let mut lexer = Lexer {
        reader: Reader::new(file.read().as_bytes()),
        string_map,
    };

    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    loop {
        let token = lexer.next_token();
        let token = match token {
            Ok(t) => t,
            Err(e) => {
                errors.push(e);
                continue;
            },
        };

        let is_eof = token.token_kind == TokenKind::EndOfFile;
        tokens.push(token);

        if is_eof {
            break;
        }
    }


    if !errors.is_empty() {
        return Err(errors.combine_into_error())
    }

    Ok(TokenList::new(tokens))

}


#[derive(Debug)]
struct Lexer<'a> {
    reader: Reader<'a, u8>,
    string_map: &'a mut StringMap,
}


impl Lexer<'_> {
    fn skip_whitespace(&mut self) {
        self.reader.consume_while(|x| x.is_ascii_whitespace());
    }
}


impl Lexer<'_> {
    fn next_token(&mut self) -> Result<Token, Error> {
        if self.reader.starts_with(b"//") {
            self.reader.consume_while(|x| *x != b'\n');
        }

        self.skip_whitespace();


        let start = self.reader.offset() as u32;
        let Some(val) = self.reader.next() else { return Ok(self.eof()) };

        let kind = match val {
            b'(' => TokenKind::LeftParenthesis,
            b')' => TokenKind::RightParenthesis,

            b'{' => TokenKind::LeftBracket,
            b'}' => TokenKind::RightBracket,

            b'[' => TokenKind::LeftSquare,
            b']' => TokenKind::RightSquare,


            b',' => TokenKind::Comma,
            b'.' => TokenKind::Dot,
            b'@' => TokenKind::At,
            b'?' => TokenKind::QuestionMark,
            b'~' => TokenKind::SquigglyDash,
            b'^' => TokenKind::BitwiseXor,


            b'&' => {
                if self.reader.consume_if_eq(&b'&') { TokenKind::LogicalAnd }
                else { TokenKind::Ampersand }
            }

            b'|' => {
                if self.reader.consume_if_eq(&b'|') { TokenKind::LogicalOr }
                else { TokenKind::BitwiseOr }
            }
            
            b'<' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::LesserEquals }
                else if self.reader.consume_if_eq(&b'<') { TokenKind::BitshiftLeft }
                else { TokenKind::LeftAngle }
            }

            b'>' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::GreaterEquals }
                else if self.reader.consume_if_eq(&b'>') { TokenKind::BitshiftRight }
                else { TokenKind::RightAngle }
            }

            b'%' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::RemEquals }
                else { TokenKind::Percent }
            }

            b'/' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::DivEquals }
                else { TokenKind::Slash }
            }

            b'+' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::AddEquals }
                else { TokenKind::Plus }
            }

            b'-' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::SubEquals }
                else { TokenKind::Minus }
            }

            b'*' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::MulEquals }
                else { TokenKind::Star }
            }

            b'!' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::NotEqualsTo }
                else { TokenKind::Bang }
            }

            b'=' => {
                if self.reader.consume_if_eq(&b'=') { TokenKind::EqualsTo }
                else if self.reader.consume_if_eq(&b'>') { TokenKind::Arrow }
                else { TokenKind::Equals }
            }

            b':' => {
                if self.reader.consume_if_eq(&b':') { TokenKind::DoubleColon }
                else { TokenKind::Colon }
            }


            b'"' => self.string(start as usize)?,

            _ if val.is_ascii_alphabetic() || val == b'_' => self.identifier(start as usize),


            _ if val.is_ascii_digit() => self.number(start as usize)?,
            

            _ => return Err(CompilerError::new(ErrorCode::LInvalidCharacter)
                .highlight(SourceRange::new(start, start))
                    .note(format!("'{}'", char::from_u32(val as u32).unwrap()))
                .build()
            )
        };

        let end = self.reader.offset() as u32 - 1;
        let source_range = SourceRange::new(start, end);

        Ok(Token {
            token_kind: kind,
            source_range,
        })
    }


    fn eof(&self) -> Token {
        Token { 
            token_kind: 
            TokenKind::EndOfFile, 
            source_range: SourceRange::new(
                self.reader.offset() as u32,
                self.reader.offset() as u32,
            ) 
        }
    }


    fn identifier(&mut self, begin: usize) -> TokenKind {
        let (value, _)= self.reader.consume_while_slice_from(begin, |x| {
            x.is_ascii_alphanumeric() || *x == b'_'
        });
        
        let value = unsafe { core::str::from_utf8_unchecked(value) };

        match value {
            "_"         => TokenKind::Underscore,
            "system"    => TokenKind::Keyword(Keyword::System),
            "fn"        => TokenKind::Keyword(Keyword::Fn),
            "struct"    => TokenKind::Keyword(Keyword::Struct),
            "component" => TokenKind::Keyword(Keyword::Component),
            "resource"  => TokenKind::Keyword(Keyword::Resource),
            "impl"      => TokenKind::Keyword(Keyword::Impl),
            "extern"    => TokenKind::Keyword(Keyword::Extern),
            "using"     => TokenKind::Keyword(Keyword::Using),
            "mut"       => TokenKind::Keyword(Keyword::Mut),
            "mod"       => TokenKind::Keyword(Keyword::Mod),
            "enum"      => TokenKind::Keyword(Keyword::Enum),
            "match"     => TokenKind::Keyword(Keyword::Match),
            "if"        => TokenKind::Keyword(Keyword::If),
            "else"      => TokenKind::Keyword(Keyword::Else),
            "let"       => TokenKind::Keyword(Keyword::Let),
            "loop"      => TokenKind::Keyword(Keyword::Loop),
            "while"     => TokenKind::Keyword(Keyword::While),
            "return"    => TokenKind::Keyword(Keyword::Return),
            "break"     => TokenKind::Keyword(Keyword::Break),
            "continue"  => TokenKind::Keyword(Keyword::Continue),

            _ => {
                let index = self.string_map.insert(value);
                TokenKind::Identifier(index)
            }
        }
    }


    fn number(&mut self, begin: usize) -> Result<TokenKind, Error> {
        let mut dot_count = 0;
        let (value, _) = self.reader.consume_while_slice_from(begin, |x| {
            if *x == b'.' {
                dot_count += 1;
                return true
            }

            x.is_ascii_digit()
        });
        
        let value = unsafe { core::str::from_utf8_unchecked(value) };

        let source = SourceRange::new(begin as u32, self.reader.offset() as u32);

        let kind = match dot_count {
            0 => {
                match value.parse() {
                    Ok(e) => Literal::Integer(e),
                    Err(_) => return Err(CompilerError::new(ErrorCode::LNumTooLarge)
                        .highlight(source)
                        .build()),
                }
            },
            1 => {
                match value.parse() {
                    Ok(e) => Literal::Float(HashableF64(e)),
                    Err(_) => return Err(CompilerError::new(ErrorCode::LNumTooLarge)
                        .highlight(source)
                        .build()),
                }
            },
            _ =>  return Err(CompilerError::new(ErrorCode::LTooManyDots)
                .highlight(source)
                .build()
            )
        };


        Ok(TokenKind::Literal(kind))
    }


    fn string(&mut self, start: usize) -> Result<TokenKind, Error> {
        let str = {
            let mut is_escaped = false;
            let (value, _) = self.reader.consume_while_slice(|at| {
                let done = !is_escaped && *at == b'"';
                is_escaped = *at == b'\\' as u8 && !is_escaped;
                return !done;
            });

            if self.reader.next() != Some(b'"') {
                return Err(CompilerError::new(ErrorCode::LUnterminatedStr)
                    .highlight(SourceRange::new(start as u32, self.reader.offset() as u32))
                        .note("consider adding a quotation mark here".to_string())

                    .build()
                );
            }
            
            value
        };
        let mut iter = str.iter();

        let mut string = String::with_capacity(str.len());

        let mut errors = vec![];

        let mut is_in_escape = false;
        while let Some(value) = iter.next() {
            if is_in_escape {
                match value {
                    b'n' => string.push('\n'),
                    b'r' => string.push('\r'),
                    b't' => string.push('\t'),
                    b'\\' => string.push('\\'),
                    b'0' => string.push('\0'),
                    b'"' => string.push('"'),

                    b'u' => match self.unicode_escape_character() {
                        Ok(val) => string.push(val),
                        Err(err) => {
                            errors.push(err);
                        },
                    },

                    _ => string.push(*value as char),
                }

                is_in_escape = false;

                continue;
            }

            match value {
                b'\\' => is_in_escape = true,
                b'"' => break,
                _ => string.push(*value as char),
            }
        }

        if errors.is_empty() {
            let index = self.string_map.insert(&string);
            return Ok(TokenKind::Literal(Literal::String(index)));
        }

        Err(errors.combine_into_error())
    }


    fn unicode_escape_character(&mut self) -> Result<char, Error> {
        if self.reader.next() != Some(b'{') {
            let offset = self.reader.offset() as u32;
            return Err(CompilerError::new(ErrorCode::LCorruptUnicodeEsc)
                .highlight(SourceRange::new(offset, offset))
                    .note("unicode escapes are formatted like '\\u{..}'".to_string())

                .build()
            );
        }

        let start = self.reader.offset();
        
        let (unicode, no_eoi) = self.reader.consume_while_slice(|x| x.is_ascii_hexdigit());
        let unicode = unsafe { core::str::from_utf8_unchecked(unicode) };


        if !no_eoi || self.reader.next() != Some(b'}') {
            return Err(CompilerError::new(ErrorCode::LUnterminatedUni)
                .highlight(SourceRange::new(start as u32, self.reader.offset() as u32))
                    .note("unicode escapes need to end with '}'".to_string())
                .build()
            )
        }

        let source = SourceRange::new(start as u32, self.reader.offset() as u32);
        
        let val = match u32::from_str_radix(unicode, 16) {
            Ok(v) => v,
            Err(_) => return Err(CompilerError::new(ErrorCode::LNumTooLarge)
                .highlight(source)
                .build()
            ),
        };

        match char::from_u32(val) {
            Some(value) => Ok(value),
            None => Err(CompilerError::new(ErrorCode::LInvalidUnicodeChr)
                .highlight(source)
                .build()
            ),
        }
    }
}
