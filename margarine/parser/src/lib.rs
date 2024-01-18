pub mod nodes;
pub mod errors;

use std::{ops::Deref, hash::Hash};

use common::{source::SourceRange, string_map::{StringMap, StringIndex}};
use errors::Error;
use ::errors::{ParserError, ErrorId};
use lexer::{Token, TokenKind, TokenList, Keyword, Literal};
use nodes::{Node, StructKind, NodeKind, Declaration, FunctionArgument,
    ExternFunction, Expression, BinaryOperator, Statement, EnumMapping, UseItem, UseItemKind, Attribute};
use sti::{prelude::{Vec, Arena}, arena_pool::ArenaPool, keyed::KVec, format_in, alloc::Alloc};

use crate::nodes::MatchMapping;


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct DataType<'a> {
    source_range: SourceRange,
    kind: DataTypeKind<'a>, 
}


impl<'a> DataType<'a> {
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn kind(&self) -> DataTypeKind<'a> { self.kind }
}


impl<'a> DataType<'a> {
    pub fn new(source_range: SourceRange, kind: DataTypeKind<'a>) -> Self { 
        Self { source_range, kind } 
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataTypeKind<'a> {
    Int,
    Bool,
    Float,
    Unit,
    Any,
    Never,
    Option(&'a DataType<'a>),
    Result(&'a DataType<'a>, &'a DataType<'a>),
    Tuple(&'a [DataType<'a>]),
    Within(StringIndex, &'a DataType<'a>),
    Rc(&'a DataType<'a>),
    CustomType(StringIndex),
}


impl<'a> DataTypeKind<'a> {
    /// Coersions happen from self to oth
    /// not the other way around
    pub fn is(&self, oth: &DataTypeKind<'a>) -> bool {
        self == &DataTypeKind::Never
        || oth == &DataTypeKind::Any
        || self == oth
    }
}


impl std::hash::Hash for DataTypeKind<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DataTypeKind::Int => 0.hash(state),
            DataTypeKind::Bool => 1.hash(state),
            DataTypeKind::Float => 2.hash(state),
            DataTypeKind::Unit => 3.hash(state),
            DataTypeKind::Any => 4.hash(state),
            DataTypeKind::Never => 6.hash(state),
            DataTypeKind::Option(v) => {
                7.hash(state);
                v.kind().hash(state)
            },
            
            DataTypeKind::Result(v1, v2) => {
                8.hash(state);
                v1.kind().hash(state);
                v2.kind().hash(state);
            },

            DataTypeKind::CustomType(v) => {
                9.hash(state);
                v.hash(state)
            },

            DataTypeKind::Tuple(v) => {
                10.hash(state);
                v.len().hash(state);
                v.iter().for_each(|x| x.kind().hash(state));
            },

            DataTypeKind::Within(name, dt) => {
                11.hash(state);
                name.hash(state);
                dt.kind().hash(state);
            },

            DataTypeKind::Rc(v) => {
                12.hash(state);
                v.kind().hash(state);
            }
        }
    }
}


/// A wrapper type for `Vec<Node>` which
/// comes with the guarantee that the vec
/// isn't empty
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Block<'a> {
    nodes: &'a [Node<'a>],
    source_range: SourceRange,
}


impl<'a> Block<'a> {
    /// # Panics
    /// if the given vec is empty
    pub fn new(nodes: &'a mut [Node<'a>], source_range: SourceRange) -> Self {
        assert!(!nodes.is_empty());
        Self {
            nodes,
            source_range,
        }
    }


    pub fn range(&self) -> SourceRange {
        self.source_range
    }
}


impl<'a> Deref for Block<'a> {
    type Target = [Node<'a>];

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}


pub fn parse<'a>(
    tokens: TokenList, 
    arena: &'a mut Arena, 
    string_map: &mut StringMap
) -> (Block<'a>, KVec<ParserError, Error>) {

    let mut parser = Parser {
        tokens: &*tokens,
        index: 0,
        string_map,
        arena,
        errors: KVec::new(),
        is_in_panic: false,
    };


    let result = parser.parse_till(
        TokenKind::EndOfFile, 
        0, 
        &ParserSettings::default()
    ).unwrap();

    (result, parser.errors)
}


// Internal
type ParseResult<'a> = Result<Node<'a>, ErrorId>;

struct ParserSettings<'a> {
    is_in_impl: Option<DataType<'a>>,
    can_parse_struct_creation: bool,
}


impl Default for ParserSettings<'_> {
    fn default() -> Self {
        Self {
            is_in_impl: None,
            can_parse_struct_creation: true,
        }
    }
}


struct Parser<'a, 'ta, 'sa> {
    tokens: &'a [Token],
    index: usize,

    arena: &'ta Arena,
    string_map: &'a mut StringMap<'sa>,

    errors: KVec<ParserError, Error>,
    is_in_panic: bool,
}


