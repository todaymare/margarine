pub mod nodes;
pub mod errors;
pub mod dt;

use common::{source::SourceRange, string_map::{StringIndex, StringMap}};
use dt::{DataType, DataTypeKind};
use errors::Error;
use ::errors::{ParserError, ErrorId};
use lexer::{Token, TokenKind, TokenList, Keyword, Literal};
use nodes::{decl::{Decl, DeclId, EnumMapping, ExternFunction, FunctionArgument, FunctionSignature, UseItem, UseItemKind}, expr::{Block, Expr, MatchMapping, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, AST};
use sti::{arena::Arena, vec::Vec, keyed::KVec};

use crate::nodes::expr::{BinaryOperator, ExprId};

pub fn parse<'a>(
    tokens: TokenList, 
    file: u32,
    arena: &'a Arena, 
    string_map: &mut StringMap,
    ast: &mut AST<'a>,
) -> (Block<'a>, KVec<ParserError, Error>) {

    let mut parser = Parser {
        tokens: &*tokens,
        index: 0,
        string_map,
        arena,
        errors: KVec::new(),
        is_in_panic: false,
        file,
        ast,
    };


    let result = parser.parse_till_decl(
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


struct Parser<'me, 'ast, 'str> {
    tokens: &'me [Token],
    index: usize,
    file: u32,

    arena: &'ast Arena,
    ast: &'me mut AST<'ast>,
    string_map: &'me mut StringMap<'str>,

    errors: KVec<ParserError, Error>,
    is_in_panic: bool,
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
        } else if self.current_is(TokenKind::LeftParenthesis) { 
            self.advance();
            if self.current_is(TokenKind::RightParenthesis) {
                DataType::new(self.current_range(), DataTypeKind::Unit)
            } else {
                let start = self.current_range().start();
                let pool = Arena::tls_get_rec();

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

                    let name = if matches!(self.current_kind(), TokenKind::Identifier(_)) 
                                  && self.peek_is(TokenKind::Colon) {
                        let ident = self.expect_identifier()?;
                        self.advance();

                        self.expect(TokenKind::Colon)?;
                        self.advance();

                        ident.some()
                    } else { None.into() };

                    let typ = self.expect_type()?;
                    vec.push((name, typ));
                    self.advance();
                }

                self.expect(TokenKind::RightParenthesis)?;

                DataType::new(
                    SourceRange::new(start, self.current_range().end()),
                    DataTypeKind::Tuple(vec.move_into(self.arena).leak())
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

                        
                        if self.current_is(TokenKind::RightParenthesis) {
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
        mut func: impl FnMut(&mut Self, usize) -> Result<T, ErrorId>,
    ) -> Result<&'out [T], ErrorId> {
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
        settings: &ParserSettings<'out>
    ) -> Result<Block<'out>, ErrorId> {

        let mut storage : Vec<NodeId, _> = Vec::with_cap_in(self.arena, 1);

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
        settings: &ParserSettings<'out>
    ) -> Result<Block<'out>, ErrorId> {
        let parse_till = self.parse_till(terminator, start, settings)?;

        for node in parse_till.into_iter() {
            if !matches!(node, NodeId::Decl(_)) {
                self.errors.push(Error::DeclarationOnlyBlock { source: self.ast.range(*node) });
                continue;
            };
        }

        Ok(parse_till)
    }


    fn generic_decl(&mut self) -> Result<&'out [StringIndex], ErrorId> {
        if !self.current_is(TokenKind::LeftAngle) {
            return Ok(&[]);
        }

        self.advance();
        let list = self.list(TokenKind::RightAngle, Some(TokenKind::Comma),
        |slf, _| {
            let ident = slf.expect_identifier()?;
            Ok(ident)
        })?;
        self.advance();

        Ok(list)
    }
}

