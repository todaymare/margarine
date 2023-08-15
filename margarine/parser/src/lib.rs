pub mod nodes;

use std::{ops::{Deref, DerefMut}, fmt::Write};

use common::{SymbolMap, SourceRange, SymbolIndex, Slice};
use errors::{Error, CompilerError, ErrorBuilder, ErrorCode, CombineIntoError};
use lexer::{Token, TokenKind, TokenList, Keyword, Literal};
use nodes::{Node, StructKind, NodeKind, Declaration, FunctionArgument, ExternFunction, Expression, BinaryOperator, Statement, EnumMapping};

use crate::nodes::MatchMapping;


#[derive(Debug, Clone, PartialEq)]
pub struct DataType {
    source_range: SourceRange,
    kind: DataTypeKind, 
}


impl DataType {
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn kind(&self) -> &DataTypeKind { &self.kind }
    #[inline(always)]
    pub fn kind_mut(&mut self) -> &mut DataTypeKind { &mut self.kind}
    #[inline(always)]
    pub fn kind_owned(self) -> DataTypeKind { self.kind }

}

impl DataType {
    pub fn new(source_range: SourceRange, kind: DataTypeKind) -> Self { Self { source_range, kind } }
}


#[derive(Debug, Clone, PartialEq)]
pub enum DataTypeKind {
    Int,
    Bool,
    Float,
    Unit,
    Any,
    /// This basically means an error
    /// occurred while trying to find the
    /// type of a node. It bypasses all
    /// type checks for error tolerance reasons.
    Unknown,
    Option(Box<DataType>),
    CustomType(SymbolIndex),
}


impl DataTypeKind {
    pub fn is(&self, oth: &DataTypeKind) -> bool {
           self == &DataTypeKind::Unknown
        || oth == &DataTypeKind::Unknown
        || self == oth
    }
}


/// A wrapper type for Vec<Node> which
/// comes with the guarantee that the vec
/// isn't empty
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    nodes: Vec<Node>,
    source_range: SourceRange,
}


impl Block {
    /// # Panics
    /// if the given vec is empty
    pub fn new(nodes: Vec<Node>, source_range: SourceRange) -> Self {
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


impl Deref for Block {
    type Target = [Node];

    fn deref(&self) -> &Self::Target {
        &self.nodes
    }
}


impl DerefMut for Block {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.nodes
    }
}


pub fn parse(tokens: TokenList, symbol_map: &mut SymbolMap) -> Result<Block, Error> {
    let mut parser = Parser {
        tokens: tokens.as_slice(),
        index: 0,
        file: tokens.file(),
        symbol_map,
    };


    parser.parse_till(TokenKind::EndOfFile, ParserSettings::default())
}


// Internal
type ParseResult = Result<Node, Error>;

#[derive(Clone)]
struct ParserSettings {
    is_in_impl: Option<DataType>,
    can_parse_struct_creation: bool,
}


impl Default for ParserSettings {
    fn default() -> Self {
        Self {
            is_in_impl: None,
            can_parse_struct_creation: true,
        }
    }
}


struct Parser<'a> {
    tokens: &'a [Token],
    index: usize,
    file: SymbolIndex,

    symbol_map: &'a mut SymbolMap,
}