impl<'ta> Parser<'_, 'ta, '_> {
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
        self.tokens.get(self.index+1)
    }


    #[inline(always)]
    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|x| x.kind())
    }


    #[inline(always)]
    fn is_error_token(&mut self) -> Result<(), ErrorId> {
        if let TokenKind::Error(e) = self.current_kind() {
            return Err(ErrorId::Lexer(e))
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
    fn expect_literal_str(&mut self) -> Result<StringIndex, ErrorId> {
        self.is_error_token()?;
        match self.current_kind() {
            TokenKind::Literal(Literal::String(v)) => Ok(v),
            _ => Err(ErrorId::Parser(self.errors.push(Error::ExpectedLiteralString { 
                    source: self.current_range(), 
                    token: self.current_kind()
                })))
        }
    }


    #[inline(always)]
    fn expect_identifier(&mut self) -> Result<StringIndex, ErrorId> {
        self.is_error_token()?;
        match self.current_kind() {
            TokenKind::Identifier(v) => Ok(v),
            _ => Err(ErrorId::Parser(self.errors.push(Error::ExpectedIdentifier {
                source: self.current_range(), 
                token: self.current_kind()
            })))
        }
    }


    #[inline(always)]
    fn expect(&mut self, token_kind: TokenKind) -> Result<&Token, ErrorId> {
        self.is_error_token()?;
        if self.current_kind() != token_kind {
            return Err(ErrorId::Parser(self.errors.push(Error::ExpectedXFoundY {
                source: self.current_range(), 
                found: self.current_kind(), 
                expected: token_kind
            })))
        }

        Ok(self.current())
    }


    #[inline(always)]
    fn expect_multi(&mut self, token_kinds: &'static [TokenKind]) -> Result<&Token, ErrorId> {
        self.is_error_token()?;
        if !token_kinds.contains(&self.current_kind()) {
            return Err(ErrorId::Parser(self.errors.push(Error::ExpectedXFoundYMulti { 
                            source: self.current_range(), 
                            found: self.current_kind(),
                            expected: token_kinds,
                        })));
        }

        Ok(self.current())
    }
    

    fn expect_type(&mut self) -> Result<DataType<'ta>, ErrorId> {
        let start = self.current_range().start();
        let mut result = 
        if self.current_is(TokenKind::Bang) {
            DataType::new(self.current_range(), DataTypeKind::Never)
        } else if self.current_is(TokenKind::LeftParenthesis) { 
            self.advance();
            if self.current_is(TokenKind::RightParenthesis) {
                DataType::new(self.current_range(), DataTypeKind::Unit)
            } else {
                let start = self.current_range().start();
                let pool = ArenaPool::tls_get_rec();
                let mut vec = Vec::new_in(&*pool);

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

                    let typ = self.expect_type()?;
                    vec.push(typ);
                    self.advance();
                }

                self.expect(TokenKind::RightParenthesis)?;

                if vec.len() == 1 {
                    vec[0]
                } else {
                    DataType::new(
                        SourceRange::new(start, self.current_range().end()),
                        DataTypeKind::Tuple(vec.move_into(self.arena).leak())
                    )
                }
            }
        } else if self.current_is(TokenKind::Star) {
            self.advance();
            let ty = self.expect_type()?;
            DataType::new(
                SourceRange::new(start, ty.range().end()),
                DataTypeKind::Rc(self.arena.alloc_new(ty)),
            )
        } else {
            let identifier = self.expect_identifier()?;
            let result = if self.peek_is(TokenKind::DoubleColon) {
                self.advance();
                self.advance();
                DataTypeKind::Within(identifier, self.arena.alloc_new(self.expect_type()?))
            } else {
                match identifier {
                    StringMap::INT   => DataTypeKind::Int,
                    StringMap::FLOAT => DataTypeKind::Float,
                    StringMap::BOOL  => DataTypeKind::Bool,
                    StringMap::ANY   => DataTypeKind::Any,
                    _ => DataTypeKind::CustomType(identifier),
                }
            };

            DataType::new(
                SourceRange::new(start, self.current_range().end()), 
                result
            )
        };
        

        loop {
            let mut has_updated = false;
            if self.peek_is(TokenKind::QuestionMark) {
                self.advance();
                let end = self.current_range().end();
                let option_result = DataTypeKind::Option(self.arena.alloc_new(result));
                let option_result = DataType::new(
                    SourceRange::new(start, end),
                    option_result,
                );

                result = option_result;
                has_updated = true;
            }

            if self.peek_is(TokenKind::SquigglyDash) {
                self.advance();
                self.advance();

                let oth_typ = self.expect_type()?;

                let range = SourceRange::new(result.range().start(), oth_typ.range().end());

                let new_result = DataTypeKind::Result(
                    self.arena.alloc_new(result), 
                    self.arena.alloc_new(oth_typ)
                );

                let new_result = DataType::new(
                    range,
                    new_result,
                );

                result = new_result;
                has_updated = true;
            }

            if !has_updated {
                break
            }
        }

        Ok(result)
    }


    fn parse_with_tag(
        &mut self, 
        settings: &ParserSettings<'ta>,
        func: fn(&mut Self, &ParserSettings<'ta>) -> ParseResult<'ta>,
    ) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::At)?;
        self.advance();

        let attr = self.parse_attribute()?;
        self.advance();
        let func = func(self, settings)?;
        let func = self.arena.alloc_new(func);


        Ok(Node::new(NodeKind::Attribute(attr, func), 
                SourceRange::new(start, self.current_range().end())))
    }


    fn parse_attribute(&mut self) -> Result<Attribute<'ta>, ErrorId> {
        let start = self.current_range().start();
        let ident = self.expect_identifier()?;

        let mut func = || -> Result<&'ta [Attribute<'ta>], ErrorId> {
            if self.peek_is(TokenKind::LeftParenthesis) {
                self.advance();
                self.advance();

                let pool = ArenaPool::tls_get_rec();
                let mut vec = Vec::new_in(&*pool);

                loop {
                    if self.current_is(TokenKind::RightParenthesis) { break }

                    if !vec.is_empty() {
                        self.expect(TokenKind::Comma)?;
                        self.advance();
                    }

                    if self.current_is(TokenKind::RightParenthesis) { break }

                    let item = self.parse_attribute()?;
                    vec.push(item);
                    self.advance();
                }

                return Ok(vec.move_into(self.arena).leak())
            }

            Ok(&[])
        };

        let item = func()?;

        Ok(Attribute::new(ident, item, SourceRange::new(start, self.current_range().end())))
    }


    fn current_is(&self, token_kind: TokenKind) -> bool {
        self.current_kind() == token_kind
    }


    fn peek_is(&self, token_kind: TokenKind) -> bool {
        self.peek_kind().map(|x| x == token_kind).unwrap_or(false)
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn parse_till(
        &mut self, 
        terminator: TokenKind, 
        start: u32,
        settings: &ParserSettings<'ta>
    ) -> Result<Block<'ta>, ErrorId> {

        let mut storage = Vec::with_cap_in(self.arena, 1);

        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            if self.current_kind() == terminator {
                break
            }


            if matches!(self.current_kind(), TokenKind::Keyword(_)) {
                self.is_in_panic = false;
            }
            

            let statement = self.statement(settings);

            match statement {
                Ok (e) => storage.push(e),
                Err(e) => {
                    storage.push(Node::new(
                        NodeKind::Error(e), 
                        SourceRange::new(start, self.current_range().end())
                    ));

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

        if storage.is_empty() {
            storage.push(Node::new(
                NodeKind::Expression(Expression::Unit),
                SourceRange::new(start, end)
            ));
        }

        Ok(Block::new(storage.leak(), SourceRange::new(start, end)))
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn statement(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        match self.current_kind() {
            | TokenKind::Keyword(Keyword::Struct)
            | TokenKind::Keyword(Keyword::Resource)
            | TokenKind::Keyword(Keyword::Component)
            => self.struct_declaration(),


            TokenKind::Keyword(Keyword::Fn) => self.function_declaration(false, &settings),
            TokenKind::Keyword(Keyword::System) => {
                self.advance();
                self.function_declaration(true, &settings)
            },


            TokenKind::Keyword(Keyword::Impl) => self.impl_declaration(),
            TokenKind::Keyword(Keyword::Mod) => self.mod_declaration(),
            TokenKind::Keyword(Keyword::Extern) => self.extern_declaration(),
            TokenKind::Keyword(Keyword::Enum) => self.enum_declaration(),
            TokenKind::Keyword(Keyword::Use) => self.using_declaration(),


            TokenKind::Keyword(Keyword::Let) => self.let_statement(),


            TokenKind::At => self.parse_with_tag(&settings, Self::statement),


            _ => self.assignment(&settings),

        }
    }


    fn struct_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        let kind = match self.current_kind() {
            TokenKind::Keyword(Keyword::Struct) => StructKind::Normal,
            TokenKind::Keyword(Keyword::Resource) => StructKind::Resource,
            TokenKind::Keyword(Keyword::Component) => StructKind::Component,

            _ => {
                self.expect_multi(&[
                    TokenKind::Keyword(Keyword::Struct),
                    TokenKind::Keyword(Keyword::Resource),
                    TokenKind::Keyword(Keyword::Component),
                ])?;
                
                unreachable!();
            }
        };
        self.advance();

        let name = self.expect_identifier()?;

        let header = SourceRange::new(start, self.current_range().end());
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut fields = Vec::new_in(self.arena);
        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            if !fields.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            let start = self.current_range().start();
            let name = self.expect_identifier()?;
            self.advance();

            self.expect(TokenKind::Colon)?;
            self.advance();

            let datatype = self.expect_type()?;
            let end = self.current_range().end();
            self.advance();

            fields.push((name, datatype, SourceRange::new(start, end)));
        }
        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        let node = Declaration::Struct { kind, name, header, fields: fields.leak() };

        Ok(Node::new(
            NodeKind::Declaration(node), 
            SourceRange::new(start, end)
        ))
    }



    fn function_declaration(
        &mut self, 
        is_system: bool, 
        settings: &ParserSettings<'ta>
    ) -> ParseResult<'ta> {

        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Fn))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let mut arguments = Vec::new_in(self.arena);
        loop {            
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            if !arguments.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            let is_inout = {
                if self.current_is(TokenKind::Ampersand) {
                    self.advance();
                    true
                } else {
                    false
                }
            };
            
            let start = self.current_range().start();
            let name = self.expect_identifier()?;
            self.advance();


            {
                if arguments.is_empty()
                    && self.string_map.get(name) == "self" {
                    if let Some(self_type) = settings.is_in_impl {
                        arguments.push(FunctionArgument::new(
                            name,
                            self_type,
                            is_inout,
                            self.current_range(),
                        ));

                        continue
                    }
                }
            }
            

            self.expect(TokenKind::Colon)?;
            self.advance();

            let data_type = self.expect_type()?;
            let end = self.current_range().end();
            self.advance();
            
            let argument = FunctionArgument::new(
                name,
                data_type,
                is_inout,
                SourceRange::new(start, end)
            );

            arguments.push(argument);
        }
        let arguments = arguments;

        self.expect(TokenKind::RightParenthesis)?;
        let args_end = self.current_range();
        self.advance();

        let return_type = {
            if self.current_is(TokenKind::Colon) {
                self.advance();

                let typ = self.expect_type()?;
                self.advance();
                typ
            } else {
                DataType::new(
                    SourceRange::new(start, args_end.end()), 
                    DataTypeKind::Unit
                )
            }
        };
        

        let header = SourceRange::new(start, return_type.range().end());
        self.expect(TokenKind::LeftBracket)?;
        let body_start = self.current_range().start();
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Function {
                is_system, 
                name,
                arguments: arguments.leak(), 
                return_type, 
                body,
                header,
            }),

            SourceRange::new(start, end)
        ))
    }


    fn impl_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Impl))?;
        self.advance();

        let data_type = self.expect_type()?;
        self.advance();

        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let settings = ParserSettings {
            is_in_impl: Some(data_type),
            ..Default::default()
        };
        
        let body = self.parse_till(TokenKind::RightBracket, body_start, &settings)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Impl { 
                data_type, body
            }),

            SourceRange::new(start, end),
        ))
    }


    fn mod_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Mod))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Module { name, body }),
            SourceRange::new(start, end)
        ))
    }


    fn extern_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Extern))?;
        self.advance();

        let file = self.expect_literal_str()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut functions = Vec::new_in(self.arena);
        loop {            
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            let start = self.current_range().start();
            self.expect(TokenKind::Keyword(Keyword::Fn))?;
            self.advance();

            let name = self.expect_identifier()?;
            self.advance();

            let path = if let Some(path) = self.is_literal_str() { self.advance(); path }
            else { name };

            self.expect(TokenKind::LeftParenthesis)?;
            self.advance();

            let mut arguments = Vec::new_in(self.arena);
            loop {
                if self.current_kind() == TokenKind::EndOfFile {
                    break
                }


                if self.current_kind() == TokenKind::RightParenthesis {
                    break
                }


                if !arguments.is_empty() {
                    self.expect(TokenKind::Comma)?;
                    self.advance();
                }


                if self.current_kind() == TokenKind::RightParenthesis {
                    break
                }

                let start = self.current_range().start();
                let is_inout = if self.current_is(TokenKind::Colon) {
                    self.advance();
                    true
                } else { false };

                let identifier = self.expect_identifier()?;
                self.advance();

                self.expect(TokenKind::Colon)?;
                self.advance();

                let data_type = self.expect_type()?;
                let end = self.current_range().end();
                self.advance();
                
                arguments.push(FunctionArgument::new(
                    identifier, 
                    data_type, 
                    is_inout, 
                    SourceRange::new(start, end)
                ));

            }
            let arguments = arguments;


            self.expect(TokenKind::RightParenthesis)?;
            self.advance();


            let end;
            let return_type = 
                if self.current_is(TokenKind::Colon) { 
                    self.advance();
                    let typ = self.expect_type()?;
                    end = self.current_range().end();
                    self.advance();
                    typ
                }
                else {
                    end = self.current_range().end();
                    DataType::new(
                        SourceRange::new(start, self.current_range().end()), 
                        DataTypeKind::Unit
                    ) 
                };


            functions.push(ExternFunction::new(
                name,
                path,
                arguments.leak(),
                return_type,
                SourceRange::new(start, end)
            ));
        }
        let functions = functions;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Extern { file, functions: functions.leak() }),
            SourceRange::new(start, end)
        ))
    }


    fn enum_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Enum))?;
        self.advance();

        let name = self.expect_identifier()?;
        let header = SourceRange::new(start, self.current_range().end());
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut mappings = Vec::new_in(self.arena);
        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            if !mappings.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            let start = self.current_range().start();
            let name = self.expect_identifier()?;

            let (data_type, is_implicit_unit) =
                if self.peek_kind() == Some(TokenKind::Colon) {
                    self.advance();
                    self.advance();
                    
                    (self.expect_type()?, false)
                }
                else {
                    (
                        DataType::new(
                            self.current_range(),
                            DataTypeKind::Unit
                        ), 
                        true
                    ) 
                };

            let end = self.current_range().end();
            self.advance();
            
            let mapping = EnumMapping::new(
                name, 
                mappings.len().try_into().unwrap(), 
                data_type, 
                SourceRange::new(start, end), 
                is_implicit_unit
            );

            mappings.push(mapping);
        }
        let mappings = mappings;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Enum { name, mappings: mappings.leak(), header }),
            SourceRange::new(start, end)
        ))
    }


    fn using_declaration(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Use))?;
        self.advance();

        let item = self.parse_use_item()?;

        Ok(Node::new(
                NodeKind::Declaration(Declaration::Using { item }),
                SourceRange::new(start, self.current_range().end())
        ))
    }


    fn parse_use_item(&mut self) -> Result<UseItem<'ta>, ErrorId> {
        let start = self.current_range().start();
        let ident = self.expect_identifier()?;

        let mut func = || {
            if self.peek_is(TokenKind::Slash) {
                self.advance();
                self.advance();
                if self.current_is(TokenKind::Star) {
                    return Ok(UseItemKind::All)
                }

                let inner = self.parse_use_item()?;
                return Ok(UseItemKind::List { 
                        list: self.arena.alloc_new([inner]) })
            }


            if self.peek_is(TokenKind::LeftParenthesis) {
                self.advance();
                self.advance();

                let pool = ArenaPool::tls_get_rec();
                let mut vec = Vec::new_in(&*pool);

                loop {
                    if self.current_is(TokenKind::RightParenthesis) { break }

                    if !vec.is_empty() {
                        self.expect(TokenKind::Comma)?;
                        self.advance();
                    }

                    if self.current_is(TokenKind::RightParenthesis) { break }

                    let item = self.parse_use_item()?;
                    vec.push(item);
                    self.advance();
                }

                return Ok(UseItemKind::List { 
                    list: vec.move_into(self.arena).leak() })

            }

            Ok(UseItemKind::BringName)

        };

        let item = func()?;

        Ok(UseItem::new(ident, item, SourceRange::new(start, self.current_range().end())))
    }


    fn let_statement(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Let))?;
        self.advance();

        let pool = ArenaPool::tls_get_temp();
        let mut bindings = Vec::new_in(&*pool);
        loop {
            if !bindings.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            let is_mut = 
                if self.current_is(TokenKind::Keyword(Keyword::Mut)) {
                    self.advance();
                    true
                } else { false };

            
            let name = self.expect_identifier()?;
            self.advance();
            
            bindings.push((name, is_mut));
            if self.current_is(TokenKind::Equals) || self.current_is(TokenKind::Colon) {
                break
            }
        }

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
        let rhs = self.arena.alloc_new(rhs);
        
        Ok(Node::new(NodeKind::Statement(if bindings.len() == 1 {
            let b = bindings[0];
            Statement::Variable { name: b.0, hint, is_mut: b.1, rhs }
        } else {
            Statement::VariableTuple {
                names: bindings.move_into(self.arena).leak(), 
                hint, rhs
            }
        }), source))
        
    }

    fn assignment(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        fn binary_op_assignment<'la>(
            parser: &mut Parser<'_, 'la, '_>, 
            operator: BinaryOperator, 
            lhs: Node<'la>, 
            settings: &ParserSettings<'la>
        ) -> ParseResult<'la> {

            parser.advance();
            parser.advance();

            let rhs = parser.expression(settings)?;
            let range = SourceRange::new(lhs.source_range.start(), parser.current_range().end());

            let lhs = parser.arena.alloc_new(lhs);
            Ok(Node::new(
                NodeKind::Statement(Statement::UpdateValue { 
                    lhs,
                    rhs: parser.arena.alloc_new(Node::new(
                        NodeKind::Expression(Expression::BinaryOp {
                                operator, 
                                lhs, 
                                rhs: parser.arena.alloc_new(rhs),
                            }),
                        range,
                    )) 
                }),
                range
            ))
        }

        
        let start = self.current_range().start();
        let lhs = self.expression(&ParserSettings::default())?;


        match self.peek_kind() {
            Some(TokenKind::AddEquals) => binary_op_assignment(self, BinaryOperator::Add, lhs, settings),
            Some(TokenKind::SubEquals) => binary_op_assignment(self, BinaryOperator::Sub, lhs, settings),
            Some(TokenKind::MulEquals) => binary_op_assignment(self, BinaryOperator::Mul, lhs, settings),
            Some(TokenKind::DivEquals) => binary_op_assignment(self, BinaryOperator::Div, lhs, settings),
            Some(TokenKind::Equals) => {
                self.advance();
                self.advance();

                let rhs = self.expression(settings)?;

                Ok(Node::new(
                    NodeKind::Statement(Statement::UpdateValue { 
                        lhs: self.arena.alloc_new(lhs), 
                        rhs: self.arena.alloc_new(rhs) 
                    }),
                    SourceRange::new(start, self.current_range().end())
                ))
            }
            _ => Ok(lhs)
        }
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn expression(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.logical_or(settings)
    }


    fn logical_or(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        let lhs = self.logical_and(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalOr) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.logical_and(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end());

        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: self.arena.alloc_new(lhs),
                body: Block::new(self.arena.alloc_new([
                    Node::new(
                        NodeKind::Expression(Expression::Literal(Literal::Bool(true))),
                        range
                    )]),
                    range
                ),
                else_block: Some(self.arena.alloc_new(Node::new(
                    NodeKind::Expression(Expression::Block {
                            block: Block::new(self.arena.alloc_new([rhs]), 
                            range
                        )
                    }), 
                    range,
                )))
            }),
            range
        ))
    }


    fn logical_and(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        let lhs = self.unary_not(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalAnd) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.unary_not(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end());

        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: self.arena.alloc_new(lhs),
                body: Block::new(self.arena.alloc_new([rhs]), range),
                else_block: Some(self.arena.alloc_new(Node::new(
                    NodeKind::Expression(Expression::Block {
                            block: Block::new(self.arena.alloc_new([Node::new(
                                NodeKind::Expression(Expression::Literal(Literal::Bool(false))),
                                range
                            )]), 
                            range
                        )
                    }), 
                    range,
                )))
            }),
            range
        ))
    }


    fn unary_not(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        if self.current_is(TokenKind::Bang) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.comparisson(settings)?;
            return Ok(Node::new(
                NodeKind::Expression(Expression::UnaryOp { 
                    operator: nodes::UnaryOperator::Not, 
                    rhs: self.arena.alloc_new(expr) 
                }),
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.comparisson(settings)
    }


    fn comparisson(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::bitwise_or, 
            Self::expression, 
            &[
                TokenKind::LeftAngle, TokenKind::RightAngle,
                TokenKind::GreaterEquals, TokenKind::LesserEquals,
                TokenKind::EqualsTo, TokenKind::NotEqualsTo,
            ], 
            settings,
        )
    }
    

    fn bitwise_or(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::bitwise_xor, 
            Self::bitwise_xor, 
            &[TokenKind::BitwiseOr], 
            settings,
        )
        
    }


    fn bitwise_xor(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::bitwise_and, 
            Self::bitwise_and, 
            &[TokenKind::BitwiseXor], 
            settings,
        )
        
    }


    fn bitwise_and(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::bitshifts, 
            Self::bitshifts, 
            &[TokenKind::Ampersand], 
            settings,
        )
        
    }
    

    fn bitshifts(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::arithmetic, 
            Self::arithmetic, 
            &[TokenKind::BitshiftLeft, TokenKind::BitshiftRight], 
            settings,
        )
        
    }
    

    fn arithmetic(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::product, 
            Self::product, 
            &[TokenKind::Plus, TokenKind::Minus], 
            settings,
        )
    }


    fn product(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.binary_operation(
            Self::unary_neg, 
            Self::unary_neg, 
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent], 
            settings,
        )
    }
    

    fn unary_neg(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        if self.current_is(TokenKind::Minus) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.as_cast(settings)?;
            return Ok(Node::new(
                NodeKind::Expression(Expression::UnaryOp { 
                    operator: nodes::UnaryOperator::Neg, 
                    rhs: self.arena.alloc_new(expr) 
                }),
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.as_cast(settings)
    }


    fn as_cast(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        let expr = self.accessors(settings)?;
        if !self.peek_is(TokenKind::Keyword(Keyword::As)) {
            return Ok(expr)
        }

        self.advance();
        self.advance();
        let ty = self.expect_type()?;

        let nk = NodeKind::Expression(Expression::AsCast {
            lhs: self.arena.alloc_new(expr), data_type: ty });

        Ok(Node::new(nk, SourceRange::new(expr.range().start(), ty.range().end())))
    }


    fn accessors(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        let mut result = self.atom(settings)?;

        while 
            self.peek_kind() == Some(TokenKind::Dot) 
            || self.peek_kind() == Some(TokenKind::Bang)
            || self.peek_kind() == Some(TokenKind::QuestionMark) {
            self.advance();

            if self.current_is(TokenKind::Bang) {
                let source = SourceRange::new(result.range().start(), self.current_range().end());
                result = Node::new(
                    NodeKind::Expression(Expression::Unwrap(self.arena.alloc_new(result))),
                    source,
                );
                continue
            }

            if self.current_is(TokenKind::QuestionMark) {
                let source = SourceRange::new(result.range().start(), self.current_range().end());
                result = Node::new(
                    NodeKind::Expression(Expression::OrReturn(self.arena.alloc_new(result))),
                    source,
                );
                continue
            }
            
            self.advance();
            
            let start = self.current_range().start();
            let ident = match self.current_kind() {
                TokenKind::Literal(Literal::Integer(int)) => {
                    let pool = ArenaPool::tls_get_temp();
                    let string = format_in!(&*pool, "{}", int);
                    self.string_map.insert(&string)
                },

                _ => self.expect_identifier()?,
            };

            if self.string_map.get(ident) == "cast" {
                self.advance();
                result = self.cast_expr(result)?;
                continue
            }

            if self.peek_kind() == Some(TokenKind::LeftParenthesis) {
                self.advance();
                self.advance();

                let args = self.parse_function_call_args(Some(result))?;

                result = Node::new(
                    NodeKind::Expression(Expression::CallFunction {
                        name: ident, 
                        args,
                        is_accessor: true, 
                    }),
                    SourceRange::new(start, self.current_range().end())
                )

                
            } else {
                result = Node::new(
                    NodeKind::Expression(Expression::AccessField { 
                        val: self.arena.alloc_new(result), 
                        field_name: ident,
                    }),
                    SourceRange::new(start, self.current_range().end())
                )
            }
        }

        Ok(result)
    }
    

    fn atom(&mut self, settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        self.is_error_token()?;

        match self.current_kind() {
            TokenKind::Literal(l) => Ok(Node::new(
                NodeKind::Expression(Expression::Literal(l)), 
                self.current_range(),
            )),

            TokenKind::Underscore => {
                 return Ok(Node::new(
                    NodeKind::Expression(Expression::Unit), 
                    self.current_range(),
                ))
            }


            TokenKind::LeftParenthesis => {
                let start = self.current_range().start();
                self.advance();

                if self.current_is(TokenKind::RightParenthesis) {
                     return Ok(Node::new(
                        NodeKind::Expression(Expression::Unit), 
                        self.current_range(),
                    ))       
                }

                let mut expr = self.expression(&ParserSettings::default())?;
                self.advance();

                if self.current_is(TokenKind::Comma) {
                    let pool = ArenaPool::tls_get_rec();
                    let mut vec = Vec::new_in(&*pool);
                    vec.push(expr);
                    while self.current_is(TokenKind::Comma) {
                        self.advance();
                        if self.current_is(TokenKind::RightParenthesis) { break }

                        vec.push(self.expression(&ParserSettings::default())?);
                        self.advance();
                    }
                    self.expect(TokenKind::RightParenthesis)?;
                    return Ok(Node::new(
                        NodeKind::Expression(Expression::Tuple(vec.move_into(self.arena).leak())), 
                        SourceRange::new(start, self.current_range().end())
                    ));
                }

                self.expect(TokenKind::RightParenthesis)?;

                expr.source_range = SourceRange::new(start, self.current_range().end());
                
                Ok(expr)
            },


            TokenKind::LeftBracket => self.block_expression(),


            TokenKind::Identifier(v) => {
                if self.peek_kind() == Some(TokenKind::LeftParenthesis) {
                    return self.function_call_expression()
                }


                if settings.can_parse_struct_creation 
                    && (
                        self.peek_kind() == Some(TokenKind::LeftBracket)
                        || self.peek_kind() == Some(TokenKind::QuestionMark)
                    ) {
                    return self.struct_creation_expression()
                }


                if self.peek_kind() == Some(TokenKind::DoubleColon) {
                    let source = self.current_range();
                    let start = self.current_range().start();
                    self.advance();
                    self.advance();

                    let expr = self.atom(settings)?;
                    
                    return Ok(Node::new(
                        NodeKind::Expression(Expression::WithinNamespace { 
                            namespace: v,
                            action: self.arena.alloc_new(expr),
                            namespace_source: source,
                        }),
                        SourceRange::new(start, self.current_range().end())
                    ))
                }
                
                Ok(Node::new(
                    NodeKind::Expression(Expression::Identifier(v)),
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Match) => self.match_expression(settings),
            TokenKind::Keyword(Keyword::If) => self.if_expression(),
            
            
            TokenKind::At => self.parse_with_tag(settings, Self::expression),


            TokenKind::Keyword(Keyword::Return) => {
                let start = self.current_range().start();

                self.advance();

                let expr = self.expression(&ParserSettings::default())?;
                Ok(Node::new(
                    NodeKind::Expression(Expression::Return(self.arena.alloc_new(expr))), 
                    SourceRange::new(start, expr.range().end())
                ))
            }


            TokenKind::Keyword(Keyword::Break) => {
                Ok(Node::new(
                    NodeKind::Expression(Expression::Break), 
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Continue) => {
                Ok(Node::new(
                    NodeKind::Expression(Expression::Continue), 
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

                Ok(Node::new(
                    NodeKind::Expression(Expression::Loop { body }),
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

                let else_block = Node::new(
                    NodeKind::Expression(Expression::Break),
                    source,
                );

                let if_node = Node::new(
                    NodeKind::Expression(Expression::If {
                        condition: self.arena.alloc_new(expr),
                        body,
                        else_block: Some(self.arena.alloc_new(else_block)),
                    }),
                    source
                );

                Ok(Node::new(
                    NodeKind::Expression(Expression::Loop {
                        body: Block::new(self.arena.alloc_new([if_node]), source) }),
                    source,
                ))
            }


            TokenKind::LeftSquare => {
                let start = self.current_range().start();
                self.advance();

                let typ = self.expect_type()?;
                self.advance();

                self.expect(TokenKind::RightSquare)?;
                self.advance();

                self.expect(TokenKind::DoubleColon)?;
                self.advance();

                let expr = self.expression(&ParserSettings::default())?;

                Ok(Node::new(
                    NodeKind::Expression(Expression::WithinTypeNamespace { 
                        namespace: typ, 
                        action: self.arena.alloc_new(expr) 
                    }),
                    SourceRange::new(start, self.current_range().end())
                ))
            }

            
            _ => Err(ErrorId::Parser(
                self.errors.push(Error::UnexpectedToken(self.current_range()))
            ))
        }
    }



    fn match_expression(&mut self, _settings: &ParserSettings<'ta>) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Match))?;
        self.advance();

        let taken_as_inout = if self.current_is(TokenKind::Ampersand) { self.advance(); true }
                             else { false };
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

        let mut mappings = vec![];
        loop {            
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            if !mappings.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightBracket {
                break
            }

            
            let start = self.current_range().start();
            let name = match self.current_kind() {
                TokenKind::Literal(Literal::Bool(true)) => StringMap::TRUE,
                TokenKind::Literal(Literal::Bool(false)) => StringMap::FALSE,
                _ => self.expect_identifier()?,
            };
            let source_range = SourceRange::new(start, self.current_range().end());
            self.advance();

            let (bind_to, is_inout, binding_range) =
                if self.current_is(TokenKind::Colon) {
                    self.advance();

                    let binding_start = self.current_range().start();
                    let is_inout = if self.current_is(TokenKind::Ampersand) {
                        self.advance();
                        true
                    } else { false };
                    
                    let name = self.expect_identifier()?;
                    let binding_range = SourceRange::new(binding_start, self.current_range().end());
                    self.advance();
                    (name, is_inout, binding_range)

                } else {
                    (self.string_map.insert("_"), false, self.current_range())
                };


            self.expect(TokenKind::Arrow)?;
            self.advance();

            let expr = self.statement(&ParserSettings::default())?;
            self.advance();

            mappings.push(MatchMapping::new(name, bind_to, binding_range, source_range, expr, is_inout));
        }
        let mappings = mappings;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::Match { 
                value: self.arena.alloc_new(val), 
                taken_as_inout,
                mappings: mappings.leak()
            }),
            SourceRange::new(start, end)
        ))
    }


    fn block_expression(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, start, &ParserSettings::default())?;

        Ok(Node::new(
            NodeKind::Expression(Expression::Block { block }),
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn if_expression(&mut self) -> ParseResult<'ta> {
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

        
        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: self.arena.alloc_new(condition), 
                body, 
                else_block: else_block.map(|x| &*self.arena.alloc_new(x)),
            }),
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn function_call_expression(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let args = self.parse_function_call_args(None)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::CallFunction { name, args, is_accessor: false }),
            SourceRange::new(start, end)
        ))
        
    }


    fn parse_function_call_args(
        &mut self, 
        associated: Option<Node<'ta>>
    ) -> Result<&'ta mut [(Node<'ta>, bool)], ErrorId> {

        let binding = ArenaPool::tls_get_rec();
        let mut args = Vec::new_in(&*binding);

        if let Some(node) = associated {
            args.push((node, false));
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


            let is_inout = if self.current_is(TokenKind::Ampersand) { self.advance(); true }
                            else { false };
            let expr = self.expression(&ParserSettings::default())?;
            self.advance();
            
            args.push((expr, is_inout));
        }
        self.expect(TokenKind::RightParenthesis)?;

        Ok(args.move_into(self.arena).leak())
    }


    fn struct_creation_expression(&mut self) -> ParseResult<'ta> {
        let start = self.current_range().start();
        let data_type = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let pool = ArenaPool::tls_get_rec();
        let mut fields = Vec::new_in(&*pool);
        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            if !fields.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightBracket {
                break
            }

            let start = self.current_range().start();
            let name = self.expect_identifier()?;
            self.advance();

            self.expect(TokenKind::Colon)?;
            self.advance();

            let expr = self.expression(&ParserSettings::default())?;
            let end = self.current_range().end();
            self.advance();
            
            fields.push((name, SourceRange::new(start, end), expr));
        }

        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::CreateStruct { data_type, fields: fields.move_into(self.arena).leak() }),
            SourceRange::new(start, end),
        ))
    }


    fn cast_expr(&mut self, lhs: Node<'ta>) -> ParseResult<'ta> {
        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let typ = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::RightParenthesis)?;

        let source = SourceRange::new(lhs.range().start(), self.current_range().end());
        Ok(Node::new(
            NodeKind::Expression(Expression::CastAny { 
                lhs: self.arena.alloc_new(lhs), 
                data_type: typ 
            }),
            source,
        ))
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn binary_operation(
        &mut self,
        lhs: fn(&mut Self, &ParserSettings<'ta>) -> ParseResult<'ta>,
        rhs: fn(&mut Self, &ParserSettings<'ta>) -> ParseResult<'ta>,
        tokens: &[TokenKind],
        settings: &ParserSettings<'ta>,
    ) -> ParseResult<'ta> {
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
                lhs.range().start(), 
                rhs.range().end(),
            );

            lhs = Node::new(
                NodeKind::Expression(Expression::BinaryOp { 
                    operator, 
                    lhs: self.arena.alloc_new(lhs), 
                    rhs: self.arena.alloc_new(rhs) 
                }), 
                range,
            )
        }
        
        Ok(lhs)
    }

}
