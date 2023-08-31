
#![allow(unused)]
use std::fmt::Debug;

use common::{string_map::{StringMap, StringIndex}, source::{SourceRange, FileData}, Slice, hashables::HashableF64};

use crate::{lex, Token, TokenKind, Literal};


#[test]
fn empty() {
    let mut symbol_table = StringMap::new();
    let file_name = symbol_table.insert("test");
    let file_data = FileData::new(String::new(), file_name);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();

    assert_eq!(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::EndOfFile,
            source_range: SourceRange::new(0, 0),
        }
    ].as_slice())
}


#[test]
fn tokens() {
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "() <> {} [] % / + - * ^ : :: , . ! = _ \
                    <= >= == != || && += -= *= /= @ ?";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(&tokens, &vec![
            token(TokenKind::LeftParenthesis, 0, 0),
            token(TokenKind::RightParenthesis, 1, 1),

            token(TokenKind::LeftAngle, 3, 3),
            token(TokenKind::RightAngle, 4, 4),

            token(TokenKind::LeftBracket, 6, 6),
            token(TokenKind::RightBracket, 7, 7),

            token(TokenKind::LeftSquare, 9, 9),
            token(TokenKind::RightSquare, 10, 10),

            token(TokenKind::Percent, 12, 12),
            token(TokenKind::Slash, 14, 14),
            token(TokenKind::Plus, 16, 16),
            token(TokenKind::Minus, 18, 18),
            token(TokenKind::Star, 20, 20),
            token(TokenKind::BitwiseXor, 22, 22),
            token(TokenKind::Colon, 24, 24),
            token(TokenKind::DoubleColon, 26, 27),
            token(TokenKind::Comma, 29, 29),
            token(TokenKind::Dot, 31, 31),
            token(TokenKind::Bang, 33, 33),
            token(TokenKind::Equals, 35, 35),
            token(TokenKind::Underscore, 37, 37),

            token(TokenKind::LesserEquals, 39, 40),
            token(TokenKind::GreaterEquals, 42, 43),
            token(TokenKind::EqualsTo, 45, 46),
            token(TokenKind::NotEqualsTo, 48, 49),
            token(TokenKind::LogicalOr, 51, 52),
            token(TokenKind::LogicalAnd, 54, 55),

            token(TokenKind::AddEquals, 57, 58),
            token(TokenKind::SubEquals, 60, 61),
            token(TokenKind::MulEquals, 63, 64),
            token(TokenKind::DivEquals, 66, 67),

            token(TokenKind::At, 69, 69),
            token(TokenKind::QuestionMark, 71, 71),

            token(TokenKind::EndOfFile, 72, 72)
        ])
    }

    // Invalid character
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = ";";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table);
        assert!(tokens.is_err());
    }
}


#[test]
fn numbers() {
    // valid integer
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "123456789";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::Integer(123456789)),
                source_range: SourceRange::new(0, 8),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(9, 9),
            }
        ].as_slice());
    }


    // valid float
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "420.69";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::Float(HashableF64(420.69))),
                source_range: SourceRange::new(0, 5),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(6, 6),
            }
        ].as_slice());
    }


    // too many dots
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "420.69.50";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table);
        assert!(tokens.is_err())
    }
}


#[test]
fn identifiers() {
    let mut symbol_table = StringMap::new();
    let file = symbol_table.insert("test");
    let data = "hello _there";
    let file_data = FileData::new(data.to_string(), file);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();

    compare_individually(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::Identifier(symbol_table.insert("hello")),
            source_range: SourceRange::new(0, 4),
        },
        Token {
            token_kind: TokenKind::Identifier(symbol_table.insert("_there")),
            source_range: SourceRange::new(6, 11),
        },
        Token {
            token_kind: TokenKind::EndOfFile,
            source_range: SourceRange::new(12, 12),
        },
    ].as_slice())
}


#[test]
fn string() {
    // valid string
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "\"hello there\"";
        let file_data = FileData::new(data.to_string(), file);

        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::String(symbol_table.insert("hello there"))),
                source_range: SourceRange::new(0, 12),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(13, 13),
            },
        ].as_slice())
    }


    // unterminated string
    {
        let mut symbol_table = StringMap::new();
        let file = symbol_table.insert("test");
        let data = "\"hello there";
        let file_data = FileData::new(data.to_string(), file);

        let tokens = lex(&file_data, &mut symbol_table);
        assert!(tokens.is_err());
    }
}


#[test]
fn comments() {
    let mut symbol_table = StringMap::new();
    let file = symbol_table.insert("test");
    let data = "// hello there!\n";
    let file_data = FileData::new(data.to_string(), file);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();
    compare_individually(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::EndOfFile, 
            source_range: SourceRange::new(data.len() as u32, data.len() as u32),
        }
    ].as_slice())
}

fn compare_individually<T: PartialEq + Debug>(list1: &[T], list2: &[T]) {
    assert_eq!(list1.len(), list2.len(), "list1: {list1:#?},\nlist2: {list2:#?}");
    for (index, (v1, v2)) in list1.iter().zip(list2.iter()).enumerate() {
        assert_eq!(v1, v2, "{index}");
    }
}


fn token(kind: TokenKind, start: u32, end: u32) -> Token {
    Token {
        token_kind: kind,
        source_range: SourceRange::new(start, end),
    }
}