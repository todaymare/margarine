#![feature(if_let_guard)]
#![deny(unused_must_use)]

use common::{string_map::{StringIndex, StringMap}, source::SourceRange, fuck_map::FuckMap, OptionalPlus};
use errors::Error;
use ::errors::{SemaError, ErrorId};
use ir::terms::{Block, EnumVariant, Reg, BlockId, IR, StrConstId};
use lexer::Literal;
use parser::nodes::{Node, NodeKind, Declaration, Statement, Expression, BinaryOperator, UnaryOperator};
use sema::{InferState, Namespace, NamespaceId, Scope, ScopeId, ScopeKind};
use sti::{keyed::KVec, vec::Vec, define_key, prelude::Arena, arena_pool::ArenaPool, packed_option::PackedOption};

use crate::ir::terms::Terminator;

pub mod symbol_vec;
pub mod errors;
pub mod sema;
pub mod ir;


pub fn semantic_analysis<'me, 'at, 'af, 'an>(
    arena_type: &'at Arena,
    arena_func: &'af Arena,
    arena_nasp: &'an Arena,
    string_map: &'me mut StringMap,

    block: &[Node],
) -> State<'me, 'at, 'af, 'an> {
    let mut state = State::new(
        arena_type,
        arena_func,
        arena_nasp,
        string_map,
    );

    let temp_arena = ArenaPool::tls_get_temp();
    let mut anal = LocalAnalyser {
        arena: &*temp_arena,
        fc: FunctionCounter::new(),
        blocks: Vec::new_in(&temp_arena),
        current: Block {
            id: BlockId(0),
            body: Vec::new_in(&temp_arena),
            terminator: ir::terms::Terminator::Ret,
        },
    };
    
    'scope: {
        let scope = {
            let pool = ArenaPool::tls_get_temp();
            let mut ns = Namespace::new(&*pool);

            if state.collect_names(&mut anal, &mut ns, block).is_none() {
                break 'scope;
            }

            let ns = ns.move_into(state.sema.arena_nasp);
            let ns = state.sema.create_ns(ns);

            let scope = Scope::new(PackedOption::NONE, sema::ScopeKind::Namespace(ns));
            if state.update_types(&mut anal, &scope, block).is_none() {
                break 'scope;
            }

            scope
        };


        let scope = state.sema.scopes.push(scope);

        for node in block {
            match node.kind() {
                NodeKind::Declaration(decl) => {
                    let Err(e) = state.declaration(scope, &decl)
                    else { continue };

                    anal.current.push(IR::Error(ErrorId::Sema(e)));
                },

                _ => (),
            }
        }
    }
    
    state
}


define_key!(u32, pub TypeId);
define_key!(u32, pub FuncId);


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    UserType(TypeId),
    Str,
    Int,
    Bool,
    Float,
    Unit,
    Any,
    Never,

    Error,
}


impl Type {
    pub fn to_string<'me>(
        self, 
        types: &KVec<TypeId, TypeSymbol>, 
        string_map: &'me StringMap
    ) -> &'me str {

        match self {
            Type::UserType(v) => string_map.get(types.get(v).unwrap().name),
            Type::Str => "str",
            Type::Int => "int",
            Type::Bool => "bool",
            Type::Float => "float",
            Type::Unit => "unit",
            Type::Any => "any",
            Type::Never => "never",
            Type::Error => "error",
        }
    }


    fn is(self, other: Self) -> bool {
        match (self, other) {
            | (Type::Str, Type::Str)
            | (Type::Int, Type::Int)
            | (Type::Bool, Type::Bool)
            | (Type::Float, Type::Float)
            | (Type::Unit, Type::Unit)
            | (Type::Any, Type::Any)
            | (Type::Never, _)
            | (_, Type::Never)
            | (Type::Error, _)
            | (_, Type::Error)
             => true,

            (Type::UserType(v1), Type::UserType(v2)) => v1 == v2,

            _ => false,
        }
    }
}


#[derive(Debug)]
pub struct State<'me, 'at, 'af, 'an> {
    arena_type: &'at Arena,
    arena_func: &'af Arena,

    pub string_map: &'me mut StringMap,
    
    pub types: KVec<TypeId, TypeSymbol<'at>>,
    pub funcs: KVec<FuncId, Option<Function<'af>>>,

    sema: InferState<'an>,
    str_consts: FuckMap<StringIndex, StrConstId>,

    pub errors: KVec<SemaError, Error>,
}


