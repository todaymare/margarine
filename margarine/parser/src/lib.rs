pub mod nodes;
pub mod errors;
pub mod dt;

use std::collections::HashMap;
use common::{source::SourceRange, string_map::{StringIndex, StringMap}};
use dt::{DataType, DataTypeKind};
use errors::Error;
use ::errors::{ParserError, ErrorId};
use lexer::{Token, TokenKind, TokenList, Keyword, Literal};
use nodes::{decl::{Attribute, AttributeValue, Decl, DeclId, EnumMapping, ExternFunction, FunctionArgument, FunctionSignature, UseItem, UseItemKind, Visibility}, expr::{Block, CallArgument, Expr, MatchMapping, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, AST};
use sti::{alloc::Alloc, arena::Arena, vec::{KVec, Vec}};

use crate::nodes::{decl::DeclGeneric, expr::{BinaryOperator, ExprId}, Pattern};

pub fn parse<'a>(
    tokens: TokenList, 
    file: u32,
    arena: &'a Arena, 
    string_map: &mut StringMap,
    ast: &mut AST<'a>,
    cfg_env: &HashMap<StringIndex, StringIndex>,
) -> (Block<'a>, KVec<u32, DeclId>, KVec<u32, DeclId>, KVec<ParserError, Error>) {

    let mut parser = Parser {
        tokens: &*tokens,
        index: 0,
        string_map,
        arena,
        errors: KVec::new(),
        is_in_panic: false,
        file,
        ast,
        cfg_env,
        imports: KVec::new(),
        link_files: KVec::new(),
        hash_attr: None,
    };


    let result = parser.parse_till_decl(
        TokenKind::EndOfFile, 
        0, 
        &ParserSettings::default()
    );

    let result = match result {
        Ok(v) => v,
        Err(e) => {
            Block::new(parser.arena.alloc_new([NodeId::Err(e)]), SourceRange::new(file, file))
        },
    };

    (result, parser.imports, parser.link_files, parser.errors)
}


// Internal
struct ParserSettings {
    is_in_impl: bool,
    can_parse_struct_creation: bool,
}


impl Default for ParserSettings {
    fn default() -> Self {
        Self {
            is_in_impl: false,
            can_parse_struct_creation: true,
        }
    }
}


struct Parser<'me, 'ast, 'str> {
    tokens: &'me [Token],
    index: usize,
    file: u32,

    arena: &'ast Arena,
    ast: &'me mut AST<'ast>,
    string_map: &'me mut StringMap<'str>,
    cfg_env: &'me HashMap<StringIndex, StringIndex>,

    imports: Vec<DeclId>,
    link_files: Vec<DeclId>,

    errors: KVec<ParserError, Error>,
    is_in_panic: bool,

    hash_attr: Option<(StringIndex, SourceRange)>,
}

type StmtResult<'ta> = Result<StmtId, ErrorId>;
type DeclResult<'ta> = Result<DeclId, ErrorId>;
type ExprResult<'a> = Result<ExprId, ErrorId>;


impl<'out> Parser<'_, 'out, '_> {
    #[inline(always)]
    fn advance(&mut self) {
        self.index += 1;
    }


