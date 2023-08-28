
#![allow(unused)]
use std::fmt::Debug;

use common::{SymbolMap, SourceRange, FileData, SymbolIndex, Slice, HashableF64};

use crate::{lex, Token, TokenKind, Literal};


#[test]
fn empty() {
    let mut symbol_table = SymbolMap::new();
    let file_name = symbol_table.insert("test");
    let file_data = FileData::new(String::new(), file_name);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();

    assert_eq!(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::EndOfFile,
            source_range: SourceRange::new(0, 0, file_name),
        }
    ].as_slice())
}


#[test]
fn tokens() {
    {
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "() <> {} [] % / + - * ^ : :: , . ! = _ \
                    <= >= == != || && += -= *= /= @ ?";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(&tokens, &vec![
            token(TokenKind::LeftParenthesis, 0, 0, file),
            token(TokenKind::RightParenthesis, 1, 1, file),

            token(TokenKind::LeftAngle, 3, 3, file),
            token(TokenKind::RightAngle, 4, 4, file),

            token(TokenKind::LeftBracket, 6, 6, file),
            token(TokenKind::RightBracket, 7, 7, file),

            token(TokenKind::LeftSquare, 9, 9, file),
            token(TokenKind::RightSquare, 10, 10, file),

            token(TokenKind::Percent, 12, 12, file),
            token(TokenKind::Slash, 14, 14, file),
            token(TokenKind::Plus, 16, 16, file),
            token(TokenKind::Minus, 18, 18, file),
            token(TokenKind::Star, 20, 20, file),
            token(TokenKind::BitwiseXor, 22, 22, file),
            token(TokenKind::Colon, 24, 24, file),
            token(TokenKind::DoubleColon, 26, 27, file),
            token(TokenKind::Comma, 29, 29, file),
            token(TokenKind::Dot, 31, 31, file),
            token(TokenKind::Bang, 33, 33, file),
            token(TokenKind::Equals, 35, 35, file),
            token(TokenKind::Underscore, 37, 37, file),

            token(TokenKind::LesserEquals, 39, 40, file),
            token(TokenKind::GreaterEquals, 42, 43, file),
            token(TokenKind::EqualsTo, 45, 46, file),
            token(TokenKind::NotEqualsTo, 48, 49, file),
            token(TokenKind::LogicalOr, 51, 52, file),
            token(TokenKind::LogicalAnd, 54, 55, file),

            token(TokenKind::AddEquals, 57, 58, file),
            token(TokenKind::SubEquals, 60, 61, file),
            token(TokenKind::MulEquals, 63, 64, file),
            token(TokenKind::DivEquals, 66, 67, file),

            token(TokenKind::At, 69, 69, file),
            token(TokenKind::QuestionMark, 71, 71, file),

            token(TokenKind::EndOfFile, 71, 71, file)
        ])
    }

    // Invalid character
    {
        let mut symbol_table = SymbolMap::new();
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
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "123456789";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::Integer(123456789)),
                source_range: SourceRange::new(0, 8, file),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(8, 8, file),
            }
        ].as_slice());
    }


    // valid float
    {
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "420.69";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::Float(HashableF64(420.69))),
                source_range: SourceRange::new(0, 5, file),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(5, 5, file),
            }
        ].as_slice());
    }


    // too many dots
    {
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "420.69.50";
        let file_data = FileData::new(data.to_string(), file);
    
        let tokens = lex(&file_data, &mut symbol_table);
        assert!(tokens.is_err())
    }
}


#[test]
fn identifiers() {
    let mut symbol_table = SymbolMap::new();
    let file = symbol_table.insert("test");
    let data = "hello _there";
    let file_data = FileData::new(data.to_string(), file);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();

    compare_individually(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::Identifier(symbol_table.insert("hello")),
            source_range: SourceRange::new(0, 4, file),
        },
        Token {
            token_kind: TokenKind::Identifier(symbol_table.insert("_there")),
            source_range: SourceRange::new(6, 11, file),
        },
        Token {
            token_kind: TokenKind::EndOfFile,
            source_range: SourceRange::new(11, 11, file),
        },
    ].as_slice())
}


#[test]
fn string() {
    // valid string
    {
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "\"hello there\"";
        let file_data = FileData::new(data.to_string(), file);

        let tokens = lex(&file_data, &mut symbol_table).unwrap();

        compare_individually(tokens.as_slice(), vec![
            Token {
                token_kind: TokenKind::Literal(Literal::String(symbol_table.insert("hello there"))),
                source_range: SourceRange::new(0, 12, file),
            },
            Token {
                token_kind: TokenKind::EndOfFile,
                source_range: SourceRange::new(12, 12, file),
            },
        ].as_slice())
    }


    // unterminated string
    {
        let mut symbol_table = SymbolMap::new();
        let file = symbol_table.insert("test");
        let data = "\"hello there";
        let file_data = FileData::new(data.to_string(), file);

        let tokens = lex(&file_data, &mut symbol_table);
        assert!(tokens.is_err());
    }
}


#[test]
fn comments() {
    let mut symbol_table = SymbolMap::new();
    let file = symbol_table.insert("test");
    let data = "// hello there!\n";
    let file_data = FileData::new(data.to_string(), file);

    let tokens = lex(&file_data, &mut symbol_table).unwrap();
    compare_individually(tokens.as_slice(), vec![
        Token {
            token_kind: TokenKind::EndOfFile, 
            source_range: SourceRange::new(15, 15, file),
        }
    ].as_slice())
}

fn compare_individually<T: PartialEq + Debug>(list1: &[T], list2: &[T]) {
    assert_eq!(list1.len(), list2.len(), "list1: {list1:#?},\nlist2: {list2:#?}");
    for (index, (v1, v2)) in list1.iter().zip(list2.iter()).enumerate() {
        assert_eq!(v1, v2, "{index}");
    }
}


fn token(kind: TokenKind, start: usize, end: usize, file: SymbolIndex) -> Token {
    Token {
        token_kind: kind,
        source_range: SourceRange::new(start, end, file),
    }
}