impl Parser<'_> {
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


    fn expect_literal_str(&self) -> Result<SymbolIndex, Error> {
        match self.current_kind() {
            TokenKind::Literal(Literal::String(v)) => Ok(v),
            _ => {
                Err(CompilerError::new(self.file, ErrorCode::PUnexpectedToken, "unexpected token")
                    .highlight(self.current_range())
                        .note(format!("expected a literal string found {:?}", self.current_kind()))
                    .build())
            }
        }
    }


    fn expect_identifier(&self) -> Result<SymbolIndex, Error> {
        match self.current_kind() {
            TokenKind::Identifier(v) => Ok(v),
            _ => {
                Err(CompilerError::new(self.file, ErrorCode::PUnexpectedToken, "unexpected token")
                    .highlight(self.current_range())
                        .note(format!("expected an identifier found {:?}", self.current_kind()))
                    .build())
            }
        }
    }


    fn expect(&self, token_kind: TokenKind) -> Result<&Token, Error> {
        if self.current_kind() != token_kind {
            return Err(CompilerError::new(self.file, ErrorCode::PUnexpectedToken, "unexpected token")
                .highlight(self.current_range())
                    .note(format!("expected {:?} found {:?}", token_kind, self.current_kind()))
                .build())
        }

        Ok(self.current())
    }


    fn expect_multi(&self, token_kinds: &[TokenKind]) -> Result<&Token, Error> {
        if !token_kinds.contains(&self.current_kind()) {
            let message = {
                let mut str = String::new();
                for (i, tk) in token_kinds.iter().enumerate() {
                    let _ = write!(str, "{} {tk:?}", if i == 0 { "" }
                                                else if i == token_kinds.len()-1 { " or" }
                                                else { "," });
                }

                str
            };
            
            return Err(CompilerError::new(self.file, ErrorCode::PUnexpectedToken, "unexpected token")
                .highlight(self.current_range())
                    .note(format!("expected {message} found {:?}", self.current_kind()))
                .build())
        }

        Ok(self.current())
    }


    fn expect_type(&mut self) -> Result<DataType, Error> {
        let start = self.current_range().start();
        let identifier = self.expect_identifier()?;

        let result = match self.symbol_map.get(identifier).as_str() {
            "int" => DataTypeKind::Int,
            "float" => DataTypeKind::Float,
            "bool" => DataTypeKind::Bool,
            "any" => DataTypeKind::Any,
            _ => DataTypeKind::CustomType(identifier),
        };

        let result = DataType::new(
            SourceRange::new(start, self.current_range().end(), self.file), 
            result
        );

        if self.peek_kind() == Some(TokenKind::QuestionMark) {
            self.advance();
            let end = self.current_range().end();
            let result = DataTypeKind::Option(Box::new(result));
            let result = DataType::new(
                SourceRange::new(start, end, self.file),
                result,
            );
            
            return Ok(result)
        }

        Ok(result)
    }


    fn current_is(&self, token_kind: TokenKind) -> bool {
        self.current_kind() == token_kind
    }
}


impl Parser<'_> {
    fn parse_till(&mut self, terminator: TokenKind, settings: ParserSettings) -> Result<Block, Error> {
        let mut storage = Vec::with_capacity(1);
        let mut errors  = Vec::new();
        let mut is_in_panic_mode = false;
        let start = self.current_range().start();

        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            if self.current_kind() == terminator {
                break
            }


            if matches!(self.current_kind(), TokenKind::Keyword(_)) {
                is_in_panic_mode = false;
            }
            

            let statement = self.statement(settings.clone());

            match statement {
                Ok (e) => storage.push(e),
                Err(e) => {
                    if is_in_panic_mode {
                        self.advance();
                        continue
                    }

                    errors.push(e);
                    is_in_panic_mode = true;
                },
            }

            self.advance();
        }

        if !errors.is_empty() {
            return Err(errors.combine_into_error())
        }

        self.expect(terminator)?;        

        let end = self.current_range().end();

        if storage.is_empty() {
            storage.push(Node::new(
                NodeKind::Expression(Expression::Unit),
                SourceRange::new(start, end, self.file)
            ));
        }

        Ok(Block::new(storage, SourceRange::new(start, end, self.file)))
    }


    fn parse_with_tag(
        &mut self, 
        settings: ParserSettings,
        func: fn(&mut Self, ParserSettings) -> ParseResult,
    ) -> ParseResult {
        self.expect(TokenKind::At)?;
        self.advance();

        let tag = self.expect_identifier()?;
        self.advance();

        let mut val = func(self, settings)?;

        val.add_tag(tag);
        Ok(val)
    }
}