    #[inline(always)]
    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }


    #[inline(always)]
    fn current_kind(&self) -> TokenKind {
        self.current().kind()
    }


    #[inline(always)]
    fn current_range(&self) -> SourceRange {
        self.current().range()
    }


    #[inline(always)]
    fn peek(&self) -> Option<&Token> {
        self.peek_n(1)
    }


    #[inline(always)]
    fn peek_n(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.index+n)
    }


    #[inline(always)]
    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|x| x.kind())
    }


    #[inline(always)]
    fn is_error_token(&mut self) -> Result<(), ErrorId> {
        if let TokenKind::Error(e) = self.current_kind() {
            return Err(ErrorId::Lexer((self.file, e)))
        }

        Ok(())
    }


    #[inline(always)]
    fn is_literal_str(&self) -> Option<StringIndex> {
        match self.current_kind() {
            TokenKind::Literal(Literal::String(v)) => Some(v),
            _ => None,
        }
    }


    #[inline(always)]
    fn expect_identifier(&mut self) -> Result<StringIndex, ErrorId> {
        self.is_error_token()?;
        match self.current_kind() {
            TokenKind::Identifier(v) => Ok(v),
            TokenKind::Underscore => Ok(StringMap::HOLE),
            _ => Err(ErrorId::Parser((self.file, self.errors.push(Error::ExpectedIdentifier {
                source: self.current_range(), 
                token: self.current_kind()
            }))))
        }
    }


    #[inline(always)]
    fn expect(&mut self, token_kind: TokenKind) -> Result<&Token, ErrorId> {
        self.is_error_token()?;
        if self.current_kind() != token_kind {
            return Err(ErrorId::Parser((self.file, self.errors.push(Error::ExpectedXFoundY {
                source: self.current_range(), 
                found: self.current_kind(), 
                expected: token_kind
            }))))
        }

        Ok(self.current())
    }


    fn expect_type(&mut self) -> Result<DataType<'out>, ErrorId> {
        let start = self.current_range().start();
        let result = if self.current_is(TokenKind::Bang) {
            DataType::new(self.current_range(), DataTypeKind::Never)

        } else if self.current_is(TokenKind::Underscore) {
            DataType::new(self.current_range(), DataTypeKind::Hole)

        } else if self.current_is(TokenKind::LeftSquare) {
            self.advance();

            let ty = self.expect_type()?;
            self.advance();

            self.expect(TokenKind::RightSquare)?;
            DataType::new(SourceRange::new(start, self.current_range().end()), DataTypeKind::List(self.arena.alloc_new(ty)))
        } else if self.current_is(TokenKind::Keyword(Keyword::Fn)) {
            self.advance();

            self.expect(TokenKind::LeftParenthesis)?;
            self.advance();

            let list = self.list(TokenKind::RightParenthesis, Some(TokenKind::Comma),
            |parser, _| {
                parser.expect_type()
            })?;

            let ret = if self.peek_is(TokenKind::Colon) {
                self.advance();
                self.advance();
                self.expect_type()?
            } else {
                DataType::new(SourceRange::new(start, self.current_range().end()), DataTypeKind::Unit)
            };
            

            DataType::new(
                SourceRange::new(start, self.current_range().end()),
                DataTypeKind::Fn(list, self.arena.alloc_new(ret))
            )

        } else if self.current_is(TokenKind::LeftParenthesis) { 
            self.advance();
            if self.current_is(TokenKind::RightParenthesis) {
                DataType::new(self.current_range(), DataTypeKind::Unit)
            } else {
                let start = self.current_range().start();

                let mut vec = Vec::new_in(self.arena);
                loop {
                    if self.current_is(TokenKind::RightParenthesis) {
                        break
                    }

                    if !vec.is_empty() {
                        self.expect(TokenKind::Comma)?;
                        self.advance();
                    }

                    
                    if self.current_is(TokenKind::RightParenthesis) {
                        break
                    }

                    let name = if matches!(self.current_kind(), TokenKind::Identifier(_)) 
                                  && self.peek_is(TokenKind::Colon) {
                        let ident = self.expect_identifier()?;
                        self.advance();

                        self.expect(TokenKind::Colon)?;
                        self.advance();

                        Some(ident)
                    } else { None };

                    let typ = self.expect_type()?;
                    vec.push((name, typ));
                    self.advance();
                }

                self.expect(TokenKind::RightParenthesis)?;

                DataType::new(
                    SourceRange::new(start, self.current_range().end()),
                    DataTypeKind::Tuple(vec.leak())
                )
            }
        } else {
            let identifier = self.expect_identifier()?;
            let result = if self.peek_is(TokenKind::DoubleColon) {
                self.advance();
                self.advance();
                DataTypeKind::Within(identifier, self.arena.alloc_new(self.expect_type()?))

            } else {
                let mut vec = Vec::new_in(self.arena);
                if self.peek_is(TokenKind::LeftAngle) {
                    self.advance();
                    self.advance();
                    loop {
                        if self.current_is(TokenKind::RightAngle) {
                            break
                        }

                        if !vec.is_empty() {
                            self.expect(TokenKind::Comma)?;
                            self.advance();
                        }

                        
                        if self.current_is(TokenKind::RightAngle) {
                            break
                        }

                        let typ = self.expect_type()?;
                        vec.push(typ);
                        self.advance();
                    }

                    self.expect(TokenKind::RightAngle)?;
                }

                DataTypeKind::CustomType(identifier, vec.leak())
            };
            
            DataType::new(
                SourceRange::new(start, self.current_range().end()), 
                result
            )

        };

        Ok(result)
    }


    fn current_is(&self, token_kind: TokenKind) -> bool {
        self.current_kind() == token_kind
    }


    fn peek_is(&self, token_kind: TokenKind) -> bool {
        self.peek_kind().map(|x| x == token_kind).unwrap_or(false)
    }


    fn list<T>(
        &mut self,
        terminator: TokenKind,
        punctuation: Option<TokenKind>,
        func: impl FnMut(&mut Self, usize) -> Result<T, ErrorId>,
    ) -> Result<&'out [T], ErrorId> {
        self.list_multi(&[terminator], punctuation, func)
    }


    fn list_multi<T>(
        &mut self,
        terminator: &[TokenKind],
        punctuation: Option<TokenKind>,
        mut func: impl FnMut(&mut Self, usize) -> Result<T, ErrorId>,
    ) -> Result<&'out [T], ErrorId> {
        let mut arguments = Vec::new_in(self.arena);


        let result : Result<(), ErrorId> = (|| {
            loop {
                if self.current_kind() == TokenKind::EndOfFile { break }
                if terminator.contains(&self.current_kind()) { break }

                if let Some(punctuation) = punctuation {
                    if !arguments.is_empty() {
                        self.expect(punctuation)?;
                        self.advance();
                    }
                    
                    // allow for trailing punctuation
                    if terminator.contains(&self.current_kind()) { break }
                }


                let result = func(self, arguments.len())?;
                self.advance();
                arguments.push(result);
            };

            Ok(())
        })();


        if let Err(e) = result {
            while !terminator.contains(&self.current_kind()) 
                  && self.current_kind() != TokenKind::EndOfFile {
                self.advance();
            }

            return Err(e);
        }

        if terminator.contains(&self.current_kind()) { return Ok(arguments.leak()) }


        Err(ErrorId::Parser((self.file, self.errors.push(Error::ExpectedXFoundYMulti {
            source: self.current_range(), 
            found: self.current_kind(), 
            expected: Vec::from_slice(terminator).leak()
        }))))
    }


    fn parse_attr(&mut self, start: u32) -> Result<Attribute<'out>, ErrorId> {
        let value = match self.current_kind() {
            TokenKind::Identifier(value) => AttributeValue::Identifier(value),
            TokenKind::Literal(value) => AttributeValue::Literal(value),
            token => return Err(ErrorId::Parser((self.file, self.errors.push(Error::ExpectedIdentifier {
                source: self.current_range(), token,
            })))),
        };

        let params: &[Attribute<'out>] = 
        if self.peek_is(TokenKind::LeftParenthesis) {
            self.advance();
            self.advance();
            let params = 
            self.list(
                TokenKind::RightParenthesis, 
                Some(TokenKind::Comma), 
                |s, _| s.parse_attr(s.current_range().start()),
            )?;

            params
        } else {
            &[]
        };

        Ok(Attribute { value, range: SourceRange::new(start, self.current_range().end()), params })
    }


    fn validate_hash_attr(&self, attr: Attribute) -> Result<StringIndex, Error> {
        let err = 
        Error::InvalidCfg {
            source: attr.range,
            expected: "hash expects a 64-character SHA-256 digest as a string literal",
        };

        if attr.params.len() != 1 {
            return Err(err);
        }

        let param = attr.params[0];
        let AttributeValue::Literal(Literal::String(s)) = param.value
        else {
            return Err(err);
        };

        let str = self.string_map.get(s);
        if str.len() != 64 || !str.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(err);
        }

        Ok(s)
    }


    fn eval_cfg(&self, attr: Attribute) -> Result<bool, Error> {
        if attr.params.len() != 1 {
            return Err(Error::InvalidCfg {
                source: attr.range,
                expected: "cfg expects exactly one predicate",
            });
        }

        self.eval_cfg_predicate(attr.params[0])
    }


    fn eval_cfg_predicate(&self, predicate: Attribute) -> Result<bool, Error> {
        let Some(name) = predicate.identifier() 
        else {
            return Err(Error::InvalidCfg {
                source: predicate.range,
                expected: "cfg predicates must be identifiers",
            });
        };

        match self.string_map.get(name) {
            "env" => {
                if predicate.params.len() != 1 
                && predicate.params.len() != 2 {
                    return Err(Error::InvalidCfg {
                        source: predicate.range,
                        expected: "env expects one or two string literals",
                    });
                }

                let key = predicate.params[0];
                let AttributeValue::Literal(Literal::String(key_name)) = key.value
                else {
                    return Err(Error::InvalidCfg {
                        source: key.range,
                        expected: "env expects a string literal variable name",
                    });
                };


                if !key.params.is_empty() {
                    return Err(Error::InvalidCfg {
                        source: key.range,
                        expected: "env arguments cannot have parameters",
                    });
                }


                let Some(&value) = self.cfg_env.get(&key_name) 
                else {
                    return Err(Error::MissingCfgEnvironment {
                        source: key.range,
                        name: key_name,
                    });
                };

                if predicate.params.len() == 1 {
                    return Ok(true);
                }

                let expected_arg = predicate.params[1];
                let AttributeValue::Literal(Literal::String(expected)) = expected_arg.value 
                else {
                    return Err(Error::InvalidCfg {
                        source: expected_arg.range,
                        expected: "env expects a string literal value",
                    });
                };

                if !expected_arg.params.is_empty() {
                    return Err(Error::InvalidCfg {
                        source: expected_arg.range,
                        expected: "env arguments cannot have parameters",
                    });
                }

                Ok(value == expected)
            },

            "not" => {
                if predicate.params.len() != 1 {
                    return Err(Error::InvalidCfg {
                        source: predicate.range,
                        expected: "not expects exactly one predicate",
                    });
                }
                Ok(!self.eval_cfg_predicate(predicate.params[0])?)
            },

            "all" => {
                if predicate.params.is_empty() {
                    return Err(Error::InvalidCfg {
                        source: predicate.range,
                        expected: "all expects at least one predicate",
                    });
                }
                let mut enabled = true;
                for child in predicate.params {
                    enabled &= self.eval_cfg_predicate(*child)?;
                }
                Ok(enabled)
            },

            "any" => {
                if predicate.params.is_empty() {
                    return Err(Error::InvalidCfg {
                        source: predicate.range,
                        expected: "any expects at least one predicate",
                    });
                }
                let mut enabled = false;
                for child in predicate.params {
                    enabled |= self.eval_cfg_predicate(*child)?;
                }
                Ok(enabled)
            },

            _ => Err(Error::InvalidCfg {
                source: predicate.range,
                expected: "expected env, not, all, or any",
            }),
        }
    }


    fn skip_cfg_item(&mut self) {
        if self.current_is(TokenKind::Keyword(Keyword::Import)) {
            // Repository imports do not require a semicolon, so consume their
            // fixed header instead of relying only on statement delimiters.
            for _ in 0..4 {
                if self.current_kind() == TokenKind::EndOfFile {
                    return;
                }
                self.advance();
            }
            if self.current_is(TokenKind::SemiColon) {
                self.advance();
            }
            return;
        }

        let mut delimiters = std::vec::Vec::new();
        loop {
            match self.current_kind() {
                TokenKind::EndOfFile => return,
                TokenKind::SemiColon if delimiters.is_empty() => {
                    self.advance();
                    return;
                },
                TokenKind::LeftParenthesis | TokenKind::LeftSquare | TokenKind::LeftBracket => {
                    delimiters.push(self.current_kind());
                    self.advance();
                },
                TokenKind::RightParenthesis | TokenKind::RightSquare | TokenKind::RightBracket => {
                    let opening = delimiters.pop();
                    if opening.is_some() {
                        self.advance();
                        if matches!(opening, Some(TokenKind::LeftBracket)) && delimiters.is_empty() {
                            return;
                        }
                    } else {
                        return;
                    }
                },
                _ => self.advance(),
            }
        }
    }


    fn parse_till(
        &mut self, 
        terminator: TokenKind, 
        start: u32,
        settings: &ParserSettings
    ) -> Result<Block<'out>, ErrorId> {

        let mut storage : Vec<NodeId, _> = Vec::with_cap_in(self.arena, 1);

        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            if self.current_kind() == terminator {
                break
            }


            if !storage.is_empty() && !self.is_in_panic 
            && !matches!(storage.last().unwrap(), NodeId::Decl(_))
            && !matches!(self.tokens[self.index-1].kind(), TokenKind::SemiColon | TokenKind::RightBracket) {
                if let Err(e) = self.expect(TokenKind::SemiColon) {
                    storage.push(NodeId::Err(e));
                    self.is_in_panic = true;
                } else {
                    storage.push(NodeId::Expr(self.ast.add_expr(Expr::Unit, self.current_range())));
                    self.advance();
                }
            }


            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            if self.current_kind() == terminator {
                break
            }


            if matches!(self.current_kind(), TokenKind::Keyword(_) | TokenKind::SemiColon) {
                self.is_in_panic = false;
            }

            

            let statement = self.statement(settings);

            match statement {
                Ok (Some(e)) => {
                    storage.push(e.into());
                },
                Ok (None) => continue,
                Err(e) => {
                    storage.push(NodeId::Err(e));

                    if self.is_in_panic {
                        if self.current_kind() == TokenKind::EndOfFile {
                            break
                        }

                        self.advance();
                        continue
                    }

                    if self.current_kind() == TokenKind::EndOfFile {
                        break
                    }

                    self.is_in_panic = true;
                },
            }

            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            self.advance();
        }

        self.expect(terminator)?;

        let end = self.current_range().end();

        Ok(Block::new(storage.leak(), SourceRange::new(start, end)))
    }


    fn parse_till_decl(
        &mut self, 
        terminator: TokenKind, 
        start: u32,
        settings: &ParserSettings
    ) -> Result<Block<'out>, ErrorId> {
        let parse_till = self.parse_till(terminator, start, settings)?;

        for node in parse_till.into_iter() {
            if !matches!(node, NodeId::Decl(_) | NodeId::Err(_)) {
                self.errors.push(Error::DeclarationOnlyBlock { source: self.ast.range(*node) });
                continue;
            };
        }

        Ok(parse_till)
    }


    fn parse_generic_usage(&mut self) -> Result<Option<&'out [DataType<'out>]>, ErrorId> {
        if !self.current_is(TokenKind::DoubleColon) {
            return Ok(None);
        }

        self.advance();

        self.expect(TokenKind::LeftAngle)?;
        self.advance();

        let list = self.list(TokenKind::RightAngle, Some(TokenKind::Comma),
        |slf, _| {
            let ident = slf.expect_type()?;
            Ok(ident)
        })?;

        Ok(Some(list))
    }

    fn generic_decl(&mut self) -> Result<&'out [DeclGeneric<'out>], ErrorId> {
        if !self.current_is(TokenKind::LeftAngle) {
            return Ok(&[]);
        }

        self.advance();
        let list = self.list(TokenKind::RightAngle, Some(TokenKind::Comma),
        |slf, _| {
            let ident = slf.expect_identifier()?;

            if !slf.peek_is(TokenKind::Colon) {
                return Ok(DeclGeneric::new(ident, &[]))
            }

            slf.advance(); // :
            slf.advance();

            let bounds = slf.list_multi(
                &[TokenKind::Comma, TokenKind::RightAngle], 
                Some(TokenKind::Plus),
                |slf, _| {
                    let x = slf.expect_type();
                    x
                }
            )?;

            if slf.current_is(TokenKind::Comma) || slf.current_is(TokenKind::RightAngle) {
                slf.index -= 1;
            }


            Ok(DeclGeneric::new(ident, bounds))
        })?;

        self.advance();

        Ok(list)
    }


    fn parse_pattern(&mut self) -> Result<Pattern<'out>, ErrorId> {
        let start = self.current_range().start();
        let mut bindings = Vec::new_in(self.arena);
        loop {
            if !bindings.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }
            

            let name = self.expect_identifier()?;
            bindings.push(name);

            if !self.peek_is(TokenKind::Comma) {
                break
            }

            self.advance();
            

        }

        if bindings.len() == 1 {
            return Ok(Pattern::new(
                SourceRange::new(start, self.current_range().end()),
                nodes::PatternKind::Variable(bindings[0])
            ))
        }


        return Ok(Pattern::new(
            SourceRange::new(start, self.current_range().end()),
            nodes::PatternKind::Tuple(bindings.leak_slice())
        ))
    }
}