#[derive(Debug)]
pub struct TypeSymbol<'a> {
    name: StringIndex,
    source: SourceRange,
    kind: TypeSymbolKind<'a>,
}


#[derive(Debug)]
enum TypeSymbolKind<'a> {
    Structure {
        kind: StructureKind,
        fields: &'a [Type],
    },

    Enum {
        mappings: &'a [(StringIndex, Type, SourceRange, bool)],
    },

    BuiltIn,
    Unknown,
}


#[derive(Debug)]
enum StructureKind {
    Normal,
    Component,
    Resource,
}


#[derive(Debug)]
pub struct Function<'a> {
    args: &'a [(StringIndex, Type, bool, SourceRange)],
    body: Vec<Block<'a>, &'a Arena>,
    return_type: Type,
}


struct FunctionCounter {
    regs: usize,
    blocks: u32,
}


impl FunctionCounter {
    pub fn new() -> Self {
        Self {
            regs: 0,
            blocks: 0,
        }
    }


    pub fn new_reg(&mut self) -> Reg {
        self.regs += 1;
        Reg(self.regs - 1)
    }


    pub fn new_block<'a>(&mut self, body: Vec<IR<'a>, &'a Arena>) -> Block<'a> {
        self.blocks += 1;
        Block {
            id: BlockId(self.blocks-1),
            body,
            terminator: ir::terms::Terminator::Ret,
        }
    }
}


pub struct LocalAnalyser<'a> {
    arena: &'a Arena,

    fc: FunctionCounter,
    blocks: Vec<Block<'a>, &'a Arena>,
    current: Block<'a>,
}

impl<'a> LocalAnalyser<'a> {
    pub fn new(arena: &'a Arena) -> Self { 
        let mut fc = FunctionCounter::new();
        let block = fc.new_block(Vec::new_in(arena));
        Self {
            arena, 
            fc, 
            blocks: Vec::new_in(arena), 
            current: block,
        } 
    }


    pub fn error(&mut self, error: ErrorId) -> AnalysisResult {
        self.current.push(IR::Error(error));
        AnalysisResult::new(self.fc.new_reg(), Type::Error)
    }


    pub fn empty_error(&mut self) -> AnalysisResult {
        AnalysisResult::new(self.fc.new_reg(), Type::Error)
    }
}


pub struct AnalysisResult {
    reg: Reg,
    typ: Type,
}

impl AnalysisResult {
    pub fn new(reg: Reg, typ: Type) -> Self { Self { reg, typ } }
}


impl<'me, 'at, 'af, 'an> State<'me, 'at, 'af, 'an> {
    pub fn new(
        arena_type: &'at Arena,
        arena_func: &'af Arena,
        arena_nasp: &'an Arena,

        string_map: &'me mut StringMap,
    ) -> Self {
        Self {
            arena_type,
            arena_func,
            string_map,

            types: KVec::new(),
            funcs: KVec::new(),
            sema: InferState {
                arena_nasp,

                namespaces: KVec::new(),
                scopes: KVec::new(),
                option_table: FuckMap::new(),
                result_table: FuckMap::new(),
                namespace_table: FuckMap::new(),
            },
            str_consts: FuckMap::new(),

            errors: KVec::new(),
        }
    }

    
    #[inline(always)]
    fn declare_type(&mut self, source: SourceRange, name: StringIndex) -> TypeId {
        self.types.push(TypeSymbol {
            name,
            source,
            kind: TypeSymbolKind::BuiltIn,
        })
    }


    #[inline(always)]
    fn declare_func(&mut self) -> FuncId {
        self.funcs.push(None)
    }


    #[inline(always)]
    fn create_func(&mut self, func: Function<'af>) -> FuncId {
        let id = self.declare_func();
        self.funcs.get_mut(id).unwrap().replace(func);
        id
    }

    
    #[inline(always)]
    fn update_type(&mut self, index: TypeId, symbol: TypeSymbolKind<'at>) {
        let type_symbol = self.types.get_mut(index).unwrap();
        type_symbol.kind = symbol;
    }


