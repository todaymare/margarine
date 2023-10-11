#![feature(if_let_guard)]
#![deny(unused_must_use)]

use common::{string_map::{StringIndex, StringMap}, source::SourceRange, fuck_map::FuckMap, Swap};
use errors::Error;
use ::errors::{SemaError, ErrorId};
use ir::terms::{Block, EnumVariant, Reg, BlockId, IR, StrConstId};
use lexer::Literal;
use parser::nodes::{Node, NodeKind, Declaration, Statement, Expression, UnaryOperator};
use sema::{InferState, Namespace, Scope, ScopeId, ScopeKind};
use sti::{keyed::KVec, vec::Vec, define_key, prelude::{Arena, GlobalAlloc}, arena_pool::ArenaPool, packed_option::PackedOption, traits::FromIn};

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


    pub fn typeid(self) -> TypeId {
        match self {
            Type::UserType(v) => v,
            Type::Unit => TypeId(0),
            Type::Int => TypeId(1),
            Type::Float => TypeId(2),
            // TODO: Add UInt
            Type::Bool => TypeId(4),
            Type::Str => TypeId(5),
            Type::Any => TypeId(6),
            Type::Never => TypeId(7),
            Type::Error => TypeId(8),
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
    pub funcs: KVec<FuncId, Function<'af>>,

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
    Struct {
        kind: StructureKind,
        fields: &'a [(StringIndex, Type)],
    },

    Enum {
        mappings: &'a [TypeEnumMapping],
        typ: EnumType,
    },

    BuiltIn,
    Unknown,
}


#[derive(Debug)]
enum EnumType {
    UserDefined,
    Option,
    Result,
}


#[derive(Debug)]
struct TypeEnumMapping {
    name: StringIndex,
    typ: Type,
    range: SourceRange,
    is_implicit_unit: bool,
    variant: EnumVariant
}

impl TypeEnumMapping {
    fn new(name: StringIndex, typ: Type, range: SourceRange, is_implicit_unit: bool, variant: EnumVariant) -> Self { Self { name, typ, range, is_implicit_unit, variant } }
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
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            regs: 0,
            blocks: 0,
        }
    }


    #[inline(always)]
    pub fn new_reg(&mut self) -> Reg {
        self.regs += 1;
        Reg(self.regs - 1)
    }


    #[inline(always)]
    pub fn new_block<'a>(&mut self, arena: &'a Arena) -> Block<'a> {
        self.blocks += 1;
        Block {
            id: BlockId(self.blocks-1),
            body: Vec::new_in(arena),
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
    #[inline(always)]
    pub fn new(arena: &'a Arena) -> Self { 
        let mut fc = FunctionCounter::new();
        let block = fc.new_block(arena);
        Self {
            arena, 
            fc, 
            blocks: Vec::new_in(arena), 
            current: block,
        } 
    }


    ///
    /// Creates an `AnalysisResult` instance of type Error with a unused
    /// register and issues an error in the bytecode of the current block
    ///
    #[inline(always)]
    pub fn error(&mut self, error: ErrorId) -> AnalysisResult {
        self.current.push(IR::Error(error));
        AnalysisResult::new(self.fc.new_reg(), Type::Error)
    }


    ///
    /// Creates an `AnalysisResult` of type Error with a unused register
    /// without issuing an error in the bytecode of the current block
    ///
    #[inline(always)]
    pub fn empty_error(&mut self) -> AnalysisResult {
        AnalysisResult::new(self.fc.new_reg(), Type::Error)
    }


    ///
    /// Creates a new block with a unique (to the analyser instance) id
    ///
    #[inline(always)]
    pub fn new_block(&mut self) -> Block<'a> {
        self.fc.new_block(self.arena)
    }


    ///
    /// Replaces the current block with a new one
    /// returning the old current block
    ///
    #[inline(always)]
    pub fn new_current(&mut self) -> Block<'a> {
        self.current.swap(self.fc.new_block(self.arena))
    }
}


pub struct AnalysisResult {
    reg: Reg,
    typ: Type,
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(reg: Reg, typ: Type) -> Self { Self { reg, typ, is_mut: false } }
}