impl<'ta> Parser<'_, 'ta, '_> {
    fn statement(&mut self, settings: &ParserSettings) -> Result<Option<NodeId>, ErrorId> {
        if self.current_is(TokenKind::Keyword(Keyword::Pub)) {
            self.advance();
            let Some(decl) = self.parse_decl(settings, Visibility::Public) 
            else {
                let err = self.errors.push(Error::UnexpectedToken(self.current_range()));
                return Err(ErrorId::Parser((self.file, err)));
            };

            let node = decl?.into();
            if self.peek_is(TokenKind::SemiColon) { self.advance(); }
            return Ok(Some(node));
        }

        if let Some(decl) = self.parse_decl(settings, Visibility::Private) {
            let node = decl?.into();
            if self.peek_is(TokenKind::SemiColon) { self.advance(); }
            return Ok(Some(node));
        }

        let node = match self.current_kind() {
            TokenKind::Keyword(Keyword::Var) => self.let_statement()?.into(),
            TokenKind::Keyword(Keyword::For) => self.for_statement()?.into(),

            TokenKind::SemiColon => {
                return Ok(Some(self.ast.add_expr(Expr::Unit, self.current_range()).into()))
            }

            TokenKind::At => {
                let start = self.current_range().start();
                self.advance();

                let attr = self.parse_attr(start)?;
                self.advance();

                if matches!(attr.identifier(), Some(name) if self.string_map.get(name) == "cfg") {
                    match self.eval_cfg(attr) {
                        Ok(true) => return self.statement(settings),
                        Ok(false) => {
                            self.skip_cfg_item();
                            return Ok(None);
                        },
                        Err(error) => {
                            self.errors.push(error);
                            return self.statement(settings);
                        },
                    }
                }


                if matches!(attr.identifier(), Some(name) if self.string_map.get(name) == "hash") {
                    let prev_hash = self.hash_attr.take();
                    match self.validate_hash_attr(attr) {
                        Ok(hash) => {
                            self.hash_attr = Some((hash, attr.range));
                        },
                        Err(e) => {
                            self.hash_attr = Some((StringMap::UNIT, attr.range));
                            let e = self.errors.push(e);
                            return Err(ErrorId::Parser((self.file, e)));
                        },
                    };

                    let value = self.statement(settings)?;
                    let curr = self.hash_attr;
                    self.hash_attr = prev_hash;

                    if curr.is_some() {
                        let err = self.errors.push(Error::InvalidCfg {
                            source: attr.range,
                            expected: "hash attribute can't be used here",
                        });

                        return Err(ErrorId::Parser((self.file, err)));
                    }

                    return Ok(value)
                }

                let Some(stmt) = self.statement(settings)? 
                else {
                    return Ok(None);
                };

                match stmt {
                    NodeId::Decl(decl) => self.ast.add_decl(
                        Decl::Attribute { attr, decl },
                        SourceRange::new(start, self.current_range().end()),
                    ).into(),

                    node => self.ast.add_stmt(
                        Stmt::Attribute { attr, node },
                        SourceRange::new(start, self.current_range().end()),
                    ).into(),
                }
            },

            _ => self.assignment(&settings)?.into(),
        };

        Ok(Some(node))
    }