    pub fn block(
        &mut self, 
        anal: &mut LocalAnalyser,
        scope: ScopeId,
        nodes: &[Node]
    ) -> AnalysisResult {

        let scope = {
            let pool = ArenaPool::tls_get_temp();
            let mut ns = Namespace::new(&*pool);

            if self.collect_names(anal, &mut ns, nodes).is_none() {
                return AnalysisResult::new(anal.fc.new_reg(), Type::Never)
            }

            let ns = ns.move_into(self.sema.arena_nasp);
            let ns = self.sema.create_ns(ns);

            let scope = Scope::new(scope.some(), sema::ScopeKind::Namespace(ns));
            if self.update_types(anal, &scope, nodes).is_none() {
                return AnalysisResult::new(anal.fc.new_reg(), Type::Never)
            }

            scope
        };


        let scope = self.sema.scopes.push(scope);

        for node in nodes {
            match node.kind() {
                NodeKind::Declaration(decl) => {
                    let Err(e) = self.declaration(scope, decl)
                    else { continue };

                    anal.current.push(IR::Error(ErrorId::Sema(e)));
                },

                _ => (),
            }
        }
        
        let mut scope = scope;

        let mut final_type = Type::Unit;
        let mut final_reg = anal.fc.new_reg();

        for node in nodes {
            let result = match node.kind() {
                NodeKind::Declaration(_) => continue,

                _ => self.node(anal, &mut scope, node),
            };

            final_type = result.typ;
            final_reg = result.reg;
        }
        

        AnalysisResult::new(final_reg, final_type)
    }

}


