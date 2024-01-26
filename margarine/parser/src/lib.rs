pub mod nodes;
pub mod errors;

use std::{ops::Deref, process::Termination};

use common::{source::SourceRange, string_map::{StringMap, StringIndex}};
use errors::Error;
use ::errors::{ParserError, ErrorId};
use lexer::{Token, TokenKind, TokenList, Keyword, Literal};
use nodes::{Node, attr::{AttributeNode, Attribute}, err::ErrorNode, expr::{ExpressionNode, Expression, UnaryOperator, MatchMapping}, stmt::{StatementNode, Statement}, decl::{StructKind, Declaration, DeclarationNode, FunctionArgument, ExternFunction, EnumMapping, UseItem, UseItemKind, FunctionSignature}, Pattern, PatternKind};
use sti::{prelude::{Vec, Arena}, arena_pool::ArenaPool, keyed::KVec, format_in};

use crate::nodes::expr::BinaryOperator;

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
    RcConst(&'a DataType<'a>),
    RcMut(&'a DataType<'a>),
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

            DataTypeKind::RcConst(v) => {
                12.hash(state);
                v.kind().hash(state);
            }

            DataTypeKind::RcMut(v) => {
                13.hash(state);
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

            let is_mut = if self.current_is(TokenKind::Keyword(Keyword::Mut)) { 
                self.advance(); 
                true 
            } else { false };

            let ty = self.expect_type()?;
            let alloc = self.arena.alloc_new(ty);
            DataType::new(
                SourceRange::new(start, ty.range().end()),
                if is_mut {
                    DataTypeKind::RcMut(alloc)
                } else {
                    DataTypeKind::RcConst(alloc)
                }
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


    fn parse_with_attr<T: Into<Node<'ta>>>(
        &mut self, 
        settings: &ParserSettings<'ta>,
        func: fn(&mut Self, &ParserSettings<'ta>) -> Result<T, ErrorId>,
    ) -> Result<AttributeNode<'ta>, ErrorId> {
        self.expect(TokenKind::At)?;
        self.advance();

        let attr = self.parse_attribute()?;
        self.advance();
        let func = func(self, settings)?;

        Ok(AttributeNode::new(attr, func.into()))
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


    fn parse_pattern(&mut self) -> Result<Pattern<'ta>, ErrorId> {
        let start = self.current_range().start();
        
        let pool = ArenaPool::tls_get_temp();
        let mut vec = Vec::new_in(&*pool);
        loop {
            if !vec.is_empty() {
                if self.peek_is(TokenKind::Comma) { self.advance(); self.advance() }
                else { break }
            }

            if self.current_is(TokenKind::LeftParenthesis) {
                self.advance();

                let pattern = self.parse_pattern()?;
                self.advance();

                self.expect(TokenKind::RightParenthesis)?;

                vec.push(pattern);
                continue
            }

            let pattern;

            let is_inout = self.is_inout();
            let ident = self.expect_identifier()?;

            if self.peek_is(TokenKind::DoubleColon) { pattern = self.parse_pattern_struct(is_inout)? }
            else if self.peek_is(TokenKind::LeftBracket) { pattern = self.parse_pattern_struct(is_inout)? }
            else { pattern = Pattern::new(self.current_range(), is_inout, PatternKind::Ident(ident)) }

            vec.push(pattern)
        }

        if vec.len() == 1 { return Ok(vec[0]) }

        Ok(Pattern::new(
            SourceRange::new(start, self.current_range().end()),
            vec.iter().any(|x| x.is_inout()),
            PatternKind::Tuple(vec.move_into(self.arena).leak()),
        ))
    }


    #[inline(always)]
    fn is_inout(&mut self) -> bool {
        if self.current_is(TokenKind::Ampersand) {
            self.advance();
            true
        } else {
            false
        }
    }


    fn parse_pattern_struct(&mut self, is_inout: bool) -> Result<Pattern<'ta>, ErrorId> {
        let start = self.current_range().start();
        let ty = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let fields = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, _| {
            let is_inout = parser.is_inout();
            let ident = parser.expect_identifier()?;

            if !parser.peek_is(TokenKind::Colon) {
                return Ok((ident, Pattern::new(parser.current_range(), is_inout, PatternKind::Ident(ident))));
            }

            parser.advance();
            parser.advance();

            let pattern = parser.parse_pattern()?;
            Ok((
                ident,
                pattern,
            ))
        }
        )?;


        Ok(Pattern::new(
            SourceRange::new(start, self.current_range().end()), 
            is_inout || fields.iter().any(|x| x.1.is_inout()),
            PatternKind::Struct(ty, fields),
        ))
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
        mut func: impl FnMut(&mut Self, usize) -> Result<T, ErrorId>,
    ) -> Result<&'ta [T], ErrorId> {
        let mut arguments = Vec::new_in(self.arena);

        loop {
            if self.current_kind() == TokenKind::EndOfFile { break }
            if self.current_kind() == terminator { break }
            if let Some(punctuation) = punctuation {
                if !arguments.is_empty() {
                    self.expect(punctuation)?;
                    self.advance();
                }
                
                // allow for trailing punctuation
                if self.current_kind() == terminator { break }
            }


            let result = func(self, arguments.len())?;
            self.advance();
            arguments.push(result);
        }

        self.expect(terminator)?;
        Ok(arguments.leak())
    }
    

    fn parse_till(
        &mut self, 
        terminator: TokenKind, 
        start: u32,
        settings: &ParserSettings<'ta>
    ) -> Result<Block<'ta>, ErrorId> {

        let mut storage : Vec<Node<'ta>, _> = Vec::with_cap_in(self.arena, 1);

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
                Ok (e) => storage.push(e.into()),
                Err(e) => {
                    storage.push(ErrorNode::new(
                        e,
                        SourceRange::new(start, self.current_range().end())
                    ).into());

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
            storage.push(ExpressionNode::new(
                Expression::Unit,
                SourceRange::new(start, end)
            ).into());
        }

        Ok(Block::new(storage.leak(), SourceRange::new(start, end)))
    }


    fn parse_till_decl(
        &mut self, 
        terminator: TokenKind, 
        start: u32,
        settings: &ParserSettings<'ta>
    ) -> Result<&'ta [DeclarationNode<'ta>], ErrorId> {
        let parse_till = self.parse_till(terminator, start, settings)?;
        let mut vec = Vec::with_cap_in(self.arena, parse_till.len());
        for n in parse_till.into_iter() {
            let Node::Declaration(decl) = *n
            else {
                return Err(ErrorId::Parser(self.errors.push(
                            Error::DeclarationOnlyBlock { source: n.range() })));
            };

            vec.push(decl);
        }

        Ok(vec.leak())
    }
}