    fn parse_decl(&mut self, settings: &ParserSettings, visibility: Visibility) -> Option<DeclResult<'ta>> {
        Some(match self.current_kind() {
            TokenKind::Keyword(Keyword::Struct) => self.struct_declaration(visibility),
            TokenKind::Keyword(Keyword::Fn) => self.function_declaration(settings, visibility),
            TokenKind::Keyword(Keyword::Trait) => self.trait_declaration(visibility),
            TokenKind::Keyword(Keyword::Mod) => self.mod_declaration(visibility),
            TokenKind::Keyword(Keyword::Extern) => self.extern_declaration(settings, visibility),
            TokenKind::Keyword(Keyword::Enum) => self.enum_declaration(visibility),
            TokenKind::Keyword(Keyword::Use) => self.using_declaration(visibility),
            TokenKind::Keyword(Keyword::Type) => self.opaque_type_declaration(visibility),
            TokenKind::Keyword(Keyword::Impl) if visibility == Visibility::Private => self.impl_declaration(),
            TokenKind::Keyword(Keyword::Import) if visibility == Visibility::Private => self.import_declaration(),
            _ => return None,
        })
    }

    fn opaque_type_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.advance();
        let name = self.expect_identifier()?;
        let gens = if self.peek_is(TokenKind::LeftAngle) {
            self.advance();
            let result = self.generic_decl()?;
            self.index -= 1;
            result
        } else { &[] };
        let range = SourceRange::new(start, self.current_range().end());
        Ok(self.ast.add_decl(Decl::OpaqueType { visibility, name, header: range, gens }, range))
    }

    fn struct_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Struct))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        let generics = self.generic_decl()?;

        let header = SourceRange::new(start, self.current_range().end());

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let fields = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, _| {
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;
            parser.advance();

            parser.expect(TokenKind::Colon)?;
            parser.advance();

            let datatype = parser.expect_type()?;
            let end = parser.current_range().end();

            Ok((name, datatype, SourceRange::new(start, end)))
        });

        let fields = fields?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        let node = Decl::Struct { visibility, name, header, fields, generics };

        Ok(self.ast.add_decl(node, SourceRange::new(start, end)))
    }



    fn function_sig(
        &mut self,
        settings: &ParserSettings,
    ) -> Result<FunctionSignature<'ta>, ErrorId> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Fn))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        let generics = self.generic_decl()?;

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let arguments = self.list(TokenKind::RightParenthesis, Some(TokenKind::Comma), |parser, index| {
            let start = parser.current_range().start();
            let is_inout = parser.current_is(TokenKind::Ampersand);
            if is_inout { parser.advance(); }
            let name = parser.expect_identifier()?;

            if index == 0
                && name == StringMap::SELF {
                if settings.is_in_impl {
                    return Ok(FunctionArgument::new(
                        name,
                        DataType::new(parser.current_range(), DataTypeKind::CustomType(StringMap::SELF_TY, &[])),
                        is_inout,
                        parser.current_range(),
                    ));
                }
            }

            parser.advance();
            

            parser.expect(TokenKind::Colon)?;
            parser.advance();

            let data_type = parser.expect_type()?;
            let end = parser.current_range().end();

            let argument = FunctionArgument::new(
                name,
                data_type,
                is_inout,
                SourceRange::new(start, end)
            );

            Ok(argument)
        })?;

        self.expect(TokenKind::RightParenthesis)?;
        let args_end = self.current_range();

        let return_type = {
            if self.peek_is(TokenKind::Colon) {
                self.advance();
                self.advance();

                let typ = self.expect_type()?;
                typ
            } else {
                DataType::new(
                    SourceRange::new(start, args_end.end()), 
                    DataTypeKind::Unit
                )
            }
        };
        
        let header = SourceRange::new(start, return_type.range().end());

        Ok(FunctionSignature::new(
             name, 
             header,
             arguments,
             generics,
             return_type,
        ))
    }


    fn function_declaration(
        &mut self, 
        settings: &ParserSettings,
        visibility: Visibility,
    ) -> DeclResult<'ta> {
        let start = self.current_range().start();
        let sig = self.function_sig(settings)?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        let body_start = self.current_range().start();
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(self.ast.add_decl(
            Decl::Function {
                visibility,
                sig,
                body,
            },

            SourceRange::new(start, end)
        ))
    }


    fn trait_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Trait))?;
        self.advance();

        let name = self.expect_identifier()?;
        let header = SourceRange::new(start, self.current_range().end());
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut set = ParserSettings::default();
        set.is_in_impl = true;
        let functions = self.list(
            TokenKind::RightBracket,
            None,
            |parser, _| {
                parser.function_sig(&set)
            }
        )?;

        Ok(self.ast.add_decl(
            Decl::Trait { visibility, header, functions, name },
            SourceRange::new(start, self.current_range().end())
        ))
    }



    fn impl_declaration(&mut self) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Impl))?;
        self.advance();

        let gens = self.generic_decl()?;

        let data_type = self.expect_type()?;

        let for_trait = 
        if self.peek_is(TokenKind::Keyword(Keyword::For)) {
            self.advance();
            self.advance();

            let ty = self.expect_type()?;
            Some(ty)
        } else {
            None
        };

        let header = SourceRange::new(start, self.current_range().end());
        self.advance();


        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let settings = ParserSettings {
            is_in_impl: true,
            ..Default::default()
        };
        
        let body = self.parse_till_decl(TokenKind::RightBracket, body_start, &settings)?;
        let end = self.current_range().end();

        let decl =
        if let Some(for_trait) = for_trait {
            Decl::ImplTrait {
                trait_name: data_type,
                data_type: for_trait,
                gens,
                body,
                header,
            }
        } else {
            Decl::Impl { 
                data_type,
                body,
                gens,
            }
        };


        Ok(self.ast.add_decl(
            decl,
            SourceRange::new(start, end),
        ))
    }


    fn mod_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Mod))?;
        self.advance();

        let name = self.expect_identifier()?;
        let header_end = self.current_range().end();

        if self.peek_is(TokenKind::LeftBracket) {
            self.advance();

            let body_start = self.current_range().start();
            self.expect(TokenKind::LeftBracket)?;
            self.advance();

            let body = self.parse_till_decl(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
            let end = self.current_range().end();

            return Ok(self.ast.add_decl(
                Decl::Module { visibility, name, body, header: SourceRange::new(start, header_end), is_root: true },
                SourceRange::new(start, end)
            ))
        }

        self.advance();
        self.expect(TokenKind::SemiColon)?;
        let decl = self.ast.add_decl(
            Decl::ImportFile { visibility, name, body: &[] },
            SourceRange::new(start, self.current_range().end())
        );

        self.imports.push(decl);
        Ok(decl)
    }


    fn import_declaration(&mut self) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Import))?;
        self.advance();

        let Some(repo) = self.is_literal_str() 
        else {
            return Err(ErrorId::Parser((self.file, self.errors.push(Error::ExpectedLiteralString {
                source: self.current_range(), 
                token: self.current_kind()
            }))))
        };

        self.advance();

        self.expect(TokenKind::Keyword(Keyword::As))?;
        self.advance();

        let alias = self.expect_identifier()?;


        let decl = self.ast.add_decl(
            Decl::ImportRepo { 
                alias,
                repo
            }, 
            SourceRange::new(start, self.current_range().end())
        );
        
        self.imports.push(decl);

        return Ok(decl);
    }



    fn extern_declaration(&mut self, settings: &ParserSettings, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        if visibility == Visibility::Public {
            let err = self.errors.push(Error::UnexpectedToken(self.current_range()));
            return Err(ErrorId::Parser((self.file, err)));
        }
        self.expect(TokenKind::Keyword(Keyword::Extern))?;
        self.advance();

        if let Some(path) = self.is_literal_str() {
            self.advance();
            self.expect(TokenKind::SemiColon)?;
            let decl = self.ast.add_decl(
                Decl::LinkFile { url: path, hash: self.hash_attr.take() },
                SourceRange::new(start, self.current_range().end()),
            );
            self.link_files.push(decl);
            return Ok(decl);
        }

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let functions = self.list(TokenKind::RightBracket, None, |parser, _| {
            let start = parser.current_range().start();
            let function_visibility = 
            if parser.current_is(TokenKind::Keyword(Keyword::Pub)) {
                parser.advance();
                Visibility::Public
            } else {
                Visibility::Private
            };
            parser.expect(TokenKind::Keyword(Keyword::Fn))?;
            parser.advance();

            let path = if let Some(path) = parser.is_literal_str() { parser.advance(); Some(path) }
            else { None };

            let name = parser.expect_identifier()?;
            let path = match path {
                Some(v) => v,
                None => name,
            };
            
            parser.advance();
            let gens = parser.generic_decl()?;

            parser.expect(TokenKind::LeftParenthesis)?;
            parser.advance();

            let arguments = parser.list(TokenKind::RightParenthesis, Some(TokenKind::Comma),
            |parser, index| {
                let start = parser.current_range().start();
                let is_inout = parser.current_is(TokenKind::Ampersand);
                if is_inout { parser.advance(); }

                let identifier = parser.expect_identifier()?;

                if index == 0
                    && identifier == StringMap::SELF {
                    if settings.is_in_impl {
                        return Ok(FunctionArgument::new(
                            identifier,
                            DataType::new(parser.current_range(), DataTypeKind::CustomType(StringMap::SELF_TY, &[])),
                            is_inout,
                            parser.current_range(),
                        ));
                    }
                }

                parser.advance();

                parser.expect(TokenKind::Colon)?;
                parser.advance();

                let data_type = parser.expect_type()?;
                let end = parser.current_range().end();
                
                Ok(FunctionArgument::new(
                    identifier, 
                    data_type, 
                    is_inout,
                    SourceRange::new(start, end)
                ))
            });

            let arguments = arguments?;


            parser.expect(TokenKind::RightParenthesis)?;


            let end;
            let return_type = 
                if parser.peek_is(TokenKind::Colon) { 
                    parser.advance();
                    parser.advance();
                    let typ = parser.expect_type()?;
                    end = parser.current_range().end();
                    typ
                }
                else {
                    end = parser.current_range().end();
                    DataType::new(
                        SourceRange::new(start, parser.current_range().end()), 
                        DataTypeKind::Unit
                    ) 
                };


            Ok(ExternFunction::new(
                function_visibility,
                name,
                path,
                gens,
                arguments,
                return_type,
                SourceRange::new(start, end)
            ))
        });
        let functions = functions?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(self.ast.add_decl(
            Decl::Extern { visibility, functions },
            SourceRange::new(start, end)
        ))
    }


    fn enum_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Enum))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();
            
        let generics = self.generic_decl()?;

        let header = SourceRange::new(start, self.current_range().end());

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mappings = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, index| {
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;

            let (data_type, is_implicit_unit) =
                if parser.peek_kind() == Some(TokenKind::Colon) {
                    parser.advance();
                    parser.advance();
                    
                    (parser.expect_type()?, false)
                }
                else {
                    (
                        DataType::new(
                            parser.current_range(),
                            DataTypeKind::Unit
                        ), 
                        true
                    ) 
                };

            let end = parser.current_range().end();
            
            let index = match index.try_into() {
                Ok(v) => v,
                Err(_) => {
                    let err = parser.errors.push(Error::TooManyEnumVariants(SourceRange::new(start, end)));
                    return Err(ErrorId::Parser((parser.file, err)));
                }
            };

            let mapping = EnumMapping::new(
                name, 
                index, 
                data_type, 
                SourceRange::new(start, end), 
                is_implicit_unit
            );

            Ok(mapping)
        });
        let mappings = mappings?;

        let end = self.current_range().end();

        Ok(self.ast.add_decl(
            Decl::Enum { visibility, name, mappings, header, generics },
            SourceRange::new(start, end)
        ))
    }


    fn using_declaration(&mut self, visibility: Visibility) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Use))?;
        self.advance();

        let item = self.parse_use_item()?;

        Ok(self.ast.add_decl(
            Decl::Using { visibility, item },
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn parse_use_item(&mut self) -> Result<UseItem<'ta>, ErrorId> {
        let start = self.current_range().start();
        let ident = self.expect_identifier()?;

        let mut func = || {
            if self.peek_is(TokenKind::DoubleColon) {
                self.advance();

                if self.peek_is(TokenKind::LeftParenthesis) {
                    self.advance();
                    self.advance();

                    let list = self.list(TokenKind::RightParenthesis, Some(TokenKind::Comma), 
                                        |parser, _| {
                                            parser.parse_use_item()
                                        })?;

                    return Ok(UseItemKind::List { list })
                }

                self.advance();
                if self.current_is(TokenKind::Star) {
                    return Ok(UseItemKind::All)
                }

                let inner = self.parse_use_item()?;
                return Ok(UseItemKind::List { 
                        list: self.arena.alloc_new([inner]) })
            }

            let name =
            if self.peek_is(TokenKind::Keyword(Keyword::As)) {
                self.advance();
                self.advance();

                self.expect_identifier()?
            } else {
                ident
            };

            Ok(UseItemKind::BringName(name))

        };

        let item = func()?;

        Ok(UseItem::new(ident, item, SourceRange::new(start, self.current_range().end())))
    }


    fn for_statement(&mut self) -> StmtResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.advance();

        let binding = self.parse_pattern()?;
        self.advance();

        self.expect(TokenKind::Keyword(Keyword::In))?;
        self.advance();

        let expr = self.expression(
            &ParserSettings { can_parse_struct_creation: false, ..Default::default() })?;
        self.advance();


        let block_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, block_start, &ParserSettings::default())?;

        Ok(self.ast.add_stmt(
            Stmt::ForLoop {
                binding,
                expr,
                body: block
            },
            SourceRange::new(start, self.current_range().end()),
        ))
    }

    fn let_statement(&mut self) -> StmtResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Var))?;
        self.advance();

        let pattern = self.parse_pattern()?;
        self.advance();

        let hint =
            if self.current_is(TokenKind::Colon) {
                self.advance();
                let typ = self.expect_type()?;
                self.advance();
                Some(typ)
            } else { None };
        
        self.expect(TokenKind::Equals)?;
        self.advance();

        let source = SourceRange::new(start, self.current_range().end());
        let rhs = self.expression(&ParserSettings::default())?;
        
        Ok(self.ast.add_stmt(
            Stmt::Variable { pat: pattern, hint, rhs },
            source
        ))
        
    }



    fn assignment(&mut self, settings: &ParserSettings) -> Result<NodeId, ErrorId> {
        fn binary_op_assignment<'la>(
            parser: &mut Parser<'_, 'la, '_>, 
            operator: BinaryOperator, 
            lhs: ExprId, 
            settings: &ParserSettings
        ) -> StmtResult<'la> {

            parser.advance();
            parser.advance();

            let rhs = parser.expression(settings)?;
            let range = SourceRange::new(parser.ast.range(lhs).start(), parser.current_range().end());

            let rhs = parser.ast.add_expr(
                Expr::BinaryOp {
                    operator, 
                    lhs, 
                    rhs,
                },
                range,
            );

            Ok(parser.ast.add_stmt(
                Stmt::UpdateValue { lhs, rhs },
                range
            ))
        }

        
        let start = self.current_range().start();
        let lhs = self.expression(&ParserSettings::default())?;


        Ok(match self.peek_kind() {
            Some(TokenKind::AddEquals) => binary_op_assignment(self, BinaryOperator::Add, lhs, settings)?.into(),
            Some(TokenKind::SubEquals) => binary_op_assignment(self, BinaryOperator::Sub, lhs, settings)?.into(),
            Some(TokenKind::MulEquals) => binary_op_assignment(self, BinaryOperator::Mul, lhs, settings)?.into(),
            Some(TokenKind::DivEquals) => binary_op_assignment(self, BinaryOperator::Div, lhs, settings)?.into(),
            Some(TokenKind::Equals) => {
                self.advance();
                self.advance();

                let rhs = self.expression(settings)?;

                self.ast.add_stmt(
                    Stmt::UpdateValue { 
                        lhs, 
                        rhs,
                    },
                    SourceRange::new(start, self.current_range().end())
                ).into()
            }
            _ => lhs.into()
        })
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn expression(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.logical_or(settings)
    }


    fn logical_or(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        let lhs = self.logical_and(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalOr) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.logical_or(settings)?;

        let range = SourceRange::new(self.ast.range(lhs).start(), self.ast.range(rhs).end());

        let body = self.ast.add_expr( 
            Expr::Literal(Literal::Bool(true)),
            range
        );

        Ok(self.ast.add_expr(
            Expr::If {
                condition: lhs,
                body,
                else_block: Some(rhs)
            },
            range
        ))
    }


    fn logical_and(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        let lhs = self.unary_not(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalAnd) {
            return Ok(lhs)
        }

        self.advance();
        self.advance();

        let rhs = self.logical_and(settings)?;

        let range = SourceRange::new(self.ast.range(lhs).start(), self.ast.range(rhs).end());

        let else_block = self.ast.add_expr(
            Expr::Literal(Literal::Bool(false)),
            range
        );

        Ok(self.ast.add_expr(
            Expr::If {
                condition: lhs,
                body: rhs,
                else_block: Some(else_block),
            },
            range
        ))
    }


    fn unary_not(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        if self.current_is(TokenKind::Bang) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.comparisson(settings)?;
            return Ok(self.ast.add_expr(
                Expr::UnaryOp { 
                    operator: UnaryOperator::Not, 
                    rhs: expr 
                },
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.comparisson(settings)
    }


    fn comparisson(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::range_expr,
            Self::range_expr,
            &[
                TokenKind::LeftAngle, TokenKind::RightAngle,
                TokenKind::GreaterEquals, TokenKind::LesserEquals,
                TokenKind::EqualsTo, TokenKind::NotEqualsTo,
            ], 
            settings,
        )
    }
    

    fn bitwise_or(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitwise_xor, 
            Self::bitwise_xor, 
            &[TokenKind::BitwiseOr], 
            settings,
        )
        
    }


    fn bitwise_xor(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitwise_and, 
            Self::bitwise_and, 
            &[TokenKind::BitwiseXor], 
            settings,
        )
        
    }


    fn bitwise_and(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitshifts, 
            Self::bitshifts, 
            &[TokenKind::Ampersand], 
            settings,
        )
        
    }
    

    fn bitshifts(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::arithmetic, 
            Self::arithmetic, 
            &[TokenKind::BitshiftLeft, TokenKind::BitshiftRight], 
            settings,
        )
        
    }
    

    fn arithmetic(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::product, 
            Self::product, 
            &[TokenKind::Plus, TokenKind::Minus], 
            settings,
        )
    }


    fn product(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.binary_operation(
            Self::unary_neg,
            Self::unary_neg,
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent], 
            settings,
        )
    }

    fn range_expr(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        let lhs = self.bitwise_or(settings)?;

        if !self.peek_is(TokenKind::DoubleDot) {
            return Ok(lhs);
        }
        self.advance();

        let is_inc = if self.peek_is(TokenKind::Equals) { self.advance(); true }
                     else { false };
        self.advance();

        let mut rhs = self.bitwise_or(settings)?;
        if is_inc {
            let range = self.ast.range(rhs);
            let r = self.ast.add_expr(
                Expr::Literal(Literal::Integer(1)),
                range,
            );

            rhs = self.ast.add_expr(Expr::BinaryOp {
                    operator: BinaryOperator::Add,
                    lhs: rhs,
                    rhs: r
                },
                range,
            );
        }

        Ok(self.ast.add_expr(
            Expr::Range { lhs, rhs, },
            SourceRange::new(self.ast.range(lhs).start(), self.ast.range(rhs).end()),
        ))
    }
    

    fn unary_neg(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        if self.current_is(TokenKind::Minus) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.as_cast(settings)?;
            return Ok(self.ast.add_expr(
                Expr::UnaryOp { 
                    operator: UnaryOperator::Neg, 
                    rhs: expr
                },
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.as_cast(settings)
    }


    fn as_cast(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        let mut expr = self.accessors(settings)?;
        while self.peek_is(TokenKind::Keyword(Keyword::As)) {
            self.advance();
            self.advance();
            let ty = self.expect_type()?;

            let nk = Expr::AsCast { lhs: expr, data_type: ty };
            expr = self.ast.add_expr(nk, SourceRange::new(self.ast.range(expr).start(), ty.range().end()));
        }

        Ok(expr)
    }


    fn accessors(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        let mut result = self.atom(settings)?;

        if self.current_is(TokenKind::SemiColon) { return Ok(result) }

        while 
            self.peek_kind() == Some(TokenKind::Dot) 
            || self.peek_kind() == Some(TokenKind::Bang)
            || self.peek_kind() == Some(TokenKind::QuestionMark)
            || self.peek_kind() == Some(TokenKind::LeftSquare)
            || self.peek_kind() == Some(TokenKind::LeftParenthesis) {

            self.advance();

            if self.current_is(TokenKind::Bang) {
                let source = SourceRange::new(self.ast.range(result).start(), self.current_range().end());
                result = self.ast.add_expr(
                    Expr::Unwrap(result),
                    source,
                );
                continue
            }

            if self.current_is(TokenKind::QuestionMark) {
                let source = SourceRange::new(self.ast.range(result).start(), self.current_range().end());
                result = self.ast.add_expr(Expr::OrReturn(result), source);
                continue
            }
            
            if self.current_is(TokenKind::LeftSquare) {
                self.advance();
                let index = self.expression(&ParserSettings::default())?;
                self.advance();

                self.expect(TokenKind::RightSquare)?;

                let source = SourceRange::new(self.ast.range(result).start(), self.current_range().end());
                result = self.ast.add_expr(Expr::IndexList { list: result, index }, source);
                continue
            }
            
            if self.current_is(TokenKind::LeftParenthesis)
                || self.peek_is(TokenKind::DoubleColon) {
                let start = self.current_range().start();
                self.advance();

                let args = self.parse_function_call_args(None)?;

                result = self.ast.add_expr(
                    Expr::CallFunction {
                        lhs: result,
                        args,
                    },
                    SourceRange::new(start, self.current_range().end())
                );

                continue;
                
            } 

            self.advance();
            
            let start = self.current_range().start();
            let ident = match self.current_kind() {
                TokenKind::Literal(Literal::Integer(int)) => self.string_map.num(int as usize),

                _ => self.expect_identifier()?,
            };


            if self.peek_kind() == Some(TokenKind::DoubleColon) {
                if self.peek_n(2).map(|t| t.kind()) == Some(TokenKind::LeftAngle) {
                    self.advance();

                    let gens = self.parse_generic_usage()?;

                    result = self.ast.add_expr(
                        Expr::AccessField { 
                            val: result, 
                            field_name: ident,
                            gens,
                        },

                        SourceRange::new(start, self.current_range().end())
                    );

                    continue;
                }
            }
            


            result = self.ast.add_expr(
                Expr::AccessField { 
                    val: result, 
                    field_name: ident,
                    gens: None,
                },

                SourceRange::new(start, self.current_range().end())
            );
        }

        Ok(result)
    }
    

    fn atom(&mut self, settings: &ParserSettings) -> ExprResult<'ta> {
        self.is_error_token()?;

        match self.current_kind() {
            TokenKind::Literal(l) => Ok(self.ast.add_expr(
                Expr::Literal(l), 
                self.current_range(),
            )),


            TokenKind::LeftParenthesis => {
                let start = self.current_range().start();
                self.advance();

                if self.current_is(TokenKind::RightParenthesis) {
                     return Ok(self.ast.add_expr(
                        Expr::Unit, 
                        self.current_range(),
                    ))       
                }

                let expr = self.expression(&ParserSettings::default())?;
                self.advance();

                if self.current_is(TokenKind::Comma) {
                    let mut vec = Vec::new_in(&*self.arena);
                    vec.push(expr);
                    while self.current_is(TokenKind::Comma) {
                        self.advance();
                        if self.current_is(TokenKind::RightParenthesis) { break }

                        vec.push(self.expression(&ParserSettings::default())?);
                        self.advance();
                    }
                    self.expect(TokenKind::RightParenthesis)?;
                    return Ok(self.ast.add_expr(
                        Expr::Tuple(vec.leak()), 
                        SourceRange::new(start, self.current_range().end())
                    ));
                }

                self.expect(TokenKind::RightParenthesis)?;

                Ok(self.ast.add_expr(
                    Expr::Paren(expr),
                    SourceRange::new(start, self.current_range().end()),
                ))

            },


            TokenKind::LeftBracket => self.block_expression(),


            TokenKind::DollarSign => {
                let start = self.current_range().start();

                self.advance();
                let ident = self.expect_identifier()?;
                let ident = self.string_map.concat_with(StringMap::DOLLAR, ident, "");

                let gens = 
                if self.peek_is(TokenKind::DoubleColon) {
                    self.advance();
                    self.parse_generic_usage()?
                } else {
                    None
                };

                Ok(self.ast.add_expr(
                    Expr::Identifier(ident, gens),
                    SourceRange::new(start, self.current_range().end())
                ))
            }


            TokenKind::Identifier(v) => {

                if settings.can_parse_struct_creation 
                    && (
                        self.peek_kind() == Some(TokenKind::LeftBracket)
                    ) {
                    return self.struct_creation_expression()
                }


                if self.peek_kind() == Some(TokenKind::DoubleColon) {
                    let source = self.current_range();
                    let start = self.current_range().start();

                    if self.peek_n(2).map(|t| t.kind()) == Some(TokenKind::LeftAngle) {
                        self.advance();

                        let gens = self.parse_generic_usage()?;

                        return Ok(self.ast.add_expr(
                            Expr::Identifier(v, gens),
                            SourceRange::new(start, self.current_range().end()),
                        ))
                    }

                    self.advance();
                    self.advance();
                    let expr = self.atom(settings)?;
                    
                    return Ok(self.ast.add_expr(
                        Expr::WithinNamespace { 
                            namespace: v,
                            action: expr,
                            namespace_source: source,
                        },
                        SourceRange::new(start, self.current_range().end())
                    ))
                }
                
                Ok(self.ast.add_expr(
                    Expr::Identifier(v, None),
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Match) => self.match_expression(),
            TokenKind::Keyword(Keyword::If) => self.if_expression(),


            TokenKind::LogicalOr => {
                let start = self.current_range().start();
                self.advance();

                let expr = self.expression(settings)?;

                Ok(self.ast.add_expr(
                    Expr::Closure { args: &[], body: expr }, 
                    SourceRange::new(start, self.current_range().end())
                ))
            }


            TokenKind::BitwiseOr => {
                let start = self.current_range().start();
                self.advance();

                let list = self.list(
                    TokenKind::BitwiseOr,
                    Some(TokenKind::Comma),
                    |parser, _| {
                        let start = parser.current_range().start();
                        let name = parser.expect_identifier()?;

                        let dt = if parser.peek_is(TokenKind::Colon) {
                            parser.advance();
                            parser.advance();
                            Some(parser.expect_type()?)
                        } else {
                            None
                        };

                        Ok((name, dt, SourceRange::new(start, parser.current_range().end())))
                    }
                )?;

                self.advance();

                let expr = self.expression(settings)?;

                Ok(self.ast.add_expr(
                    Expr::Closure { args: list, body: expr }, 
                    SourceRange::new(start, self.current_range().end())
                ))
            },


            
            
            // TokenKind::At => self.parse_with_attr(settings, Self::expression),


            TokenKind::Keyword(Keyword::Return) => {
                let start = self.current_range().start();

                self.advance();

                let expr = self.expression(&ParserSettings::default())?;
                Ok(self.ast.add_expr(
                    Expr::Return(expr), 
                    SourceRange::new(start, self.ast.range(expr).end())
                ))
            }


            TokenKind::Keyword(Keyword::Break) => {
                Ok(self.ast.add_expr(
                    Expr::Break, 
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Continue) => {
                Ok(self.ast.add_expr(
                    Expr::Continue, 
                    self.current_range(),
                ))
            },


            TokenKind::Keyword(Keyword::Loop) => {
                let start = self.current_range().start();
                self.advance();

                let body_start = self.current_range().start();
                self.expect(TokenKind::LeftBracket)?;
                self.advance();
                let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;

                Ok(self.ast.add_expr(
                    Expr::Loop { body },
                    SourceRange::new(start, self.current_range().end())
                ))
            }


            TokenKind::Keyword(Keyword::While) => {
                let start = self.current_range().start();
                self.advance();

                let expr = self.expression(&ParserSettings {
                    can_parse_struct_creation: false,
                    ..Default::default()
                })?;

                self.advance();

                let body_start = self.current_range().start();
                self.expect(TokenKind::LeftBracket)?;
                self.advance();
                let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;

                let source = SourceRange::new(start, self.current_range().end());

                let else_block = self.ast.add_expr(
                    Expr::Break,
                    source,
                );

                let body = self.ast.add_expr(Expr::Block { block: body }, body.range());
                let if_node = self.ast.add_expr(
                    Expr::If {
                        condition: expr,
                        body,
                        else_block: Some(else_block),
                    },
                    source
                );

                Ok(self.ast.add_expr(
                    Expr::Loop {
                        body: Block::new(self.arena.alloc_new([if_node.into()]), source) },
                    source,
                ))
            }


            TokenKind::LeftSquare => {
                let start = self.current_range().start();
                self.advance();

                let exprs = self.list(
                    TokenKind::RightSquare,
                    Some(TokenKind::Comma),
                    |parser, _| {
                        parser.expression(&ParserSettings::default())
                    }
                )?;


                Ok(self.ast.add_expr(
                    Expr::CreateList { exprs },
                    SourceRange::new(start, self.current_range().end()),
                ))
            }

            
            _ => Err(ErrorId::Parser((
                self.file,
                self.errors.push(Error::UnexpectedToken(self.current_range())))
            ))
        }
    }



    fn match_expression(&mut self) -> ExprResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Match))?;
        self.advance();

        let val = {
            let settings = ParserSettings {
                can_parse_struct_creation: false,
                ..Default::default()
            };

            self.expression(&settings)?
        };
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mappings = self.list(TokenKind::RightBracket, Some(TokenKind::Comma),
        |parser, _| {
            let start = parser.current_range().start();
            let name = match parser.current_kind() {
                TokenKind::Literal(Literal::Bool(true)) => StringMap::TRUE,
                TokenKind::Literal(Literal::Bool(false)) => StringMap::FALSE,
                _ => parser.expect_identifier()?,
            };
            parser.advance();

            let (bind_to, binding_range) =
                if parser.current_is(TokenKind::LeftParenthesis) {
                    parser.advance();

                    let binding_start = parser.current_range().start();

                    let name = parser.expect_identifier()?;
                    let binding_range = SourceRange::new(binding_start, parser.current_range().end());
                    parser.advance();

                    parser.expect(TokenKind::RightParenthesis)?;
                    parser.advance();
                    (name, binding_range)

                } else {
                    (parser.string_map.insert("_"), parser.current_range())
                };

            let source_range = SourceRange::new(start, parser.current_range().start());

            parser.expect(TokenKind::Arrow)?;
            parser.advance();

            let expr = parser.expression(&ParserSettings::default())?;

            Ok(MatchMapping::new(name, bind_to, binding_range, source_range, expr))
        })?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(self.ast.add_expr(
            Expr::Match { 
                value: val, 
                mappings
            },
            SourceRange::new(start, end)
        ))
    }


    fn block_expression(&mut self) -> ExprResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, start, &ParserSettings::default())?;

        Ok(self.ast.add_expr(
            Expr::Block { block },
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn if_expression(&mut self) -> ExprResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::If))?;
        self.advance();

        let settings = ParserSettings { can_parse_struct_creation: false, ..Default::default()};
        let condition = self.expression(&settings)?;
        self.advance();

        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;

        let else_block = 
            if self.peek_kind() == Some(TokenKind::Keyword(Keyword::Else)) {
                self.advance();
                self.advance();

                Some(if self.current_is(TokenKind::Keyword(Keyword::If)) {
                    self.if_expression()?
                } else {
                    self.block_expression()?
                })
            } else { None };

        
        let body = self.ast.add_expr(Expr::Block { block: body }, body.range());
        Ok(self.ast.add_expr(
            Expr::If {
                condition, 
                body,
                else_block,
            },
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn parse_function_call_args(
        &mut self, 
        associated: Option<ExprId>
    ) -> Result<&'ta mut [CallArgument], ErrorId> {

        let mut args = Vec::new_in(&*self.arena);

        if let Some(node) = associated {
            args.push(CallArgument { expr: node, is_inout: false });
        }
        
        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            if (associated.is_none() && args.len() != 0)
                || (associated.is_some() && args.len() != 1) {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            let is_inout = self.current_is(TokenKind::Ampersand);
            if is_inout { self.advance(); }
            let expr = self.expression(&ParserSettings::default())?;
            self.advance();
            
            args.push(CallArgument { expr, is_inout });
        }
        self.expect(TokenKind::RightParenthesis)?;

        Ok(args.leak_slice())
    }


    fn struct_creation_expression(&mut self) -> ExprResult<'ta> {
        let start = self.current_range().start();
        let data_type = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let fields = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, _| {
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;

            if parser.peek_is(TokenKind::Comma) || parser.peek_is(TokenKind::RightBracket) {
                return Ok((
                    name,
                    parser.current_range(),
                    parser.ast.add_expr(Expr::Identifier(name, None), parser.current_range())
                ));
            }

            parser.advance();

            parser.expect(TokenKind::Colon)?;
            parser.advance();

            let expr = parser.expression(&ParserSettings::default())?;
            let end = parser.current_range().end();
            
            Ok((name, SourceRange::new(start, end), expr))
        })?;

        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(self.ast.add_expr(
            Expr::CreateStruct { data_type, fields },
            SourceRange::new(start, end),
        ))
    }

}