impl<'me, 'at, 'af, 'an> State<'me, 'at, 'af, 'an> {
    pub fn new(
        arena_type: &'at Arena,
        arena_func: &'af Arena,
        arena_nasp: &'an Arena,

        string_map: &'me mut StringMap,
    ) -> Self {
        let types = {
            let mut kvec = KVec::with_cap(256);
            let mut func = |x: StringIndex| {
                kvec.push(TypeSymbol {
                    name: x,
                    source: SourceRange::ZERO,
                    kind: TypeSymbolKind::BuiltIn,
                });
            };

            func(string_map.insert("int"));
            func(string_map.insert("float"));
            func(string_map.insert("uint"));
            func(string_map.insert("bool"));
            func(string_map.insert("str"));
            func(string_map.insert("any"));
            func(string_map.insert("never"));
            func(string_map.insert("error"));
            let reserved = string_map.insert("reserved");
            let kvec_len = kvec.len();
            let mut func = |x: StringIndex| {
                kvec.push(TypeSymbol {
                    name: x,
                    source: SourceRange::ZERO,
                    kind: TypeSymbolKind::BuiltIn,
                });
            };
            for _ in 0..(256 - kvec_len) {
                func(reserved);
            }

            kvec
        };
        
        Self {
            arena_type,
            arena_func,
            string_map,

            types,
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
        self.funcs.push(Function { args: self.arena_func.alloc_new([]), body: Vec::new_in(self.arena_func), return_type: Type::Error })
    }


    #[inline(always)]
    fn create_func(&mut self, func: Function<'af>) -> FuncId {
        let id = self.declare_func();
        self.funcs.get_mut(id).unwrap().swap(func);
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
        node: &Node,
    ) -> AnalysisResult {
        match node.kind() {
            NodeKind::Declaration(_) => unreachable!(),

            NodeKind::Statement(stmt) => {
                self.stmt(anal, scope, stmt, node.range());
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

            Declaration::Function { is_system: _, name, header, arguments: _, return_type, body } => {
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

                let func = self.funcs.get(index).unwrap();
                let scope = Scope::new(scope.some(), sema::ScopeKind::Function((func.return_type, return_type.range())));
                let mut scope = self.sema.scopes.push(scope);

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

                let func = self.funcs.get_mut(index).unwrap();

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

                let mut ir_body = anal.blocks.swap(Vec::new_in(self.arena_func));
                ir_body.push(anal.current);
                ir_body.sort_unstable_by_key(|b| b.id.0);

                func.body = ir_body;

                Ok(())
            },

            Declaration::Impl { data_type: _, body: _ } => todo!(),
            Declaration::Using { file: _ } => todo!(),
            Declaration::Module { name: _, body: _ } => todo!(),
            Declaration::Extern { file: _, functions: _ } => todo!(),
        }
    }


    pub fn stmt(
        &mut self,
        anal: &mut LocalAnalyser,
        scope: &mut ScopeId,
        stmt: &Statement,
        source: SourceRange,
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

            
            Statement::UpdateValue { lhs, rhs } => {
                let lhs_anal = self.node(anal, scope, lhs);
                let rhs_anal = self.node(anal, scope, rhs);

                if !lhs_anal.typ.is(rhs_anal.typ) {
                    anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                        Error::ValueUpdateTypeMismatch { 
                            lhs: lhs_anal.typ, rhs: rhs_anal.typ, source }
                    ))))
                }