type StatementResult<'ta> = Result<StatementNode<'ta>, ErrorId>;
type DeclarationResult<'ta> = Result<DeclarationNode<'ta>, ErrorId>;
impl<'ta> Parser<'_, 'ta, '_> {
    fn statement(&mut self, settings: &ParserSettings<'ta>) -> Result<Node<'ta>, ErrorId> {
        let node : Node = match self.current_kind() {
            | TokenKind::Keyword(Keyword::Struct)
            | TokenKind::Keyword(Keyword::Resource)
            | TokenKind::Keyword(Keyword::Component)
            => self.struct_declaration()?.into(),


            TokenKind::Keyword(Keyword::Fn) => self.function_declaration(false, &settings)?.into(),
            TokenKind::Keyword(Keyword::System) => {
                self.advance();
                self.function_declaration(true, &settings)?.into()
            },


            TokenKind::Keyword(Keyword::Impl) => self.impl_declaration()?.into(),
            TokenKind::Keyword(Keyword::Mod) => self.mod_declaration()?.into(),
            TokenKind::Keyword(Keyword::Extern) => self.extern_declaration(settings)?.into(),
            TokenKind::Keyword(Keyword::Enum) => self.enum_declaration()?.into(),
            TokenKind::Keyword(Keyword::Use) => self.using_declaration()?.into(),


            TokenKind::Keyword(Keyword::Let) => self.let_statement()?.into(),
            TokenKind::Keyword(Keyword::For) => self.for_statement()?.into(),


            TokenKind::At => {
                let attr = self.parse_with_attr(&settings, Self::statement)?;
                let attr : &'ta _ = self.arena.alloc_new(attr);
                attr.into()
            }


            _ => self.assignment(&settings)?.into(),
        };

        Ok(node)
    }


    fn struct_declaration(&mut self) -> DeclarationResult<'ta> {
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

        let fields = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, _| {
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;
            parser.advance();

            parser.expect(TokenKind::Colon)?;
            parser.advance();

            let datatype = parser.expect_type()?;
            let end = parser.current_range().end();
            parser.advance();

            Ok((name, datatype, SourceRange::new(start, end)))
        });