impl<'ta> Parser<'_, 'ta, '_> {
    fn binary_operation(
        &mut self,
        lhs: fn(&mut Self, &ParserSettings) -> ExprResult<'ta>,
        rhs: fn(&mut Self, &ParserSettings) -> ExprResult<'ta>,
        tokens: &[TokenKind],
        settings: &ParserSettings,
    ) -> ExprResult<'ta> {
        let mut lhs = lhs(self, settings)?;

        while self.peek_kind()
                .map(|x| tokens.contains(&x))
                .unwrap_or(false) {
            self.advance();
            let operator = match self.current_kind() {
                TokenKind::Plus => BinaryOperator::Add,
                TokenKind::Minus => BinaryOperator::Sub,
                TokenKind::Star => BinaryOperator::Mul,
                TokenKind::Slash => BinaryOperator::Div,
                TokenKind::Percent => BinaryOperator::Rem,

                TokenKind::BitshiftLeft => BinaryOperator::BitshiftLeft,
                TokenKind::BitshiftRight => BinaryOperator::BitshiftRight,
                TokenKind::Ampersand => BinaryOperator::BitwiseAnd, 
                TokenKind::BitwiseOr => BinaryOperator::BitwiseOr,
                TokenKind::BitwiseXor => BinaryOperator::BitwiseXor,

                TokenKind::LeftAngle => BinaryOperator::Lt,
                TokenKind::LesserEquals => BinaryOperator::Le,
                TokenKind::RightAngle => BinaryOperator::Gt,
                TokenKind::GreaterEquals => BinaryOperator::Ge,
                TokenKind::EqualsTo => BinaryOperator::Eq,
                TokenKind::NotEqualsTo => BinaryOperator::Ne,

                _ => unreachable!(),
            };
            self.advance();

            
            let rhs = rhs(self, settings)?;

            let range = SourceRange::new(
                self.ast.range(lhs).start(), 
                self.ast.range(rhs).end(),
            );

            lhs = self.ast.add_expr(
                Expr::BinaryOp { operator, lhs, rhs }, 
                range,
            )
        }
        