impl Parser<'_> {
    fn statement(&mut self, settings: ParserSettings) -> ParseResult {
        match self.current_kind() {
            | TokenKind::Keyword(Keyword::Struct)
            | TokenKind::Keyword(Keyword::Resource)
            | TokenKind::Keyword(Keyword::Component)
            => self.struct_declaration(),


            TokenKind::Keyword(Keyword::Fn) => self.function_declaration(false, settings),
            TokenKind::Keyword(Keyword::System) => {
                self.advance();
                self.function_declaration(true, settings)
            },


            TokenKind::Keyword(Keyword::Impl) => self.impl_declaration(),
            TokenKind::Keyword(Keyword::Mod) => self.mod_declaration(),
            TokenKind::Keyword(Keyword::Extern) => self.extern_declaration(),
            TokenKind::Keyword(Keyword::Enum) => self.enum_declaration(),
            TokenKind::Keyword(Keyword::Using) => self.using_declaration(),


            TokenKind::Keyword(Keyword::Let) => self.let_statement(),


            TokenKind::At => self.parse_with_tag(settings, Self::statement),


            _ => self.assignment(settings),

        }
    }


    fn struct_declaration(&mut self) -> ParseResult {
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
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut fields = vec![];
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

            fields.push((name, datatype, SourceRange::new(start, end, self.file)));
        }
        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Struct { kind, name, fields }),
            SourceRange::new(start, end, self.file),
        ))
    }



    fn function_declaration(&mut self, is_system: bool, settings: ParserSettings) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Fn))?;
        self.advance();

        let (name, is_anonymous)= if let TokenKind::Identifier(v) = self.current_kind() {
            self.advance();
            (v, false)
        } else {
            let ident = self.symbol_map.const_str("anonymous");
            (ident, true)
        };

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let mut arguments = vec![];
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
                    && self.symbol_map.get(name).as_str() == "self" {
                    if let Some(settings) = settings.is_in_impl.clone() {
                        arguments.push(FunctionArgument::new(
                            name,
                            settings,
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
                SourceRange::new(start, end, self.file)
            );

            arguments.push(argument);
        }
        let arguments = arguments;

        self.expect(TokenKind::RightParenthesis)?;
        self.advance();

        let return_type = {
            if self.current_is(TokenKind::Colon) {
                self.advance();

                let typ = self.expect_type()?;
                self.advance();
                typ
            } else {
                DataType::new(
                    SourceRange::new(start, self.current_range().end(), self.file), 
                    DataTypeKind::Unit
                )
            }
        };
        

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Function {
                is_system, 
                is_anonymous,
                name,
                arguments, 
                return_type, 
                body,
            }),

            SourceRange::new(start, end, self.file)
        ))
    }


    fn impl_declaration(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Impl))?;
        self.advance();

        let data_type = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let settings = ParserSettings {
            is_in_impl: Some(data_type.clone()),
            ..Default::default()
        };
        
        let body = self.parse_till(TokenKind::RightBracket, settings)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Impl { 
                data_type, body
            }),

            SourceRange::new(start, end, self.file),
        ))
    }


    fn mod_declaration(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Mod))?;
        self.advance();

        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Module { name, body }),
            SourceRange::new(start, end, self.file)
        ))
    }


    fn extern_declaration(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Extern))?;
        self.advance();

        let file = self.expect_literal_str()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut functions = vec![];
        loop {            
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightBracket {
                break
            }


            self.expect(TokenKind::Keyword(Keyword::Fn))?;
            self.advance();

            let name = self.expect_identifier()?;
            self.advance();

            self.expect(TokenKind::LeftParenthesis)?;
            self.advance();

            let mut arguments = vec![];
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


                if matches!(self.current_kind(), TokenKind::Identifier(_)) {
                    self.advance();
                    self.expect(TokenKind::Colon)?;
                    self.advance();
                }

                let data_type = self.expect_type()?;
                self.advance();
                
                arguments.push(data_type);

            }
            let arguments = arguments;


            self.expect(TokenKind::RightParenthesis)?;
            self.advance();


            let return_type = 
                if self.current_is(TokenKind::Colon) { 
                    self.advance();
                    let typ = self.expect_type()?;
                    self.advance();
                    typ
                }
                else {
                    DataType::new(
                        SourceRange::new(start, self.current_range().end(), self.file), 
                        DataTypeKind::Unit
                    ) 
                };


            functions.push(ExternFunction::new(
                name,
                arguments,
                return_type,
            ));
        }
        let functions = functions;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Extern { file, functions }),
            SourceRange::new(start, end, self.file)
        ))
    }


    fn enum_declaration(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Enum))?;
        self.advance();

        let name = self.expect_identifier()?;
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
            let name = self.expect_identifier()?;

            let (data_type, is_implicit_unit)=
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
            
            let mapping = EnumMapping::new(name, data_type, SourceRange::new(start, end, self.file), is_implicit_unit);
            mappings.push(mapping);
        }
        let mappings = mappings;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Enum { name, mappings }),
            SourceRange::new(start, end, self.file)
        ))
    }


    fn using_declaration(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Using))?;
        self.advance();

        let file = self.expect_literal_str()?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Declaration(Declaration::Using { file }),
            SourceRange::new(start, end, self.file)
        ))
    }


    fn let_statement(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Let))?;
        self.advance();

        let is_mut = 
            if self.current_is(TokenKind::Keyword(Keyword::Mut)) {
                self.advance();
                true
            } else { false };

        
        let name = self.expect_identifier()?;
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

        let expr = self.expression(ParserSettings::default())?;

        Ok(Node::new(
            NodeKind::Statement(Statement::Variable {
                name, 
                hint, 
                is_mut, 
                rhs: Box::new(expr)
            }),
            SourceRange::new(start, self.current_range().end(), self.file)
        ))
    }


    fn assignment(&mut self, settings: ParserSettings) -> ParseResult {
        fn binary_op_assignment(parser: &mut Parser, operator: BinaryOperator, lhs: Node, settings: ParserSettings) -> ParseResult {
            parser.advance();
            parser.advance();

            let rhs = parser.expression(settings)?;
            let range = SourceRange::new(lhs.source_range.start(), parser.current_range().end(), parser.file);

            Ok(Node::new(
                NodeKind::Statement(Statement::UpdateValue { 
                    lhs: Box::new(lhs.clone()),
                    rhs: Box::new(Node::new(
                        NodeKind::Expression(Expression::BinaryOp {
                                operator, 
                                lhs: Box::new(lhs), 
                                rhs: Box::new(rhs)
                            }),
                        range,
                    )) 
                }),
                range
            ))
        }

        
        let start = self.current_range().start();
        let lhs = self.expression(ParserSettings::default())?;


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
                        lhs: Box::new(lhs), 
                        rhs: Box::new(rhs) 
                    }),
                    SourceRange::new(start, self.current_range().end(), self.file)
                ))
            }
            _ => Ok(lhs)
        }
    }
}


