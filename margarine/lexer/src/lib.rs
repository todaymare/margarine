use std::ops::{Deref, DerefMut};

use ::errors::LexerError;
use common::{string_map::{StringMap, StringIndex},
    source::{SourceRange, FileData}, hashables::NonNaNF64};
use crate::errors::Error;
use sti::{reader::Reader, vec::KVec};

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
    /// ':'
    SemiColon,
    /// ','
    Comma,
    /// '.'
    Dot,
    /// '..'
    DoubleDot,
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

    /// '|'
    Pipe,

    /// '$'
    DollarSign,
    
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
    Fn,
    Struct,
    Impl,
    Extern,
    Use,
    Type,
    Mod,
    Enum,
    Match,
    If,
    Else,
    Var,
    Loop,
    While,
    Return,
    Break,
    Continue,
    As,
    For,
    In,
    Trait,
    Static,
}

pub fn lex<'a, 'arena>(
    file: &'a FileData,
    string_map: &'a mut StringMap<'arena>,
    source_offset: u32,
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
        source_offset,
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
    source_offset: u32,
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
            b'@' => TokenKind::At,
            b'?' => TokenKind::QuestionMark,
            b'~' => TokenKind::SquigglyDash,
            b'^' => TokenKind::BitwiseXor,
            b';' => TokenKind::SemiColon,
            b'$' => TokenKind::DollarSign,

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

            b'.' => {
                if self.reader.consume_if_eq(&b'.') { TokenKind::DoubleDot }
                else { TokenKind::Dot }
            }

            b'"' => self.string(start as usize),

            _ if val.is_ascii_alphabetic() || val == b'_' => self.identifier(start as usize),


            _ if val.is_ascii_digit() => self.number(start, true),
            

            _ => {
                let slice = &self.reader.original_slice()[self.reader.offset()-1..];
                let char = common::utf8::check_1(slice);
                let len = val.leading_ones().max(1);
                let char = if let Ok(char) = char {
                    let char = &slice[..(slice.len()-char.len())];
                    let char = std::str::from_utf8(char).unwrap().chars().next().unwrap();
                    char
                } else { '�' };

                self.reader.consume(len as usize - 1);

                TokenKind::Error(
                self.errors.push(Error::InvalidCharacter { 
                    character: char, 
                    position: SourceRange::new(self.source_offset + start, self.source_offset + start + len) 
                }))
            }
        };

        let end = self.source_offset + self.reader.offset() as u32 - 1;
        let source_range = SourceRange::new(self.source_offset + start, end);

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
                (self.source_offset + self.reader.offset() as u32).saturating_sub(1),
                (self.source_offset + self.reader.offset() as u32).saturating_sub(1),
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
            "fn"        => TokenKind::Keyword(Keyword::Fn),
            "struct"    => TokenKind::Keyword(Keyword::Struct),
            "impl"      => TokenKind::Keyword(Keyword::Impl),
            "extern"    => TokenKind::Keyword(Keyword::Extern),
            "use"       => TokenKind::Keyword(Keyword::Use),
            "mod"       => TokenKind::Keyword(Keyword::Mod),
            "type"      => TokenKind::Keyword(Keyword::Type),
            "enum"      => TokenKind::Keyword(Keyword::Enum),
            "match"     => TokenKind::Keyword(Keyword::Match),
            "if"        => TokenKind::Keyword(Keyword::If),
            "else"      => TokenKind::Keyword(Keyword::Else),
            "var"       => TokenKind::Keyword(Keyword::Var),
            "loop"      => TokenKind::Keyword(Keyword::Loop),
            "while"     => TokenKind::Keyword(Keyword::While),
            "return"    => TokenKind::Keyword(Keyword::Return),
            "break"     => TokenKind::Keyword(Keyword::Break),
            "continue"  => TokenKind::Keyword(Keyword::Continue),
            "as"        => TokenKind::Keyword(Keyword::As),
            "for"       => TokenKind::Keyword(Keyword::For),
            "in"        => TokenKind::Keyword(Keyword::In),
            "trait"      => TokenKind::Keyword(Keyword::Trait),


            "true"      => TokenKind::Literal(Literal::Bool(true)),
            "false"     => TokenKind::Literal(Literal::Bool(false)),

            _ => {
                let index = self.string_map.insert(value);
                TokenKind::Identifier(index)
            }
        }
    }


    /*
    fn number(&mut self, begin: usize) -> TokenKind {
        let mut dot_count = 0;
        let value = {
            let mut value = sti::string::String::from_str_in(self.string_map.arena(), str::from_utf8(&self.reader.original_slice()[begin..self.reader.offset()]).unwrap());
            loop {
                let Some(x) = self.reader.peek()
                else { break };

                if x == b'.' {
                    let Some(next_next) = self.reader.peek_at(1)
                    else { break };

                    if !next_next.is_ascii_digit() {
                        break
                    }

                    dot_count += 1;
                    self.reader.consume(1);
                    value.push_char(x as char);
                    continue
                }

                if x == b'_' { self.reader.consume(1); continue };
                if !x.is_ascii_digit() { break }

                value.push_char(x as char);
                self.reader.consume(1);
            }

            value.leak()
        };
       
        let source = SourceRange::new(self.source_offset + begin as u32, self.source_offset + self.reader.offset() as u32 - 1);

        let kind = match dot_count {
            0 => {
                match value.parse() {
                    Ok(e) => Literal::Integer(e),
                    Err(e) => {
                        dbg!(e, value);
                        return TokenKind::Error(
                        self.errors.push(Error::NumberTooLarge(source)))
                    },
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
    }*/


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
                    self.source_offset + start as u32, 
                    self.source_offset + clone.offset() as u32 - 1
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
            let offset = self.source_offset + self.reader.offset() as u32;
            return Err(Error::CorruptUnicodeEscape(SourceRange::new(
                offset as u32, offset as u32
            )));
        }

        let start = self.reader.offset()-2;
        let _ = self.reader.next();
        
        let (unicode, _) = self.reader.consume_while_slice(|x| x.is_ascii_hexdigit());
        let unicode = unsafe { core::str::from_utf8_unchecked(unicode) };

        if self.reader.peek() != Some(b'}') || unicode.is_empty() {
            return Err(Error::CorruptUnicodeEscape(SourceRange::new(
                self.source_offset + start as u32, self.reader.offset() as u32
            )));
        }

        let source = SourceRange::new(self.source_offset + start as u32, self.reader.offset() as u32);
        let _ = self.reader.next();

        
        let val = match u32::from_str_radix(unicode, 16) {
            Ok(v) => v,
            Err(_) => return Err(Error::NumberTooLarge(source))
        };

        match char::from_u32(val) {
            Some(value) => Ok(value),
            None => Err(Error::InvalidUnicodeCharacter(source))
        }
    }



    fn number(&mut self, start: u32, supports_dot: bool) -> TokenKind {
        self.reader.set_offset(start as usize);
        if true {
            if self.reader.starts_with(b"0b") { return self.based_integer(2) }
            if self.reader.starts_with(b"0o") { return self.based_integer(8) }
            if self.reader.starts_with(b"0x") { return self.based_integer(16) }
        }

        // before decimal 

        let mut has_dot = false;
        let str = self.reader.consume_while_slice(|a| {
            if *a == b'.' {
                if has_dot { return false }
                has_dot = true;
                return true;
            }

            a.is_ascii_digit()
        });

        let mut str = core::str::from_utf8(str.0).unwrap();

        println!("{:?}", self.reader.peek().map(|x| x as char));
        println!("{str}, has_dot: {has_dot}");
        if has_dot && self.reader.peek().map_or(true, |v| !v.is_ascii_digit() && !v.is_ascii_whitespace()) {
            has_dot = false;
            self.reader.set_offset(self.reader.offset()-1);
            str = &str[..str.len()-1];
        }


        if (!has_dot && self.reader.peek() != Some(b'e'))
        || !supports_dot {
            let Ok(int) = i64::from_str_radix(str, 10)
            else {
                dbg!(self.reader.offset());
                panic!();
                let source = SourceRange::new(start as u32, self.reader.offset() as u32-1).offset(self.source_offset);
                let err = Error::NumberTooLarge(source);
                return TokenKind::Error(self.errors.push(err))
            };


            return TokenKind::Literal(Literal::Integer(int))
        }


        let Ok(float) = str.parse::<f64>()
        else {
            let source = SourceRange::new(start as u32, self.reader.offset() as u32-1).offset(self.source_offset);
            let err = Error::NumberTooLarge(source);
            return TokenKind::Error(self.errors.push(err))
        };


        if self.reader.peek() != Some(b'e') {
            return TokenKind::Literal(Literal::Float(NonNaNF64::new(float)))
        }

        self.reader.consume(1);

        let is_neg = self.reader.next_if(|a| *a == b'-').is_some();

        let TokenKind::Literal(Literal::Integer(exponent)) = self.number(self.reader.offset() as u32, false)
        else { unreachable!() };

        let exponent = exponent as i128;
        let exponent = 
            if is_neg { -exponent }
            else { exponent };

        let float = float * 10f64.powi(exponent as i32);

        TokenKind::Literal(Literal::Float(NonNaNF64::new(float)))
    }


    fn based_integer(&mut self, base: u8) -> TokenKind {
        let start = self.reader.offset();
        self.reader.consume(2);
        
        let mut max_base = 0;
        let mut has_dot = false;

        let str = self.reader.consume_while_slice(|a| {
            if a.is_ascii_digit() {
                let digit = a - b'0';
                max_base = max_base.max(digit + 1);
                return true;
            }

            if a.is_ascii_hexdigit() {
                let digit = a.to_ascii_uppercase() - b'A';
                max_base = max_base.max(digit + 10);
                return true;
            }

            if *a == b'.' {
                if has_dot { return false }
                has_dot = true;
                return true;
            }

            return false;
        });


        if has_dot {
            let source = SourceRange::new(start as u32, self.reader.offset() as u32-1).offset(self.source_offset);
            let err = Error::BasedFloatsArentSupported(source);
            return TokenKind::Error(self.errors.push(err))
        }


        if max_base > base {
            let source = SourceRange::new(start as u32, self.reader.offset() as u32-1).offset(self.source_offset);
            let err = Error::InvalidBaseForNumber(source);
            return TokenKind::Error(self.errors.push(err))
        }


        let str = core::str::from_utf8(str.0).unwrap();
        let Ok(num) = i64::from_str_radix(str, base as u32)
        else { 
            let source = SourceRange::new(start as u32, self.reader.offset() as u32-1).offset(self.source_offset);
            let err = Error::NumberTooLarge(source);
            return TokenKind::Error(self.errors.push(err))
        };


        TokenKind::Literal(Literal::Integer(num))
    }


}