                anal.current.push(IR::Copy { dst: lhs_anal.reg, src: rhs_anal.reg });

            },
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

                let mut result = AnalysisResult::new(var.3, var.1);
                result.is_mut = var.2;
                result
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
                    op: *operator, typ, dst, 
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
                
                
                let mut prev_block = anal.new_current();

                let body_id = anal.current.id;

                let continue_block = anal.new_block();

                anal.current.terminator = prev_block.terminator.clone();
                let body_anal = self.block(anal, *scope, &body);
                anal.current.terminator = Terminator::Jmp(continue_block.id);
                
                let dst = body_anal.reg;

                if let Some(else_node) = else_block {
                    let new_block = anal.new_block();
                    let id = new_block.id;

                    let body_block = anal.current.swap(new_block);

                    let else_res = self.node(anal, &mut *scope, &else_node);
                    let mut else_block = anal.current.swap(continue_block);
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
                    let body_block = anal.current.swap(continue_block);

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

            
            Expression::Match { value, mappings, taken_as_inout } => {
                let value_anal = self.node(anal, scope, &value);
                if *taken_as_inout && !value_anal.is_mut {
                    anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                        Error::InOutValueIsntMut(value.range())
                    ))))
                }

                let enum_mappings = match value_anal.typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { mappings, .. } = self.types.get(v).unwrap().kind
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

                {
                    let err_count = self.errors.len();
                    // duplicates
                    {
                        for i in 0..mappings.len() {
                            for j in 0..i {
                                if mappings[i].name() == mappings[j].name() {
                                    anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                        Error::DuplicateMatch { 
                                            declared_at: mappings[j].range(), 
                                            error_point: mappings[i].range(),
                                        }
                                    ))));
                                }
                            }
                        }
                    }

                    // invalids
                    {
                        for i in mappings.iter() {
                            if !enum_mappings.iter().any(|x| x.name == i.name()) {
                                anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                    Error::InvalidMatch {
                                        name: i.name(), range: i.range(), value: value_anal.typ }
                                ))))
                            }
                        }
                    }

                    // missings
                    {
                        let pool = ArenaPool::tls_get_temp();
                        let mut vec = Vec::with_cap_in(
                            &*pool,
                            mappings.len(),
                        );

                        for i in enum_mappings.iter() {
                            if !mappings.iter().any(|x| x.name() == i.name) {
                                vec.push(i.name)
                            }
                        }

                        if !vec.is_empty() {
                            anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::MissingMatch {
                                    name: vec.move_into(GlobalAlloc), 
                                    range: source, 
                                }
                            ))))
                        }
                    }

                    if err_count != self.errors.len() {
                        return anal.empty_error()
                    }
                }
                
                let temp = ArenaPool::tls_get_temp();
                let mut blocks = Vec::with_cap_in(&*temp, mappings.len());
                
                let continue_block = anal.new_block();
                let continue_block_id = continue_block.id;
                let mut entry_block = anal.current.swap(continue_block);

                let mut return_type = None;
                let return_reg = anal.fc.new_reg();
                for m in mappings.iter() {

                    let mapping_in_enum = 
                        enum_mappings.iter()
                        .find(|x| x.name == m.name()).unwrap();

                    let binding_reg = anal.fc.new_reg();
                    let binding_scope = Scope::new(scope.some(), ScopeKind::Variable((
                        m.binding(), 
                        mapping_in_enum.typ,
                        false,
                        binding_reg,
                    )));

                    let mut binding_scope = self.sema.scopes.push(binding_scope);

                    let mut block_start = anal.new_block();
                    let block_start_id = block_start.id;

                    if m.is_inout() {
                        if !*taken_as_inout {
                            block_start.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::InOutBindingWithoutInOutValue {
                                    value_range: value.range(), 
                                    binding_range: m.binding_range()
                                }
                            ))))
                        }

                        block_start.push(IR::AccUnwrapEnumVariant {
                            dst: binding_reg, 
                            src: value_anal.reg, 
                            variant: mapping_in_enum.variant, 
                            typ: mapping_in_enum.typ.typeid(),
                        });
                        
                    }
                    
                    let swapped = anal.current.swap(block_start);

                    let result = self.node(anal, &mut binding_scope, m.node());

                    let mut block_end = anal.current.swap(swapped);

                    block_end.push(IR::Copy { dst: return_reg, src: result.reg });
                    if m.is_inout() {
                        block_end.push(IR::CopyData { dst: value_anal.reg, src: binding_reg });
                    }

                    block_end.terminator = Terminator::Jmp(continue_block_id);

                    match (return_type, result.typ) {
                        | (Some((Type::Error | Type::Never, _)), _) 
                        | (None, _) 
                         => return_type = Some((result.typ, m.node().range())),

                        (Some(v), _) if v.0 == result.typ => (),
                        (Some(v), _) => {
                            let error = self.errors.push(Error::MatchBranchesDifferInReturnType { 
                                initial_source: v.1, 
                                initial_typ: v.0, 
                                branch_source: m.node().range(), 
                                branch_typ: result.typ,
                            });

                            block_end.push(IR::Error(ErrorId::Sema(error)))
                        },
                    }

                    anal.blocks.push(block_end);
                    blocks.push(block_start_id);

                }

                let entry_block_term = entry_block.terminator.swap(Terminator::Match {
                    src: value_anal.reg, jumps: blocks.move_into(anal.arena).leak() 
                });

                anal.current.terminator = entry_block_term;
                anal.blocks.push(entry_block);

                AnalysisResult::new(
                    return_reg, 
                    return_type.map(|x| x.0)
                        .unwrap_or(Type::Unit)
                )
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


                let (type_id, struct_fields) = match typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Struct { fields, .. } = self.types.get(v).unwrap().kind
                     => (v, fields),
                    
                    Type::Error => return anal.empty_error(),

                    _ => {
                        let error = self.errors.push(Error::StructCreationOnNonStruct {
                            source: data_type.range(), 
                            typ
                        });

                        return anal.error(ErrorId::Sema(error))
                    }
                };

                let err_count = self.errors.len();
                // duplicates
                {
                    for i in 0..fields.len() {
                        for j in 0..i {
                            if fields[i].0 == fields[j].0 {
                                anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                    Error::DuplicateField { 
                                        declared_at: fields[j].1, 
                                        error_point: fields[i].1,
                                    }
                                ))));
                            }
                        }
                    }
                }

                // invalids
                {
                    for i in fields.iter() {
                        if !struct_fields.iter().any(|x| x.0 == i.0) {
                            anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                                Error::FieldDoesntExist {
                                    field: i.0, source: i.1, typ }
                            ))))
                        }
                    }
                }

                // missings
                {
                    let pool = ArenaPool::tls_get_temp();
                    let mut vec = Vec::with_cap_in(
                        &*pool,
                        fields.len(),
                    );

                    for i in struct_fields.iter() {
                        if !fields.iter().any(|x| x.0 == i.0) {
                            vec.push(i.0)
                        }
                    }

                    if !vec.is_empty() {
                        anal.current.push(IR::Error(ErrorId::Sema(self.errors.push(
                            Error::MissingMatch {
                                name: vec.move_into(GlobalAlloc), 
                                range: source, 
                            }
                        ))))
                    }
                }

                if err_count != self.errors.len() {
                    return anal.empty_error()
                }

                let temp = ArenaPool::tls_get_temp();
                let mut fields_anal = Vec::with_cap_in(&*temp, fields.len());
                for f in fields.iter() {
                    fields_anal.push(self.node(anal, scope, &f.2))
                }

                let dst = anal.fc.new_reg();
                anal.current.push(IR::CreateStruct { 
                    dst, 
                    type_id, 
                    fields: Vec::from_in(
                        anal.arena, 
                        fields_anal.iter().map(|x| x.reg)
                    ).leak() 
                });

                AnalysisResult::new(dst, typ)
            },


            Expression::AccessField { val, field_name } => {
                let val_anal = self.node(anal, scope, val);

                match val_anal.typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Struct { fields, .. } = self.types.get(v).unwrap().kind
                     => {
                        for (i, f) in fields.iter().enumerate() {
                            if f.0 == *field_name {
                                let dst = anal.fc.new_reg();
                                anal.current.push(IR::AccField { 
                                    dst, 
                                    src: val_anal.reg, 
                                    field_index: i.try_into().unwrap(),
                                });

                                return AnalysisResult::new(dst, f.1)
                            }
                        }

                        let error = self.errors.push(Error::FieldDoesntExist {
                            source: val.range(), 
                            field: *field_name, 
                            typ: val_anal.typ,
                        });

                        anal.error(ErrorId::Sema(error))
                    },

                    
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { mappings, .. } = self.types.get(v).unwrap().kind
                     => {
                        for (i, f) in mappings.iter().enumerate() {
                            if f.name == *field_name {
                                let dst = anal.fc.new_reg();
                                anal.current.push(IR::AccEnumVariant { 
                                    dst, 
                                    src: val_anal.reg,
                                    variant: EnumVariant(i as u16),
                                    typ: f.typ.typeid(), 
                                });

                                let option = self.create_option(f.typ);

                                return AnalysisResult::new(dst, option)
                            }
                        }

                        let error = self.errors.push(Error::FieldDoesntExist {
                            source: val.range(), 
                            field: *field_name, 
                            typ: val_anal.typ,
                        });

                        anal.error(ErrorId::Sema(error))
                    },
                    
                    Type::Error => anal.empty_error(),

                    _ => {
                        let error = self.errors.push(Error::FieldAccessOnNonEnumOrStruct {
                            source: val.range(),
                            typ: val_anal.typ
                        });

                        anal.error(ErrorId::Sema(error))
                    }
                }
            },


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

                let func = self.funcs.get(func_id).unwrap();

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


            Expression::WithinNamespace { namespace, namespace_source, action } => {
                let namespace = {
                    let scope = self.sema.scopes.get(*scope).unwrap();
                    let ns = scope.find_namespace(*namespace, &self.sema.scopes);
                    match ns {
                        Some(v) => v,
                        None => {
                            let typ_namespace = scope.find_type(
                                *namespace, 
                                &self.sema.scopes, 
                                &self.sema.namespaces
                            );

                            match typ_namespace {
                                Some(v) => self.namespaceof(Type::UserType(v)),
                                None => {
                                    let error = self.errors.push(Error::NamespaceNotFound {
                                        source: *namespace_source, 
                                        namespace: *namespace
                                    });

                                    return anal.error(ErrorId::Sema(error))
                                },
                            }
                        },
                    }
                };

                {
                    let scope = Scope::new(scope.some(), ScopeKind::Namespace(namespace));
                    let mut scope = self.sema.scopes.push(scope);
                    self.node(anal, &mut scope, action)
                }
            },

            
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

            
            Expression::Loop { body } => {
                let mut previous_block = anal.new_current();
                let start_block_id = anal.current.id;
                let continue_block = anal.new_block();

                let scope = Scope::new(scope.some(), ScopeKind::Loop {
                    start: start_block_id, 
                    end: continue_block.id,
                });

                let scope = self.sema.scopes.push(scope);

                let _body_anal = self.block(anal, scope, body);
                let mut body_end_block = anal.current.swap(continue_block);

                let terminator = previous_block.terminator.swap(Terminator::Jmp(start_block_id));
                body_end_block.terminator = Terminator::Jmp(start_block_id);
                anal.current.terminator = terminator;

                anal.blocks.push(previous_block);
                anal.blocks.push(body_end_block);

                AnalysisResult::new(anal.fc.new_reg(), Type::Unit)
            },

            
            Expression::Return(v) => {
                let val_anal = self.node(anal, scope, v);
                let (func_ret, func_src) = {
                    let scope = self.sema.scopes.get(*scope).unwrap();
                    scope.current_func_return_type(&self.sema.scopes)
                };

                if !val_anal.typ.is(func_ret) {
                    let err = self.errors.push(Error::ReturnAndFuncTypDiffer {
                        source,
                        func_source: func_src,
                        typ: val_anal.typ,
                        func_typ: func_ret,
                    });

                    return anal.error(ErrorId::Sema(err))
                }

                let mut prev_block = anal.new_current();
                prev_block.terminator = Terminator::Ret;
                anal.blocks.push(prev_block);

                AnalysisResult::new(anal.fc.new_reg(), Type::Never)
            },


            Expression::Continue => {
                let scope = self.sema.scopes.get(*scope).unwrap();
                let Some((loop_start, _)) = scope.find_loop(&self.sema.scopes)
                else {
                    let err = self.errors.push(Error::ContinueOutsideOfLoop(source));
                    return anal.error(ErrorId::Sema(err))
                };

                let mut prev_block = anal.new_current();
                prev_block.terminator = Terminator::Jmp(loop_start);
                anal.blocks.push(prev_block);

                AnalysisResult::new(anal.fc.new_reg(), Type::Never)
            },


            Expression::Break => {
                let scope = self.sema.scopes.get(*scope).unwrap();
                let Some((_, loop_end)) = scope.find_loop(&self.sema.scopes)
                else {
                    let err = self.errors.push(Error::ContinueOutsideOfLoop(source));
                    return anal.error(ErrorId::Sema(err))
                };

                let mut prev_block = anal.new_current();
                prev_block.terminator = Terminator::Jmp(loop_end);
                anal.blocks.push(prev_block);

                AnalysisResult::new(anal.fc.new_reg(), Type::Never)
            },


            Expression::CastAny { lhs, data_type } => {
                let lhs_anal = self.node(anal, scope, lhs);
                let scope = *self.sema.scopes.get(*scope).unwrap();
                let typ = self.update_data_type(data_type, &scope);
                let typ = match typ {
                    Ok(v) => v,
                    Err(e) => return anal.error(ErrorId::Sema(e)),
                };

                let opt_typ = self.create_option(typ);
                let dst = anal.fc.new_reg();

                anal.current.push(IR::CastAny { dst, src: lhs_anal.reg, target: typ.typeid() });

                AnalysisResult::new(dst, opt_typ)
            },

            
            Expression::Unwrap(v) => {
                let val_anal = self.node(anal, scope, v);
                match val_anal.typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { 
                        typ: EnumType::Option | EnumType::Result, 
                        mappings 
                    } = self.types.get(v).unwrap().kind
                     => {
                        let typ = mappings[0].typ;
                        let dst = anal.fc.new_reg();

                        anal.current.push(IR::Unwrap { src: val_anal.reg, dst });
                        
                        AnalysisResult::new(dst, typ)
                    },

                    | Type::Never
                    | Type::Error => return anal.empty_error(),

                    _ => {
                        let error = self.errors.push(
                            Error::CantUnwrapOnGivenType(source, val_anal.typ));

                        return anal.error(ErrorId::Sema(error))
                    }
                }
            },

            
            Expression::OrReturn(v) => {
                let val_anal = self.node(anal, scope, v);
                match val_anal.typ {
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { 
                        typ: EnumType::Option, 
                        mappings 
                    } = self.types.get(v).unwrap().kind
                     => {
                        let typ = mappings[0].typ;
                        let dst = anal.fc.new_reg();
                        let (func_ret, _func_src) = {
                            let scope = self.sema.scopes.get(*scope).unwrap();
                            scope.current_func_return_type(&self.sema.scopes)
                        };

                        match func_ret {
                            Type::UserType(v) 
                             if let TypeSymbolKind::Enum { 
                                typ: EnumType::Option, 
                                ..
                            } = self.types.get(v).unwrap().kind
                             => {}

                            _ => {
                                let error = self.errors.push(Error::FunctionDoesntReturnAnOption {
                                    source, func_typ: func_ret });

                                return anal.error(ErrorId::Sema(error))
                            }
                        }

                        anal.current.push(IR::OrReturn { src: val_anal.reg, dst });
                        
                        AnalysisResult::new(dst, typ)
                    },

                    
                    Type::UserType(v) 
                     if let TypeSymbolKind::Enum { 
                        typ: EnumType::Result, 
                        mappings 
                    } = self.types.get(v).unwrap().kind
                     => {
                        let typ = mappings[0].typ;
                        let err = mappings[1].typ;
                        let dst = anal.fc.new_reg();
                        let (func_ret, func_src) = {
                            let scope = self.sema.scopes.get(*scope).unwrap();
                            scope.current_func_return_type(&self.sema.scopes)
                        };

                        match func_ret {
                            Type::UserType(v) 
                             if let TypeSymbolKind::Enum { 
                                typ: EnumType::Result, 
                                mappings: &[_, TypeEnumMapping { typ, .. }],
                            } = self.types.get(v).unwrap().kind
                             => {
                                if !err.is(typ) {
                                    let func_err_typ = self.types.get(func_ret.typeid()).unwrap();
                                    let TypeSymbolKind::Enum { mappings: &[_, TypeEnumMapping { typ: func_err_typ, .. }], .. } = func_err_typ.kind
                                    else { unreachable!() };

                                    let error = self.errors.push(Error::FunctionReturnsAResultButTheErrIsntTheSame { 
                                        source, func_source: func_src, 
                                        func_err_typ, err_typ: err
                                    });

                                    return anal.error(ErrorId::Sema(error))
                                }
                            }

                            _ => {
                                let error = self.errors.push(Error::FunctionDoesntReturnAResult {
                                    source, func_typ: func_ret });

                                return anal.error(ErrorId::Sema(error))
                            }
                        }

                        anal.current.push(IR::OrReturn { src: val_anal.reg, dst });
                        
                        AnalysisResult::new(dst, typ)
                    },

                    | Type::Never
                    | Type::Error => return anal.empty_error(),

                    _ => {
                        let error = self.errors.push(
                            Error::CantTryOnGivenType(source, val_anal.typ));

                        return anal.error(ErrorId::Sema(error))
                    }
                }
            }
        }
    }
}