#![deny(unused_must_use)]

use common::{string_map::{StringIndex, StringMap}, source::SourceRange, fuck_map::FuckMap, OptionalPlus};
use errors::Error;
use ::errors::{SemaError, ErrorId};
use ir::terms::{Block, EnumVariant, Reg, BlockId, IR};
use parser::nodes::{Node, NodeKind, Declaration, Statement, Expression};
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
    let mut state = State {
        arena_type,
        arena_func,
        string_map,
        types: KVec::new(),
        funcs: KVec::new(),
        errors: KVec::new(),
        sema: InferState {
            arena_nasp,
            namespaces: KVec::new(),
            scopes: KVec::new(),
            option_table: FuckMap::new(),
            result_table: FuckMap::new(),
            namespace_table: FuckMap::new(),
        },
    };

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


#[derive(Clone, Copy, Debug, Eq, Hash)]
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


impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            | (Type::UserType(_), Type::UserType(_))
            | (Type::Str, Type::Str)
            | (Type::Int, Type::Int)
            | (Type::Bool, Type::Bool)
            | (Type::Float, Type::Float)
            | (Type::Unit, Type::Unit)
            | (Type::Any, Type::Any)
            | (Type::Never, _)
            | (_, Type::Never)
             => true,

            _ => false,
        }
    }
}


#[derive(Debug)]
pub struct State<'me, 'at, 'af, 'an> {
    arena_type: &'at Arena,
    arena_func: &'af Arena,

    pub string_map: &'me mut StringMap,
    
    types: KVec<TypeId, Option<TypeSymbol<'at>>>,
    funcs: KVec<FuncId, Option<Function<'af>>>,

    sema: InferState<'an>,

    pub errors: KVec<SemaError, Error>,
}


#[derive(Debug)]
enum TypeSymbol<'a> {
    Structure {
        kind: StructureKind,
        fields: &'a [Type],
    },

    Enum {
        mappings: &'a [(StringIndex, Type, SourceRange, bool)],
    },

    BuiltIn,
}


#[derive(Debug)]
enum StructureKind {
    Normal,
    Component,
    Resource,
}


#[derive(Debug)]
struct Function<'a> {
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

            errors: KVec::new(),
        }
    }

    
    #[inline(always)]
    fn declare_type(&mut self) -> TypeId {
        self.types.push(None)
    }


    #[inline(always)]
    fn declare_func(&mut self) -> FuncId {
        self.funcs.push(None)
    }

    
    fn update_type(&mut self, index: TypeId, symbol: TypeSymbol<'at>) {
        let type_symbol = self.types.get_mut(index).unwrap();
        type_symbol.replace(symbol);
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
        let pool = ArenaPool::tls_get_rec();

        let mut final_type = Type::Unit;
        let mut final_reg = anal.fc.new_reg();

        for node in nodes {
            let result = match node.kind() {
                NodeKind::Declaration(_) => continue,

                _ => self.node(anal, &mut scope, node),
            };

            match result {
                Some(v) => {
                    final_type = v.typ;
                    final_reg = v.reg;
                },

                None => final_type = Type::Never,
            }
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
    ) -> Option<AnalysisResult> {
        match node.kind() {
            NodeKind::Declaration(_) => unreachable!(),

            NodeKind::Statement(stmt) => {
                self.stmt(anal, scope, stmt)?;
                Some(AnalysisResult::new(anal.fc.new_reg(), Type::Unit))
            },

            NodeKind::Expression(expr) => self.expression(anal, scope, expr),

            NodeKind::Error(v) => {
                anal.current.push(IR::Error(*v));
                Some(AnalysisResult::new(anal.fc.new_reg(), Type::Never))
            },
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
                let start = anal.fc.new_block(Vec::new_in(self.arena_func));
                let mut start = std::mem::replace(&mut anal.current, start);
                
                let (analysis, mut ir_body) = {
                    let block_anal = self.block(
                        &mut anal, 
                        scope, 
                        &body,
                    );

                    let mut vec = anal.blocks;
                    vec.push(anal.current);

                    (block_anal, vec)
                };

                let func = self.funcs.get_mut(index).unwrap().as_mut().unwrap();

                if analysis.typ != func.return_type {
                    start.push(IR::Error(ErrorId::Sema(self.errors.push(
                        Error::FunctionBodyAndReturnMismatch {
                            source: body.range(), 
                            return_type: func.return_type, 
                            body_type: analysis.typ,
                        }
                    ))));
                }

                start.terminator = Terminator::Jmp(ir_body.last().unwrap().id);
                ir_body.push(start);
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

                let rhs_anal = match rhs_anal {
                    Some(v) => v,

                    None => {
                        *scope = self.sema.scopes.push(Scope::new(
                            scope.some(),
                            ScopeKind::Variable((
                                *name,
                                Type::Error,
                                *is_mut,
                                reg,
                            ))
                        ));

                        return None
                    },
                };

            
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
                    if hint != rhs_anal.typ {
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
        expr: &Expression
    ) -> Option<AnalysisResult> {

        let result = match expr {
            Expression::Unit => AnalysisResult::new(anal.fc.new_reg(), Type::Unit),
            Expression::Literal(_) => todo!(),
            Expression::Identifier(_) => todo!(),
            Expression::BinaryOp { operator, lhs, rhs } => todo!(),
            Expression::UnaryOp { operator, rhs } => todo!(),
            Expression::If { condition, body, else_block } => todo!(),
            Expression::Match { value, mappings } => todo!(),
            Expression::Block { block } => todo!(),
            Expression::CreateStruct { data_type, fields } => todo!(),
            Expression::AccessField { val, field, field_meta } => todo!(),
            Expression::CallFunction { name, is_accessor, args } => todo!(),
            Expression::WithinNamespace { namespace, namespace_source, action } => todo!(),
            Expression::WithinTypeNamespace { namespace, action } => todo!(),
            Expression::Loop { body } => todo!(),
            Expression::Return(_) => todo!(),
            Expression::Continue => todo!(),
            Expression::Break => todo!(),
            Expression::CastAny { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
        };

        Some(result)
    }
}