        Ok(lhs)
    }

}


#[cfg(test)]
mod tests {
    use super::*;
    use sti::arena::Arena;
    use common::{source::{FileData, Extension}, string_map::StringMap};
    use lexer::lex;

    #[test]
    fn generic_type_argument() {
        let arena = Arena::new();
        let mut sm = StringMap::new(&arena);
        let file_name = sm.insert("test");
        let file = FileData::new("fn f(x: Foo<T>) {}".to_string(), file_name, Extension::None);
        let (tokens, _) = lex(&file, &mut sm, 0);
        let mut ast = AST::new(&arena);
        let cfg_env = std::collections::HashMap::new();
        let (_, _, _, errors) = parse(tokens, 0, &arena, &mut sm, &mut ast, &cfg_env);
        assert!(errors.is_empty(), "parse errors: {errors:?}");
    }

    #[test]
    fn binary_operator_display() {
        assert_eq!(format!("{}", BinaryOperator::BitshiftLeft), "<<");
        assert_eq!(format!("{}", BinaryOperator::BitshiftRight), ">>");
    }


    #[test]
    fn match_bindings_use_parentheses() {
        let arena = Arena::new();
        let mut sm = StringMap::new(&arena);
        let file_name = sm.insert("test");
        let file = FileData::new(
            "fn f(x: Option<int>) { match x { some(value) => value, none => 0, } }".to_string(),
            file_name,
            Extension::None,
        );
        let (tokens, _) = lex(&file, &mut sm, 0);
        let mut ast = AST::new(&arena);
        let cfg_env = std::collections::HashMap::new();
        let (_, _, _, errors) = parse(tokens, 0, &arena, &mut sm, &mut ast, &cfg_env);
        assert!(errors.is_empty(), "parse errors: {errors:?}");
    }


