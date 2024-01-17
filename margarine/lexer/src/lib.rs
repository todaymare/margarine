use std::ops::{Deref, DerefMut};

use ::errors::LexerError;
use common::{string_map::{StringMap, StringIndex},
    source::{SourceRange, FileData}, hashables::NonNaNF64};
use crate::errors::Error;
use sti::{reader::Reader, keyed::KVec};

mod tests;
pub mod errors;


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

    Error(LexerError),
}


#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Literal {
    Integer(i64),
    Float(NonNaNF64),
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
    Use,
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
    As,
}

pub fn lex<'a, 'arena>(
    file: &'a FileData,
    string_map: &'a mut StringMap<'arena>
) -> (TokenList, KVec<LexerError, Error>) {
    {
        //
        // https://en.wikipedia.org/wiki/Heaps'_law
        // K = 8
        // B = 0.384
        // Numbers came from my asshole
        //
        let unique_ident_count = 8.0 * (file.read().len() as f64).powf(0.384);
        string_map.reserve(unique_ident_count as usize);
    }

    let mut lexer = Lexer {
        reader: Reader::new(file.read().as_bytes()),
        string_map,
        errors: KVec::new(),
    };


    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();

        let is_eof = token.token_kind == TokenKind::EndOfFile;
        tokens.push(token);

        if is_eof {
            break;
        }
    }

    (TokenList::new(tokens), lexer.errors)

}


#[derive(Debug)]
struct Lexer<'a, 's> {
    reader: Reader<'a, u8>,
    string_map: &'a mut StringMap<'s>,
    errors: KVec<LexerError, Error>,
}


impl Lexer<'_, '_> {
    fn skip_whitespace(&mut self) {
        self.reader.consume_while(|x| x.is_ascii_whitespace());
    }
}


impl Lexer<'_, '_> {
    fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        while self.reader.starts_with(b"//") {
            self.reader.consume_while(|x| *x != b'\n');
            self.skip_whitespace();
        }


        let start = self.reader.offset() as u32;
        let Some(val) = self.reader.next() else { return self.eof() };

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


            b'"' => self.string(start as usize),

            _ if val.is_ascii_alphabetic() || val == b'_' => self.identifier(start as usize),


            _ if val.is_ascii_digit() => self.number(start as usize),
            

            _ => TokenKind::Error(
                self.errors.push(Error::InvalidCharacter { 
                    character: val as char, 
                    position: SourceRange::new(start, start) 
                }))
        };

        let end = self.reader.offset() as u32 - 1;
        let source_range = SourceRange::new(start, end);

        Token {
            token_kind: kind,
            source_range,
        }
    }


    fn eof(&self) -> Token {
        Token { 
            token_kind: 
            TokenKind::EndOfFile, 
            source_range: SourceRange::new(
                self.reader.offset() as u32 - 1,
                self.reader.offset() as u32 - 1,
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
            "use"       => TokenKind::Keyword(Keyword::Use),
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
            "as"        => TokenKind::Keyword(Keyword::As),

            "true"      => TokenKind::Literal(Literal::Bool(true)),
            "false"     => TokenKind::Literal(Literal::Bool(false)),

            _ => {
                let index = self.string_map.insert(value);
                TokenKind::Identifier(index)
            }
        }
    }


    fn number(&mut self, begin: usize) -> TokenKind {
        let mut dot_count = 0;
        let (value, _) = self.reader.consume_while_slice_from(begin, |x| {
            if *x == b'.' {
                dot_count += 1;
                return true
            }

            x.is_ascii_digit()
        });
        
        let value = unsafe { core::str::from_utf8_unchecked(value) };

        let source = SourceRange::new(begin as u32, self.reader.offset() as u32 - 1);

        let kind = match dot_count {
            0 => {
                match value.parse() {
                    Ok(e) => Literal::Integer(e),
                    Err(_) => return TokenKind::Error(
                        self.errors.push(Error::NumberTooLarge(source))),
                }
            },
            1 => {
                match value.parse() {
                    Ok(e) => Literal::Float(NonNaNF64::new(e)),
                    Err(_) => return TokenKind::Error(
                        self.errors.push(Error::NumberTooLarge(source))),
                }
            },
            _ =>  return TokenKind::Error(
                self.errors.push(Error::TooManyDots(source)))
        };


        TokenKind::Literal(kind)
    }


    fn string(&mut self, start: usize) -> TokenKind {
        let (str, recover) = {
            let mut is_escaped = false;
            let mut clone = self.reader.clone();
            let (value, _) = clone.consume_while_slice(|at| {
                let done = !is_escaped && *at == b'"';
                is_escaped = *at == b'\\' as u8 && !is_escaped;
                return !done;
            });

            if clone.next() != Some(b'"') {
                self.reader.set_offset(clone.offset());
                let err = Error::UnterminatedString(SourceRange::new(
                    start as u32, 
                    clone.offset() as u32 - 1
                ));

                let err = self.errors.push(err);
                return TokenKind::Error(err)
            }
            
            (value, clone)
        };

        let mut string = String::with_capacity(str.len());

        let mut is_in_escape = false;
        while let Some(value) = self.reader.next() {
            if is_in_escape {
                match value {
                    b'n' => string.push('\n'),
                    b'r' => string.push('\r'),
                    b't' => string.push('\t'),
                    b'\\' => string.push('\\'),
                    b'0' => string.push('\0'),
                    b'"' => string.push('"'),

                    b'u' => match self.unicode_escape_character() {
                        Ok(v) => string.push(v),
                        Err(e) => {
                            self.reader = recover;
                            return TokenKind::Error(self.errors.push(e));
                        },
                    },

                    _ => string.push(value as char),
                }

                is_in_escape = false;

                continue;
            }

            match value {
                b'\\' => is_in_escape = true,
                b'"' => break,
                _ => string.push(value as char),
            }
        }

        let index = self.string_map.insert(&string);
        return TokenKind::Literal(Literal::String(index));
    }


    fn unicode_escape_character(&mut self) -> Result<char, Error> {
        if self.reader.peek() != Some(b'{') {
            let offset = self.reader.offset() as u32;
            return Err(Error::CorruptUnicodeEscape(SourceRange::new(
                offset as u32, offset as u32
            )));
        }

        let start = self.reader.offset()-2;
        let _ = self.reader.next();
        
        let (unicode, _) = self.reader.consume_while_slice(|x| x.is_ascii_hexdigit());
        let unicode = unsafe { core::str::from_utf8_unchecked(unicode) };

        if self.reader.peek() != Some(b'}') {
            return Err(Error::CorruptUnicodeEscape(SourceRange::new(
                start as u32, self.reader.offset() as u32
            )));
        }
        let _ = self.reader.next();

        let source = SourceRange::new(start as u32, self.reader.offset() as u32);
        
        let val = match u32::from_str_radix(unicode, 16) {
            Ok(v) => v,
            Err(_) => return Err(Error::NumberTooLarge(source))
        };

        match char::from_u32(val) {
            Some(value) => Ok(value),
            None => Err(Error::InvalidUnicodeCharacter(source))
        }
    }
}