impl<'ta> Parser<'_, 'ta, '_> {
    fn statement(&mut self, settings: &ParserSettings<'ta>) -> Result<NodeId, ErrorId> {
        let node = match self.current_kind() {
            | TokenKind::Keyword(Keyword::Struct)
            => self.struct_declaration()?.into(),


            TokenKind::Keyword(Keyword::Fn) => self.function_declaration(&settings)?.into(),

            TokenKind::Keyword(Keyword::Impl) => self.impl_declaration()?.into(),
            TokenKind::Keyword(Keyword::Mod) => self.mod_declaration()?.into(),
            TokenKind::Keyword(Keyword::Extern) => self.extern_declaration(settings)?.into(),
            TokenKind::Keyword(Keyword::Enum) => self.enum_declaration()?.into(),
            TokenKind::Keyword(Keyword::Use) => self.using_declaration()?.into(),


            TokenKind::Keyword(Keyword::Var) => self.let_statement()?.into(),
            TokenKind::Keyword(Keyword::For) => self.for_statement()?.into(),


            TokenKind::At => {
                let start = self.current_range().start();
                self.advance();
                let ident = self.expect_identifier()?;
                let attr_range = SourceRange::new(start, self.current_range().end());
                self.advance();

                let stmt = self.statement(settings)?;

                let NodeId::Decl(decl) = stmt
                else { return Ok(stmt.into()) };


                self.ast.add_decl(Decl::Attribute { attr: ident, decl, attr_range },
                                SourceRange::new(start, self.current_range().end())).into()
            },

            _ => self.assignment(&settings)?.into(),
        };

        Ok(node)
    }


    fn struct_declaration(&mut self) -> DeclResult<'ta> {
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

        let node = Decl::Struct { name, header, fields, generics };