    #[test]
    fn disabled_cfg_items_are_omitted_before_import_collection() {
        let arena = Arena::new();
        let mut sm = StringMap::new(&arena);
        let file_name = sm.insert("test");
        let file = FileData::new(
            "@cfg(env(\"DISABLED\", \"no\")) import \"missing\" as missing;\n\
             @cfg(env(\"DISABLED\", \"no\")) extern \"missing.o\";\n\
             @cfg(env(\"DISABLED\", \"no\")) fn bad() { this is not valid; }\n\
             fn enabled() {}"
                .to_string(),
            file_name,
            Extension::None,
        );
        let (tokens, _) = lex(&file, &mut sm, 0);
        let mut ast = AST::new(&arena);
        let mut cfg_env = std::collections::HashMap::new();
        cfg_env.insert(sm.insert("DISABLED"), sm.insert("yes"));

        let (body, imports, links, errors) = parse(
            tokens, 0, &arena, &mut sm, &mut ast, &cfg_env,
        );

        assert!(errors.is_empty(), "parse errors: {errors:?}");
        assert!(imports.is_empty());
        assert!(links.is_empty());
        assert_eq!(body.len(), 1);
        assert!(matches!(ast.decl(match body[0] {
            NodeId::Decl(id) => id,
            _ => panic!("expected a declaration"),
        }), Decl::Function { .. }));
    }

    #[test]
    fn hash_attribute_requires_64_hexadecimal_characters() {
        for hash in [
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcde",
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdeg",
        ] {
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let file_name = sm.insert("test");
            let file = FileData::new(
                format!("@hash(\"{hash}\") extern \"missing.o\";"),
                file_name,
                Extension::None,
            );
            let (tokens, _) = lex(&file, &mut sm, 0);
            let mut ast = AST::new(&arena);
            let cfg_env = std::collections::HashMap::new();
            let (_, _, _, errors) = parse(tokens, 0, &arena, &mut sm, &mut ast, &cfg_env);

            assert!(matches!(errors.first(), Some(Error::InvalidCfg { .. })));
        }
    }
}