impl<'me, 'at, 'af, 'an> State<'me, 'at, 'af, 'an> {
    pub fn node(
        &mut self,
        anal: &mut LocalAnalyser,
        scope: &mut ScopeId,
        node: &Node
    ) -> AnalysisResult {
        match node.kind() {
            NodeKind::Declaration(_) => unreachable!(),

            NodeKind::Statement(stmt) => {
                self.stmt(anal, scope, stmt);
                AnalysisResult::new(anal.fc.new_reg(), Type::Unit)
            },

            NodeKind::Expression(expr) => self.expression(anal, scope, expr, node.range()),

            NodeKind::Error(v) => anal.error(*v),
        }
    }

    
    pub fn declaration(
        &mut self, 
        scope: ScopeId, 
        decl: &Declaration
    ) -> Result<(), SemaError> {

        match decl {
            Declaration::Struct { .. } => Ok(()),
            Declaration::Enum { .. } => Ok(()),

            Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                let index = {
                    let current = self.sema.scopes.get(scope).unwrap();
                    let index = current.find_func(
                        *name, 
                        &self.sema.scopes, 
                        &self.sema.namespaces
                    ).unwrap();
                    index
                };

                let mut fc = FunctionCounter::new();

                let return_reg = fc.new_reg();
                assert_eq!(return_reg.0, 0);

                let scope = Scope::new(scope.some(), sema::ScopeKind::Function);
                let mut scope = self.sema.scopes.push(scope);
                let func = self.funcs.get(index).unwrap().unwrap_ref();

                for arg in func.args.iter() {
                    scope = self.sema.scopes.push(Scope::new(
                        scope.some(),
                        sema::ScopeKind::Variable((
                            arg.0,
                            arg.1,
                            arg.2,
                            fc.new_reg(),
                        ))
                    ));
                }

                let mut anal = LocalAnalyser::new(self.arena_func);
                
                let analysis = {
                    let block_anal = self.block(
                        &mut anal, 
                        scope, 
                        &body,
                    );

                    anal.current.push(IR::Copy { dst: Reg(0), src: block_anal.reg });

                    block_anal
                };

                let func = self.funcs.get_mut(index).unwrap().as_mut().unwrap();

                if !analysis.typ.is(func.return_type) {
                    anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                        Error::FunctionBodyAndReturnMismatch {
                            header: *header, 
                            item: body.last().unwrap().range(),
                            return_type: func.return_type, 
                            body_type: analysis.typ,
                        }
                    ))));
                }

                let mut ir_body = std::mem::replace(&mut anal.blocks, Vec::new_in(self.arena_func));
                ir_body.push(anal.current);
                ir_body.sort_unstable_by_key(|b| b.id.0);

                func.body = ir_body;

                Ok(())
            },

            Declaration::Impl { data_type, body } => todo!(),
            Declaration::Using { file } => todo!(),
            Declaration::Module { name, body } => todo!(),
            Declaration::Extern { file, functions } => todo!(),
        }
    }


    pub fn stmt(
        &mut self,
        anal: &mut LocalAnalyser,
        scope: &mut ScopeId,
        stmt: &Statement,
    ) -> Option<()> {

        match stmt {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let reg = anal.fc.new_reg();
                let rhs_anal = self.node(anal, scope, rhs);

            
                let hint = hint.map(|hint| {
                    let scope = *self.sema.scopes.get(*scope).unwrap();
                    self.update_data_type(
                        &hint,
                        &scope,
                    )
                });

                let hint = match hint {
                    Some(Err(e)) => {                        
                        *scope = self.sema.scopes.push(Scope::new(
                            scope.some(), 
                            ScopeKind::Variable((*name, Type::Error, *is_mut, reg))
                        ));

                        anal.current.push(IR::Error(ErrorId::Sema(e)));
                        return None;
                    },

                    Some(Ok(e)) => Some(e),

                    _ => None,
                };
               

                if let Some(hint) = hint {
                    if !hint.is(rhs_anal.typ) {
                        *scope = self.sema.scopes.push(Scope::new(
                            scope.some(), 
                            ScopeKind::Variable((*name, Type::Error, *is_mut, reg))
                        ));

                        let error_id = ErrorId::Sema(self.errors.push(
                            Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.typ,
                                hint_type: hint,
                                source: rhs.range(),
                            }
                        ));
                        
                        anal.current.push(IR::Error(error_id));
                        return None;
                    }
                }

                anal.current.push(IR::Copy { dst: reg, src: rhs_anal.reg });

                *scope = self.sema.scopes.push(Scope::new(
                    scope.some(), 
                    ScopeKind::Variable((*name, rhs_anal.typ, *is_mut, reg))
                ));
                    
            },

            
            Statement::UpdateValue { lhs, rhs } => todo!(),
        };

        Some(())
    }

    
    pub fn expression(
        &mut self, 
        anal: &mut LocalAnalyser,
        scope: &mut ScopeId, 
        expr: &Expression,
        source: SourceRange,
    ) -> AnalysisResult {

        match expr {
            Expression::Unit => {
                let reg = anal.fc.new_reg();
                anal.current.push(IR::Unit { dst: reg });
                AnalysisResult::new(reg, Type::Unit)
            },

            
            Expression::Literal(v) => {
                let reg = anal.fc.new_reg();

                match v {
                    Literal::Integer(v) => anal.current.push(IR::LitI { dst: reg, lit: *v }),
                    Literal::Float(v)   => anal.current.push(IR::LitF { dst: reg, lit: v.0 }),
                    Literal::Bool(v)    => anal.current.push(IR::LitB { dst: reg, lit: *v }),

                    Literal::String(v)  => {
                        let id = StrConstId(self.str_consts.len() as u32);
                        let id = *self.str_consts.kget_or_insert(*v, id);
                        anal.current.push(IR::LitS { dst: reg, lit: id });
                    },
                }

                AnalysisResult::new(reg, match v {
                    Literal::Integer(_) => Type::Int,
                    Literal::Float(_) => Type::Float,
                    Literal::String(_) => Type::Str,
                    Literal::Bool(_) => Type::Bool,
                })
            },

            
            Expression::Identifier(v) => {
                let scope = self.sema.scopes.get(*scope).unwrap();
                let Some(var) = scope.find_var(*v, &self.sema.scopes)
                else {
                    let error = self.errors.push(Error::VariableNotFound {
                        name: *v, 
                        source, 
                    });
                    
                    return anal.error(ErrorId::Sema(error));
                };

                AnalysisResult::new(var.3, var.1)
            },

            
            Expression::BinaryOp { operator, lhs, rhs } => {
                let dst = anal.fc.new_reg();
                let lhs_anal = self.node(anal, scope, lhs);
                let rhs_anal = self.node(anal, scope, rhs);

                let typ = match (lhs_anal.typ, rhs_anal.typ) {
                    | (_, Type::Error)
                    | (Type::Error, _)
                     => return anal.empty_error(),

                    (  Type::Int,   Type::Int) if operator.is_arith() => Type::Int,
                    (Type::Float, Type::Float) if operator.is_arith() => Type::Float,

                    (  Type::Int,   Type::Int) if operator.is_bw()    => Type::Int,

                    (  Type::Int,   Type::Int) if operator.is_ocomp() => Type::Bool,
                    (Type::Float, Type::Float) if operator.is_ocomp() => Type::Bool,

                    (  Type::Int,   Type::Int) if operator.is_ecomp() => Type::Bool,
                    (Type::Float, Type::Float) if operator.is_ecomp() => Type::Bool,
                    ( Type::Bool,  Type::Bool) if operator.is_ecomp() => Type::Bool,
                    ( Type::Unit,  Type::Unit) if operator.is_ecomp() => Type::Bool,
                    (  Type::Any,   Type::Any) if operator.is_ecomp() => Type::Bool,

                    (Type::UserType(v1),  Type::UserType(v2)) 
                     if operator.is_ecomp() && v1 == v2
                     => Type::Bool,

                    _ => {
                        let error = self.errors.push(Error::InvalidBinaryOp {
                            operator: *operator,
                            lhs: lhs_anal.typ, 
                            rhs: rhs_anal.typ, 
                            source,
                        });

                        return anal.error(ErrorId::Sema(error));
                    }

                };

                anal.current.push(IR::BinaryOp {
                    op: *operator, 
                    typ, 
                    dst, 
                    lhs: lhs_anal.reg, 
                    rhs: rhs_anal.reg,
                });

                AnalysisResult::new(dst, typ)
            },

            
            Expression::UnaryOp { operator, rhs } => {
                let dst = anal.fc.new_reg();
                let rhs_anal = self.node(anal, scope, rhs);

                let ir = match (operator, rhs_anal.typ) {
                    (_, Type::Error) => return anal.empty_error(),

                    (UnaryOperator::Not, Type::Bool ) => IR::Not { dst, src: rhs_anal.reg },
                    (UnaryOperator::Neg, Type::Int  ) => IR::NegF { dst, src: rhs_anal.reg },
                    (UnaryOperator::Neg, Type::Float) => IR::NegI { dst, src: rhs_anal.reg },

                    _ => {
                        let error = self.errors.push(Error::InvalidUnaryOp {
                            operator: *operator,
                            rhs: rhs_anal.typ, 
                            source,
                        });

                        anal.current.push(IR::Error(ErrorId::Sema(error)));
                        return anal.empty_error();
                        
                    }
                };

                anal.current.push(ir);

                AnalysisResult::new(dst, rhs_anal.typ)
            },

            
            Expression::If { condition, body, else_block } => {
                let condition_anal = self.node(anal, scope, &condition);


                if !condition_anal.typ.is(Type::Bool) {
                    let error = self.errors.push(Error::InvalidType {
                        source: condition.range(), 
                        found: condition_anal.typ, 
                        expected: Type::Bool,
                    });

                    return anal.error(ErrorId::Sema(error));
                }
                
                
                let mut prev_block = std::mem::replace(
                    &mut anal.current,
                    anal.fc.new_block(Vec::new_in(anal.arena)),
                );

                let body_id = anal.current.id;

                let continue_block = anal.fc.new_block(Vec::new_in(anal.arena));

                anal.current.terminator = prev_block.terminator.clone();
                let body_anal = self.block(anal, *scope, &body);
                anal.current.terminator = Terminator::Jmp(continue_block.id);
                
                let dst = body_anal.reg;

                if let Some(else_node) = else_block {
                    let new_block = anal.fc.new_block(Vec::new_in(anal.arena));
                    let id = new_block.id;

                    let body_block = std::mem::replace (
                        &mut anal.current,
                        new_block,
                    );

                    let else_res = self.node(anal, &mut *scope, &else_node);

                    let mut else_block = std::mem::replace (
                        &mut anal.current,
                        continue_block,
                    );

                    else_block.terminator = Terminator::Jmp(anal.current.id);

                    prev_block.terminator = Terminator::Jif {
                        cond: condition_anal.reg, 
                        if_true: body_id, 
                        if_false: id,
                    };

                    if !body_anal.typ.is(else_res.typ) {
                        let error = self.errors.push(Error::IfBodyAndElseMismatch {
                            body: (body.range(), body_anal.typ), 
                            else_block: (else_node.range(), else_res.typ),
                        });

                        else_block.push(IR::Error(ErrorId::Sema(error)));

                        anal.blocks.push(prev_block);
                        anal.blocks.push(body_block);
                        anal.blocks.push(else_block);

                        return anal.empty_error()
                    }


                    else_block.push(IR::Copy { dst, src: else_res.reg });
                    anal.blocks.push(prev_block);
                    anal.blocks.push(body_block);
                    anal.blocks.push(else_block);
                    

                } else {
                    let body_block = std::mem::replace (
                        &mut anal.current,
                        continue_block,
                    );

                    prev_block.terminator = Terminator::Jif {
                        cond: condition_anal.reg, 
                        if_true: body_id, 
                        if_false: anal.current.id,
                    };

                    anal.blocks.push(prev_block);
                    anal.blocks.push(body_block);
                }

                AnalysisResult::new(dst, body_anal.typ)
            },

            
            Expression::Match { value, mappings } => {
                let value_anal = self.node(anal, scope, &value);
                let enum_mappings = match value_anal.typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { mappings } = self.types.get(v).unwrap().kind
                     => mappings,

                    Type::Error => return anal.empty_error(),

                    _ => {
                        let error = self.errors.push(Error::MatchValueIsntEnum {
                            source: value.range(), 
                            typ: value_anal.typ
                        });

                        return anal.error(ErrorId::Sema(error))
                    }
                };
                todo!()
            },

            
            Expression::Block { block } => self.block(anal, *scope, &block),

            
            Expression::CreateStruct { data_type, fields } => {
                let typ = {
                    let scope = *self.sema.scopes.get(*scope).unwrap();
                    let typ = self.update_data_type(data_type, &scope);
                    match typ {
                        Ok(v) => v,
                        Err(e) => return anal.error(ErrorId::Sema(e)),
                    }
                };


                let struct_fields = match typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Structure { fields, .. } = self.types.get(v).unwrap().kind
                     => fields,
                    
                    Type::Error => return anal.empty_error(),

                    _ => {
                        let error = self.errors.push(Error::StructCreationOnNonStruct {
                            source: data_type.range(), 
                            typ
                        });

                        return anal.error(ErrorId::Sema(error))
                    }
                };

                todo!()
            },

            
            Expression::AccessField { val, field, field_meta } => todo!(),
            Expression::CallFunction { name, is_accessor, args } => {
                let temp_arena = ArenaPool::tls_get_temp();
                let mut args_anal = Vec::with_cap_in(&*temp_arena, args.len());
                for arg in args.iter() {
                    args_anal.push(self.node(anal, scope, &arg.0))
                }

                let call_scope = if *is_accessor {
                    let typ_namespace = self.namespaceof(args_anal[0].typ);
                    let scope = Scope::new(scope.some(), ScopeKind::Namespace(typ_namespace));
                    self.sema.scopes.push(scope)
                } else { *scope };

                let func_id = {
                    let scope = self.sema.scopes.get(call_scope).unwrap();
                    let Some(func) = scope.find_func(*name, &self.sema.scopes, &self.sema.namespaces)
                    else {
                        let error = if *is_accessor {
                            Error::BindedFunctionNotFound { name: *name, bind: args_anal[0].typ, source }
                        } else { Error::FunctionNotFound { name: *name, source }};

                        return anal.error(ErrorId::Sema(self.errors.push(error)))
                    };
                    
                    func
                };

                let func = self.funcs.get(func_id).unwrap().unwrap_ref();

                if args.len() != func.args.len() {
                    let error = self.errors.push(Error::FunctionArgsMismatch {
                        source, 
                        sig_len: func.args.len(), 
                        call_len: args.len(),
                    });

                    return anal.error(ErrorId::Sema(error))
                }


                let dst = anal.fc.new_reg();
                let mut args_vec = Vec::with_cap_in(anal.arena, args.len());
                for (arg_anal, arg) in args_anal.iter().zip(args.iter()) {
                    args_vec.push((
                        arg_anal.reg, 
                        if arg.1 { anal.fc.new_reg() } else { arg_anal.reg}
                    ))
                }

                
                anal.current.push(IR::Call {
                    dst, 
                    function: func_id, 
                    args: args_vec.leak()
                });

                AnalysisResult::new(dst, func.return_type)
            },

            
            Expression::WithinNamespace { namespace, namespace_source, action } => todo!(),
            Expression::WithinTypeNamespace { namespace, action } => {                
                let typ = {
                    let scope = *self.sema.scopes.get(*scope).unwrap();
                    let typ = self.update_data_type(namespace, &scope);
                    match typ {
                        Ok(v) => v,
                        Err(e) => return anal.error(ErrorId::Sema(e)),
                    }
                };

                let namespace = self.namespaceof(typ);
                {
                    let scope = Scope::new(scope.some(), ScopeKind::Namespace(namespace));
                    let mut scope = self.sema.scopes.push(scope);
                    self.node(anal, &mut scope, action)
                }
            },
            Expression::Loop { body } => todo!(),
            Expression::Return(_) => todo!(),
            Expression::Continue => todo!(),
            Expression::Break => todo!(),
            Expression::CastAny { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
        }
    }
}