        let fields = fields?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        let node = Declaration::Struct { kind, name, header, fields };

        Ok(DeclarationNode::new(
            node, 
            SourceRange::new(start, end)
        ))
    }



    fn function_declaration(
        &mut self, 
        is_system: bool, 
        settings: &ParserSettings<'ta>
    ) -> DeclarationResult<'ta> {

        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Fn))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let arguments = self.list(TokenKind::RightParenthesis, Some(TokenKind::Comma), |parser, index| {
            let is_inout = parser.is_inout();
            
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;

            if index == 0
                && name == StringMap::SELF {
                if let Some(parser_type) = settings.is_in_impl {
                    return Ok(FunctionArgument::new(
                        name,
                        parser_type,
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

        Ok(DeclarationNode::new(
            Declaration::Function {
                sig: FunctionSignature::new(
                     is_system, 
                     name, 
                     header,
                     arguments,
                     return_type
                ),
                body,
            },

            SourceRange::new(start, end)
        ))
    }


    fn impl_declaration(&mut self) -> DeclarationResult<'ta> {
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
        
        let body = self.parse_till_decl(TokenKind::RightBracket, body_start, &settings)?;
        let end = self.current_range().end();

        Ok(DeclarationNode::new(
            Declaration::Impl { 
                data_type, body
            },

            SourceRange::new(start, end),
        ))
    }


    fn mod_declaration(&mut self) -> DeclarationResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Mod))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till_decl(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(DeclarationNode::new(
            Declaration::Module { name, body },
            SourceRange::new(start, end)
        ))
    }


    fn extern_declaration(&mut self, settings: &ParserSettings<'ta>) -> DeclarationResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Extern))?;
        self.advance();

        let file = self.expect_literal_str()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let functions = self.list(TokenKind::RightBracket, None, |parser, _| {
            let start = parser.current_range().start();
            parser.expect(TokenKind::Keyword(Keyword::Fn))?;
            parser.advance();

            let name = parser.expect_identifier()?;
            
            parser.advance();

            let path = if let Some(path) = parser.is_literal_str() { parser.advance(); path }
            else { name };

            parser.expect(TokenKind::LeftParenthesis)?;
            parser.advance();

            let arguments = parser.list(TokenKind::RightParenthesis, Some(TokenKind::Comma),
            |parser, index| {
                let start = parser.current_range().start();
                let is_inout = if parser.current_is(TokenKind::Colon) {
                    parser.advance();
                    true
                } else { false };

                let identifier = parser.expect_identifier()?;

                if index == 0
                    && identifier == StringMap::SELF {
                    if let Some(parser_type) = settings.is_in_impl {
                        return Ok(FunctionArgument::new(
                            name,
                            parser_type,
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
                name,
                path,
                arguments,
                return_type,
                SourceRange::new(start, end)
            ))
        });
        let functions = functions?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(DeclarationNode::new(
            Declaration::Extern { file, functions },
            SourceRange::new(start, end)
        ))
    }


    fn enum_declaration(&mut self) -> DeclarationResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Enum))?;
        self.advance();

        let name = self.expect_identifier()?;
        let header = SourceRange::new(start, self.current_range().end());
        self.advance();

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
            parser.advance();
            
            let mapping = EnumMapping::new(
                name, 
                index.try_into().unwrap(), 
                data_type, 
                SourceRange::new(start, end), 
                is_implicit_unit
            );

            Ok(mapping)
        });
        let mappings = mappings?;

        let end = self.current_range().end();

        Ok(DeclarationNode::new(
            Declaration::Enum { name, mappings, header },
            SourceRange::new(start, end)
        ))
    }


    fn using_declaration(&mut self) -> DeclarationResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Use))?;
        self.advance();

        let item = self.parse_use_item()?;

        Ok(DeclarationNode::new(
                Declaration::Using { item },
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

                let list = self.list(TokenKind::RightParenthesis, Some(TokenKind::Comma), 
                                    |parser, _| parser.parse_use_item())?;

                return Ok(UseItemKind::List { list })
            }

            Ok(UseItemKind::BringName)

        };

        let item = func()?;

        Ok(UseItem::new(ident, item, SourceRange::new(start, self.current_range().end())))
    }


    fn for_statement(&mut self) -> StatementResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.advance();

        let binding = self.parse_pattern()?;
        self.advance();

        self.expect(TokenKind::Keyword(Keyword::In))?;
        self.advance();

        let is_expr_inout = self.is_inout();

        let expr = self.expression(
            &ParserSettings { can_parse_struct_creation: false, ..Default::default() })?;
        self.advance();


        let block_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, block_start, &ParserSettings::default())?;

        Ok(StatementNode::new(
            Statement::ForLoop {
                binding,
                expr: (is_expr_inout, expr),
                body: block
            },
            SourceRange::new(start, self.current_range().end()),
        ))
    }

    fn let_statement(&mut self) -> StatementResult<'ta> {
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
        
        Ok(StatementNode::new(if bindings.len() == 1 {
            let b = bindings[0];
            Statement::Variable { name: b.0, hint, is_mut: b.1, rhs }
        } else {
            Statement::VariableTuple {
                names: bindings.move_into(self.arena).leak(), 
                hint, rhs
            }
        }, source))
        
    }



    fn assignment(&mut self, settings: &ParserSettings<'ta>) -> Result<Node<'ta>, ErrorId> {
        fn binary_op_assignment<'la>(
            parser: &mut Parser<'_, 'la, '_>, 
            operator: BinaryOperator, 
            lhs: ExpressionNode<'la>, 
            settings: &ParserSettings<'la>
        ) -> StatementResult<'la> {

            parser.advance();
            parser.advance();

            let rhs = parser.expression(settings)?;
            let range = SourceRange::new(lhs.range().start(), parser.current_range().end());

            Ok(StatementNode::new(
                Statement::UpdateValue { 
                    lhs,
                    rhs: ExpressionNode::new(
                        Expression::BinaryOp {
                            operator, 
                            lhs: parser.arena.alloc_new(lhs), 
                            rhs: parser.arena.alloc_new(rhs),
                        },
                        range,
                    )
                },
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

                StatementNode::new(
                    Statement::UpdateValue { 
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


type ExpressionResult<'a> = Result<ExpressionNode<'a>, ErrorId>;
impl<'ta> Parser<'_, 'ta, '_> {
    fn expression(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.logical_or(settings)
    }


    fn logical_or(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        let lhs = self.logical_and(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalOr) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.logical_and(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end());

        Ok(ExpressionNode::new(
            Expression::If {
                condition: self.arena.alloc_new(lhs),
                body: self.arena.alloc_new(ExpressionNode::new(
                    Expression::Literal(Literal::Bool(true)),
                    range
                )),
                else_block: Some(self.arena.alloc_new(rhs.into()))
            },
            range
        ))
    }


    fn logical_and(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        let lhs = self.unary_not(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalAnd) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.unary_not(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end());

        Ok(ExpressionNode::new(
            Expression::If {
                condition: self.arena.alloc_new(lhs),
                body: self.arena.alloc_new(rhs.into()),
                else_block: Some(self.arena.alloc_new(ExpressionNode::new(
                    Expression::Literal(Literal::Bool(false)),
                    range
                ))),
            },
            range
        ))
    }


    fn unary_not(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        if self.current_is(TokenKind::Bang) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.comparisson(settings)?;
            return Ok(ExpressionNode::new(
                Expression::UnaryOp { 
                    operator: UnaryOperator::Not, 
                    rhs: self.arena.alloc_new(expr) 
                },
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.comparisson(settings)
    }


    fn comparisson(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
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
    

    fn bitwise_or(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::bitwise_xor, 
            Self::bitwise_xor, 
            &[TokenKind::BitwiseOr], 
            settings,
        )
        
    }


    fn bitwise_xor(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::bitwise_and, 
            Self::bitwise_and, 
            &[TokenKind::BitwiseXor], 
            settings,
        )
        
    }


    fn bitwise_and(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::bitshifts, 
            Self::bitshifts, 
            &[TokenKind::Ampersand], 
            settings,
        )
        
    }
    

    fn bitshifts(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::arithmetic, 
            Self::arithmetic, 
            &[TokenKind::BitshiftLeft, TokenKind::BitshiftRight], 
            settings,
        )
        
    }
    

    fn arithmetic(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::product, 
            Self::product, 
            &[TokenKind::Plus, TokenKind::Minus], 
            settings,
        )
    }


    fn product(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.binary_operation(
            Self::unary_neg, 
            Self::unary_neg, 
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent], 
            settings,
        )
    }
    

    fn unary_neg(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        if self.current_is(TokenKind::Minus) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.as_cast(settings)?;
            return Ok(ExpressionNode::new(
                Expression::UnaryOp { 
                    operator: UnaryOperator::Neg, 
                    rhs: self.arena.alloc_new(expr) 
                },
                SourceRange::new(start, self.current_range().end())
            ))
        }

        self.as_cast(settings)
    }


    fn as_cast(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        let expr = self.accessors(settings)?;
        if !self.peek_is(TokenKind::Keyword(Keyword::As)) {
            return Ok(expr)
        }

        self.advance();
        self.advance();
        let ty = self.expect_type()?;

        let nk = Expression::AsCast { lhs: self.arena.alloc_new(expr), data_type: ty };

        Ok(ExpressionNode::new(nk, SourceRange::new(expr.range().start(), ty.range().end())))
    }


    fn accessors(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        let mut result = self.atom(settings)?;

        while 
            self.peek_kind() == Some(TokenKind::Dot) 
            || self.peek_kind() == Some(TokenKind::Bang)
            || self.peek_kind() == Some(TokenKind::QuestionMark) {
            self.advance();

            if self.current_is(TokenKind::Bang) {
                let source = SourceRange::new(result.range().start(), self.current_range().end());
                result = ExpressionNode::new(
                    Expression::Unwrap(self.arena.alloc_new(result)),
                    source,
                );
                continue
            }

            if self.current_is(TokenKind::QuestionMark) {
                let source = SourceRange::new(result.range().start(), self.current_range().end());
                result = ExpressionNode::new(
                    Expression::OrReturn(self.arena.alloc_new(result)),
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

                result = ExpressionNode::new(
                    Expression::CallFunction {
                        name: ident, 
                        args,
                        is_accessor: true, 
                    },
                    SourceRange::new(start, self.current_range().end())
                )

                
            } else {
                result = ExpressionNode::new(
                    Expression::AccessField { 
                        val: self.arena.alloc_new(result), 
                        field_name: ident,
                    },
                    SourceRange::new(start, self.current_range().end())
                )
            }
        }

        Ok(result)
    }
    

    fn atom(&mut self, settings: &ParserSettings<'ta>) -> ExpressionResult<'ta> {
        self.is_error_token()?;

        match self.current_kind() {
            TokenKind::Literal(l) => Ok(ExpressionNode::new(
                Expression::Literal(l), 
                self.current_range(),
            )),

            TokenKind::Underscore => {
                 return Ok(ExpressionNode::new(
                    Expression::Unit, 
                    self.current_range(),
                ))
            }


            TokenKind::LeftParenthesis => {
                let start = self.current_range().start();
                self.advance();

                if self.current_is(TokenKind::RightParenthesis) {
                     return Ok(ExpressionNode::new(
                        Expression::Unit, 
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
                    return Ok(ExpressionNode::new(
                        Expression::Tuple(vec.move_into(self.arena).leak()), 
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
                    
                    return Ok(ExpressionNode::new(
                        Expression::WithinNamespace { 
                            namespace: v,
                            action: self.arena.alloc_new(expr),
                            namespace_source: source,
                        },
                        SourceRange::new(start, self.current_range().end())
                    ))
                }
                
                Ok(ExpressionNode::new(
                    Expression::Identifier(v),
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Match) => self.match_expression(),
            TokenKind::Keyword(Keyword::If) => self.if_expression(),
            
            
            // TokenKind::At => self.parse_with_attr(settings, Self::expression),


            TokenKind::Keyword(Keyword::Return) => {
                let start = self.current_range().start();

                self.advance();

                let expr = self.expression(&ParserSettings::default())?;
                Ok(ExpressionNode::new(
                    Expression::Return(self.arena.alloc_new(expr)), 
                    SourceRange::new(start, expr.range().end())
                ))
            }


            TokenKind::Keyword(Keyword::Break) => {
                Ok(ExpressionNode::new(
                    Expression::Break, 
                    self.current_range(),
                ))
            }


            TokenKind::Keyword(Keyword::Continue) => {
                Ok(ExpressionNode::new(
                    Expression::Continue, 
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

                Ok(ExpressionNode::new(
                    Expression::Loop { body },
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

                let else_block = ExpressionNode::new(
                    Expression::Break,
                    source,
                );

                let if_node = ExpressionNode::new(
                    Expression::If {
                        condition: self.arena.alloc_new(expr),
                        body: self.arena.alloc_new(
                            ExpressionNode::new(Expression::Block { block: body }, body.range())),
                        else_block: Some(self.arena.alloc_new(else_block)),
                    },
                    source
                );

                Ok(ExpressionNode::new(
                    Expression::Loop {
                        body: Block::new(self.arena.alloc_new([if_node.into()]), source) },
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

                Ok(ExpressionNode::new(
                    Expression::WithinTypeNamespace { 
                        namespace: typ, 
                        action: self.arena.alloc_new(expr) 
                    },
                    SourceRange::new(start, self.current_range().end())
                ))
            }

            
            _ => Err(ErrorId::Parser(
                self.errors.push(Error::UnexpectedToken(self.current_range()))
            ))
        }
    }



    fn match_expression(&mut self) -> ExpressionResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Match))?;
        self.advance();

        let taken_as_inout = self.is_inout();
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
            let source_range = SourceRange::new(start, parser.current_range().end());
            parser.advance();

            let (bind_to, is_inout, binding_range) =
                if parser.current_is(TokenKind::Colon) {
                    parser.advance();

                    let binding_start = parser.current_range().start();
                    let is_inout = parser.is_inout();
                    
                    let name = parser.expect_identifier()?;
                    let binding_range = SourceRange::new(binding_start, parser.current_range().end());
                    parser.advance();
                    (name, is_inout, binding_range)

                } else {
                    (parser.string_map.insert("_"), false, parser.current_range())
                };


            parser.expect(TokenKind::Arrow)?;
            parser.advance();

            let expr = parser.expression(&ParserSettings::default())?;
            parser.advance();

            Ok(MatchMapping::new(name, bind_to, binding_range, source_range, expr, is_inout))
        })?;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(ExpressionNode::new(
            Expression::Match { 
                value: self.arena.alloc_new(val), 
                taken_as_inout,
                mappings
            },
            SourceRange::new(start, end)
        ))
    }


    fn block_expression(&mut self) -> ExpressionResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, start, &ParserSettings::default())?;

        Ok(ExpressionNode::new(
            Expression::Block { block },
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn if_expression(&mut self) -> ExpressionResult<'ta> {
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

        
        Ok(ExpressionNode::new(
            Expression::If {
                condition: self.arena.alloc_new(condition), 
                body: self.arena.alloc_new(ExpressionNode::new(
                        Expression::Block { block: body }, body.range()
                )), 
                else_block: else_block.map(|x| &*self.arena.alloc_new(x)),
            },
            SourceRange::new(start, self.current_range().end())
        ))
    }


    fn function_call_expression(&mut self) -> ExpressionResult<'ta> {
        let start = self.current_range().start();
        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let args = self.parse_function_call_args(None)?;
        let end = self.current_range().end();

        Ok(ExpressionNode::new(
            Expression::CallFunction { name, args, is_accessor: false },
            SourceRange::new(start, end)
        ))
        
    }


    fn parse_function_call_args(
        &mut self, 
        associated: Option<ExpressionNode<'ta>>
    ) -> Result<&'ta mut [(ExpressionNode<'ta>, bool)], ErrorId> {

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


            let is_inout = self.is_inout();
            let expr = self.expression(&ParserSettings::default())?;
            self.advance();
            
            args.push((expr, is_inout));
        }
        self.expect(TokenKind::RightParenthesis)?;

        Ok(args.move_into(self.arena).leak())
    }


    fn struct_creation_expression(&mut self) -> ExpressionResult<'ta> {
        let start = self.current_range().start();
        let data_type = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let fields = self.list(TokenKind::RightBracket, Some(TokenKind::Comma), 
        |parser, _| {
            let start = parser.current_range().start();
            let name = parser.expect_identifier()?;
            parser.advance();

            parser.expect(TokenKind::Colon)?;
            parser.advance();

            let expr = parser.expression(&ParserSettings::default())?;
            let end = parser.current_range().end();
            parser.advance();
            
            Ok((name, SourceRange::new(start, end), expr))
        })?;

        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(ExpressionNode::new(
            Expression::CreateStruct { data_type, fields },
            SourceRange::new(start, end),
        ))
    }


    fn cast_expr(&mut self, lhs: ExpressionNode<'ta>) -> ExpressionResult<'ta> {
        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let typ = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::RightParenthesis)?;

        let source = SourceRange::new(lhs.range().start(), self.current_range().end());
        Ok(ExpressionNode::new(
            Expression::CastAny { 
                lhs: self.arena.alloc_new(lhs), 
                data_type: typ 
            },
            source,
        ))
    }
}


impl<'ta> Parser<'_, 'ta, '_> {
    fn binary_operation(
        &mut self,
        lhs: fn(&mut Self, &ParserSettings<'ta>) -> ExpressionResult<'ta>,
        rhs: fn(&mut Self, &ParserSettings<'ta>) -> ExpressionResult<'ta>,
        tokens: &[TokenKind],
        settings: &ParserSettings<'ta>,
    ) -> ExpressionResult<'ta> {
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

            lhs = ExpressionNode::new(
                Expression::BinaryOp { 
                    operator, 
                    lhs: self.arena.alloc_new(lhs), 
                    rhs: self.arena.alloc_new(rhs) 
                }, 
                range,
            )
        }
        
        Ok(lhs)
    }

}