        Ok(self.ast.add_decl(node, SourceRange::new(start, end)))
    }



    fn function_declaration(
        &mut self, 
        settings: &ParserSettings<'ta>
    ) -> DeclResult<'ta> {

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
            let name = parser.expect_identifier()?;

            if index == 0
                && name == StringMap::SELF {
                if let Some(parser_type) = settings.is_in_impl {
                    return Ok(FunctionArgument::new(
                        name,
                        parser_type,
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

        Ok(self.ast.add_decl(
            Decl::Function {
                sig: FunctionSignature::new(
                     name, 
                     header,
                     arguments,
                     generics,
                     return_type,
                ),
                body,
                is_in_impl: settings.is_in_impl,
            },

            SourceRange::new(start, end)
        ))
    }


    fn impl_declaration(&mut self) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Impl))?;
        self.advance();

        let gens = self.generic_decl()?;

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

        Ok(self.ast.add_decl(
            Decl::Impl { 
                data_type, body,
                gens,
            },

            SourceRange::new(start, end),
        ))
    }


    fn mod_declaration(&mut self) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Mod))?;
        self.advance();

        let name = self.expect_identifier()?;
        let header_end = self.current_range().end();
        self.advance();

        let body_start = self.current_range().start();
        self.expect(TokenKind::LeftBracket)?;
        self.advance();

        let body = self.parse_till_decl(TokenKind::RightBracket, body_start, &ParserSettings::default())?;
        let end = self.current_range().end();

        Ok(self.ast.add_decl(
            Decl::Module { name, body, header: SourceRange::new(start, header_end) },
            SourceRange::new(start, end)
        ))
    }


    fn extern_declaration(&mut self, settings: &ParserSettings<'ta>) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Extern))?;
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

                let identifier = parser.expect_identifier()?;

                if index == 0
                    && identifier == StringMap::SELF {
                    if let Some(parser_type) = settings.is_in_impl {
                        return Ok(FunctionArgument::new(
                            name,
                            parser_type,
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

        Ok(self.ast.add_decl(
            Decl::Extern { functions },
            SourceRange::new(start, end)
        ))
    }


    fn enum_declaration(&mut self) -> DeclResult<'ta> {
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

        Ok(self.ast.add_decl(
            Decl::Enum { name, mappings, header, generics },
            SourceRange::new(start, end)
        ))
    }


    fn using_declaration(&mut self) -> DeclResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::Use))?;
        self.advance();

        let item = self.parse_use_item()?;

        Ok(self.ast.add_decl(
            Decl::Using { item },
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
                                        |parser, _| parser.parse_use_item())?;

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
            Ok(UseItemKind::BringName)

        };

        let item = func()?;

        Ok(UseItem::new(ident, item, SourceRange::new(start, self.current_range().end())))
    }


    fn for_statement(&mut self) -> StmtResult<'ta> {
        let start = self.current_range().start();
        self.expect(TokenKind::Keyword(Keyword::For))?;
        self.advance();

        let binding_start = self.current_range().start();

        let binding = self.expect_identifier()?;
        let binding_range = SourceRange::new(binding_start, self.current_range().end());
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
                binding: (binding, binding_range),
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

        let pool = Arena::tls_get_temp();
        let mut bindings = Vec::new_in(&*pool);
        loop {
            if !bindings.is_empty() {
                self.expect(TokenKind::Comma)?;
                self.advance();
            }
            

            let name = self.expect_identifier()?;
            self.advance();
            
            bindings.push(name);
            if self.current_is(TokenKind::Equals) || self.current_is(TokenKind::Colon) {
                break
            }

            // @todo: enable tuple var decls
            break
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
        
        Ok(self.ast.add_stmt(if bindings.len() == 1 {
            let b = bindings[0];
            Stmt::Variable { name: b, hint, rhs }
        } else {
            Stmt::VariableTuple {
                names: bindings.move_into(self.arena).leak(), 
                hint, rhs
            }
        }, source))
        
    }



    fn assignment(&mut self, settings: &ParserSettings<'ta>) -> Result<NodeId, ErrorId> {
        fn binary_op_assignment<'la>(
            parser: &mut Parser<'_, 'la, '_>, 
            operator: BinaryOperator, 
            lhs: ExprId, 
            settings: &ParserSettings<'la>
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
    fn expression(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.logical_or(settings)
    }


    fn logical_or(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        let lhs = self.logical_and(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalOr) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.logical_and(settings)?;

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


    fn logical_and(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        let lhs = self.unary_not(settings)?;

        if self.peek_kind() != Some(TokenKind::LogicalAnd) {
            return Ok(lhs)
        }
        self.advance();
        self.advance();

        let rhs = self.unary_not(settings)?;

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


    fn unary_not(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
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


    fn comparisson(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
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
    

    fn bitwise_or(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitwise_xor, 
            Self::bitwise_xor, 
            &[TokenKind::BitwiseOr], 
            settings,
        )
        
    }


    fn bitwise_xor(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitwise_and, 
            Self::bitwise_and, 
            &[TokenKind::BitwiseXor], 
            settings,
        )
        
    }


    fn bitwise_and(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::bitshifts, 
            Self::bitshifts, 
            &[TokenKind::Ampersand], 
            settings,
        )
        
    }
    

    fn bitshifts(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::arithmetic, 
            Self::arithmetic, 
            &[TokenKind::BitshiftLeft, TokenKind::BitshiftRight], 
            settings,
        )
        
    }
    

    fn arithmetic(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::product, 
            Self::product, 
            &[TokenKind::Plus, TokenKind::Minus], 
            settings,
        )
    }


    fn product(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.binary_operation(
            Self::range_expr, 
            Self::range_expr, 
            &[TokenKind::Star, TokenKind::Slash, TokenKind::Percent], 
            settings,
        )
    }

    fn range_expr(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        let lhs = self.unary_neg(settings)?;

        if !self.peek_is(TokenKind::DoubleDot) {
            return Ok(lhs);
        }
        self.advance();

        let is_inc = if self.peek_is(TokenKind::Equals) { self.advance(); true }
                     else { false };
        self.advance();

        let mut rhs = self.unary_neg(settings)?;
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
    

    fn unary_neg(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
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


    fn as_cast(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
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


    fn accessors(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        let mut result = self.atom(settings)?;

        while 
            self.peek_kind() == Some(TokenKind::Dot) 
            || self.peek_kind() == Some(TokenKind::Bang)
            || self.peek_kind() == Some(TokenKind::QuestionMark) {
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
            
            self.advance();
            
            let start = self.current_range().start();
            let ident = match self.current_kind() {
                TokenKind::Literal(Literal::Integer(int)) => self.string_map.num(int as usize),

                _ => self.expect_identifier()?,
            };

            if self.peek_is(TokenKind::LeftParenthesis) {
                self.advance();
                self.advance();

                let args = self.parse_function_call_args(Some(result))?;

                result = self.ast.add_expr(
                    Expr::CallFunction {
                        name: ident, 
                        args,
                        is_accessor: true, 
                    },
                    SourceRange::new(start, self.current_range().end())
                )

                
            } else {
                result = self.ast.add_expr(
                    Expr::AccessField { 
                        val: result, 
                        field_name: ident,
                    },
                    SourceRange::new(start, self.current_range().end())
                )
            }
        }

        Ok(result)
    }
    

    fn atom(&mut self, settings: &ParserSettings<'ta>) -> ExprResult<'ta> {
        self.is_error_token()?;

        match self.current_kind() {
            TokenKind::Literal(l) => Ok(self.ast.add_expr(
                Expr::Literal(l), 
                self.current_range(),
            )),

            TokenKind::Underscore => {
                 return Ok(self.ast.add_expr(
                    Expr::Unit, 
                    self.current_range(),
                ))
            }


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
                    let pool = Arena::tls_get_rec();
                    let mut vec = Vec::new_in(&*pool);
                    vec.push(expr);
                    while self.current_is(TokenKind::Comma) {
                        self.advance();
                        if self.current_is(TokenKind::RightParenthesis) { break }

                        vec.push(self.expression(&ParserSettings::default())?);
                        self.advance();
                    }
                    self.expect(TokenKind::RightParenthesis)?;
                    return Ok(self.ast.add_expr(
                        Expr::Tuple(vec.move_into(self.arena).leak()), 
                        SourceRange::new(start, self.current_range().end())
                    ));
                }

                self.expect(TokenKind::RightParenthesis)?;

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
                    ) {
                    return self.struct_creation_expression()
                }


                if self.peek_kind() == Some(TokenKind::DoubleColon) {
                    let source = self.current_range();
                    let start = self.current_range().start();

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
                    Expr::Identifier(v),
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

                let typ = self.expect_type()?;
                self.advance();

                self.expect(TokenKind::RightSquare)?;
                self.advance();

                self.expect(TokenKind::DoubleColon)?;
                self.advance();

                let expr = self.expression(&ParserSettings::default())?;

                Ok(self.ast.add_expr(
                    Expr::WithinTypeNamespace { 
                        namespace: typ, 
                        action: expr 
                    },
                    SourceRange::new(start, self.current_range().end())
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
            let source_range = SourceRange::new(start, parser.current_range().end());
            parser.advance();

            let (bind_to, binding_range) =
                if parser.current_is(TokenKind::Colon) {
                    parser.advance();

                    let binding_start = parser.current_range().start();

                    let name = parser.expect_identifier()?;
                    let binding_range = SourceRange::new(binding_start, parser.current_range().end());
                    parser.advance();
                    (name, binding_range)

                } else {
                    (parser.string_map.insert("_"), parser.current_range())
                };


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


    fn function_call_expression(&mut self) -> ExprResult<'ta> {
        let start = self.current_range().start();
        let name = self.expect_identifier()?;
        self.advance();

        self.expect(TokenKind::LeftParenthesis)?;
        self.advance();

        let args = self.parse_function_call_args(None)?;
        let end = self.current_range().end();

        Ok(self.ast.add_expr(
            Expr::CallFunction { name, args, is_accessor: false },
            SourceRange::new(start, end)
        ))
        
    }


    fn parse_function_call_args(
        &mut self, 
        associated: Option<ExprId>
    ) -> Result<&'ta mut [ExprId], ErrorId> {

        let binding = Arena::tls_get_rec();
        let mut args = Vec::new_in(&*binding);

        if let Some(node) = associated {
            args.push(node);
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


            let expr = self.expression(&ParserSettings::default())?;
            self.advance();
            
            args.push(expr);
        }
        self.expect(TokenKind::RightParenthesis)?;

        Ok(args.move_into(self.arena).leak())
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
        lhs: fn(&mut Self, &ParserSettings<'ta>) -> ExprResult<'ta>,
        rhs: fn(&mut Self, &ParserSettings<'ta>) -> ExprResult<'ta>,
        tokens: &[TokenKind],
        settings: &ParserSettings<'ta>,
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