impl Parser<'_> {
    fn expression(&mut self, settings: ParserSettings) -> ParseResult {
        self.logical_or(settings)
    }


    fn logical_or(&mut self, settings: ParserSettings) -> ParseResult {
        let lhs = self.logical_and(settings.clone())?;

        if self.peek_kind() != Some(TokenKind::LogicalOr) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.logical_and(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end(), self.file);

        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: Box::new(lhs),
                body: Block::new(vec![
                    Node::new(
                        NodeKind::Expression(Expression::Literal(Literal::Bool(true))),
                        range,
                    )],
                    range),
                else_block: Some(Box::new(Node::new(
                    NodeKind::Expression(Expression::Block {
                            block: Block::new(vec![rhs], 
                            range
                        )
                    }), 
                    range,
                )))
            }),
            range
        ))
    }


    fn logical_and(&mut self, settings: ParserSettings) -> ParseResult {
        let lhs = self.unary_not(settings.clone())?;

        if self.peek_kind() != Some(TokenKind::LogicalAnd) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.unary_not(settings)?;

        let range = SourceRange::new(lhs.range().start(), rhs.range().end(), self.file);

        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: Box::new(lhs),
                body: Block::new(vec![rhs], range),
                else_block: Some(Box::new(Node::new(
                    NodeKind::Expression(Expression::Block {
                            block: Block::new(vec![Node::new(
                                NodeKind::Expression(Expression::Literal(Literal::Bool(false))),
                                range
                            )], 
                            range
                        )
                    }), 
                    range,
                )))
            }),
            range
        ))
    }


    fn unary_not(&mut self, settings: ParserSettings) -> ParseResult {
        if self.current_is(TokenKind::Bang) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.comparisson(settings)?;
            return Ok(Node::new(
                NodeKind::Expression(Expression::UnaryOp { 
                    operator: nodes::UnaryOperator::Not, 
                    rhs: Box::new(expr) 
                }),
                SourceRange::new(start, self.current_range().end(), self.file)
            ))
        }

        self.comparisson(settings)
    }


    fn comparisson(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::bitwise_or, 
            Self::bitwise_or, 
            &[
                TokenKind::LeftAngle, TokenKind::RightAngle,
                TokenKind::GreaterEquals, TokenKind::LesserEquals,
                TokenKind::EqualsTo, TokenKind::NotEqualsTo,
            ], 
            settings,
        )
    }
    

    fn bitwise_or(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::bitwise_xor, 
            Self::bitwise_xor, 
            &[TokenKind::BitwiseOr], 
            settings,
        )
        
    }


    fn bitwise_xor(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::bitwise_and, 
            Self::bitwise_and, 
            &[TokenKind::BitwiseXor], 
            settings,
        )
        
    }


    fn bitwise_and(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::bitshifts, 
            Self::bitshifts, 
            &[TokenKind::Ampersand], 
            settings,
        )
        
    }
    

    fn bitshifts(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::arithmetic, 
            Self::arithmetic, 
            &[TokenKind::BitshiftLeft, TokenKind::BitshiftRight], 
            settings,
        )
        
    }
    

    fn arithmetic(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::product, 
            Self::product, 
            &[TokenKind::Plus, TokenKind::Minus], 
            settings,
        )
    }


    fn product(&mut self, settings: ParserSettings) -> ParseResult {
        self.binary_operation(
            Self::unary_neg, 
            Self::unary_neg, 
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent], 
            settings,
        )
    }
    

    fn unary_neg(&mut self, settings: ParserSettings) -> ParseResult {
        if self.current_is(TokenKind::Minus) {
            let start = self.current_range().start();
            self.advance();
            let expr = self.accessors(settings)?;
            return Ok(Node::new(
                NodeKind::Expression(Expression::UnaryOp { 
                    operator: nodes::UnaryOperator::Neg, 
                    rhs: Box::new(expr) 
                }),
                SourceRange::new(start, self.current_range().end(), self.file)
            ))
        }

        self.accessors(settings)
    }


    fn accessors(&mut self, settings: ParserSettings) -> ParseResult {
        let mut result = self.atom(settings)?;
        println!("\n\n\n{:?}", self.peek_kind());

        while self.peek_kind() == Some(TokenKind::Dot) {
            self.advance();
            self.advance();
            
            let start = self.current_range().start();
            let ident = self.expect_identifier()?;

            if self.peek_kind() == Some(TokenKind::LeftParenthesis) {
                self.advance();
                self.advance();

                let args = self.parse_function_call_args()?;
                self.expect(TokenKind::RightParenthesis)?;

                result = Node::new(
                    NodeKind::Expression(Expression::CallFunction {
                        name: ident, 
                        args,
                        is_accessor: Some(Box::new(result)), 
                    }),
                    SourceRange::new(start, self.current_range().end(), self.file)
                )
            } else {
                result = Node::new(
                    NodeKind::Expression(Expression::AccessField { 
                        val: Box::new(result), 
                        field: ident,
                    }),
                    SourceRange::new(start, self.current_range().end(), self.file)
                )
            }
        }

        Ok(result)
    }
    

    fn atom(&mut self, settings: ParserSettings) -> ParseResult {
        match self.current_kind() {
            TokenKind::Literal(l) => Ok(Node::new(
                NodeKind::Expression(Expression::Literal(l)), 
                self.current_range(),
            )),


            TokenKind::LeftParenthesis => {
                let start = self.current_range().start();
                self.advance();

                if self.current_is(TokenKind::RightParenthesis) {
                     return Ok(Node::new(
                        NodeKind::Expression(Expression::Unit), 
                        self.current_range(),
                    ))       
                }

                let mut expr = self.expression(ParserSettings::default())?;
                self.advance();

                self.expect(TokenKind::RightParenthesis)?;

                expr.source_range = SourceRange::new(start, self.current_range().end(), self.file);
                
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
                            action: Box::new(expr),
                            namespace_source: source,
                        }),
                        SourceRange::new(start, self.current_range().end(), self.file)
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

            
            _ => Err(CompilerError::new(self.file, ErrorCode::PUnexpectedToken, "unexpected token")
                .highlight(self.current_range())
                    .note(format!("'{:?}'", self.current_kind()))
                .build())
        }
    }


    fn binary_operation(
        &mut self,
        lhs: fn(&mut Self, ParserSettings) -> ParseResult,
        rhs: fn(&mut Self, ParserSettings) -> ParseResult,
        tokens: &[TokenKind],
        settings: ParserSettings,
    ) -> ParseResult {
        let mut lhs = lhs(self, settings.clone())?;

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

            
            let rhs = rhs(self, settings.clone())?;

            let range = SourceRange::new(
                lhs.range().start(), 
                rhs.range().end(),
                self.file,
            );

            lhs = Node::new(
                NodeKind::Expression(Expression::BinaryOp { 
                    operator, 
                    lhs: Box::new(lhs), 
                    rhs: Box::new(rhs) 
                }), 
                range,
            )
        }
        
        Ok(lhs)
    }


    fn match_expression(&mut self, settings: ParserSettings) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Match))?;
        self.advance();

        let val = {
            let settings = ParserSettings {
                can_parse_struct_creation: false,
                ..settings
            };

            self.expression(settings)?
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
            let name = self.expect_identifier()?;
            let source_range = SourceRange::new(start, self.current_range().end(), self.file);
            self.advance();

            let bind_to =
                if self.current_is(TokenKind::Colon) {
                    self.advance();
                    let name = self.expect_identifier()?;
                    self.advance();
                    name
                } else {
                    self.symbol_map.const_str("_")
                };


            self.expect(TokenKind::Arrow)?;
            self.advance();

            let expr = self.expression(ParserSettings::default())?;
            self.advance();

            mappings.push(MatchMapping::new(name, bind_to, source_range, expr));
        }
        let mappings = mappings;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::Match { 
                value: Box::new(val), 
                mappings
            }),
            SourceRange::new(start, end, self.file)
        ))
    }


    fn block_expression(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let block = self.parse_till(TokenKind::RightBracket, ParserSettings::default())?;

        Ok(Node::new(
            NodeKind::Expression(Expression::Block { block }),
            SourceRange::new(start, self.current_range().end(), self.file)
        ))
    }


    fn if_expression(&mut self) -> ParseResult {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::If))?;
        self.advance();

        let condition = self.expression(ParserSettings { can_parse_struct_creation: false, ..Default::default()})?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till(TokenKind::RightBracket, ParserSettings::default())?;

        let else_block = 
            if self.peek_kind() == Some(TokenKind::Keyword(Keyword::Else)) {
                self.advance();
                self.advance();

                Some(Box::new(if self.current_is(TokenKind::Keyword(Keyword::If)) {
                    self.if_expression()?
                } else {
                    self.block_expression()?
                }))
            } else { None };
        
        Ok(Node::new(
            NodeKind::Expression(Expression::If {
                condition: Box::new(condition), 
                body, 
                else_block,
            }),
            SourceRange::new(start, self.current_range().end(), self.file)
        ))
    }


    fn function_call_expression(&mut self) -> ParseResult {
        let start = self.current_range().start();
        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let args = self.parse_function_call_args()?;

        self.expect(TokenKind::RightParenthesis)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::CallFunction { name, args, is_accessor: None }),
            SourceRange::new(start, end, self.file)
        ))
        
    }


    fn parse_function_call_args(&mut self) -> Result<Vec<(Node, bool)>, Error> {        
        let mut args = vec![];
        loop {
            if self.current_kind() == TokenKind::EndOfFile {
                break
            }

            
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            if !args.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }

            
            // To allow for trailing commas
            if self.current_kind() == TokenKind::RightParenthesis {
                break
            }


            let is_inout = if self.current_is(TokenKind::Ampersand) { self.advance(); true }
                            else { false };
            let expr = self.expression(ParserSettings::default())?;
            self.advance();
            
            args.push((expr, is_inout));
        }
        Ok(args)
    }


    fn struct_creation_expression(&mut self) -> ParseResult {
        let start = self.current_range().start();
        let data_type = self.expect_type()?;
        self.advance();

        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let mut fields = vec![];
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

            let expr = self.expression(ParserSettings::default())?;
            let end = self.current_range().end();
            self.advance();
            
            fields.push((name, SourceRange::new(start, end, self.file), expr));
        }
        let fields = fields;

        self.expect(TokenKind::RightBracket)?;
        let end = self.current_range().end();

        Ok(Node::new(
            NodeKind::Expression(Expression::CreateStruct { data_type, fields }),
            SourceRange::new(start, end, self.file),
        ))
    }
}
