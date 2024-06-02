#![feature(get_many_mut)]
use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use errors::Error;
use ::errors::{ErrorId, SemaError};
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::{decl::DeclId, expr::ExprId, stmt::StmtId, NodeId, AST}, dt::{DataType, DataTypeKind}};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, keyed::KVec};
use types::{func::FunctionTy, ty::Type, GenListId, Generic, GenericKind, SymbolId, SymbolMap};

use crate::scope::ScopeKind;

pub mod scope;
pub mod namespace;
pub mod types;
pub mod errors;
pub mod analysis;
pub mod codegen;
pub mod global;

#[derive(Debug)]
pub struct TyChecker<'me, 'out, 'ast, 'str> {
    output      : &'out Arena,
    pub string_map: &'me mut StringMap<'str>,
    ast         : &'me AST<'ast>,

    scopes      : ScopeMap<'out>,
    namespaces  : NamespaceMap,
    pub syms    : SymbolMap<'out>,
    type_info   : TyInfo,
    startups    : Vec<SymbolId>,

    pub errors    : KVec<SemaError, Error>,
}


#[derive(Debug)]
pub struct TyInfo {
    exprs: KVec<ExprId, Option<ExprInfo>>,
    stmts: KVec<StmtId, Option<ErrorId>>,
    decls: KVec<DeclId, Option<ErrorId>>,
    funcs: HashMap<ExprId, (SymbolId, GenListId)>,
}


#[derive(Debug, Clone, Copy)]
pub enum ExprInfo {
    Result {
        ty    : Type,
        is_mut: bool,
    },

    Errored(ErrorId),
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Type,
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(ty: Type, is_mut: bool) -> Self { Self { ty, is_mut } }
    pub fn error() -> Self { Self::new(Type::ERROR, true) }
    pub fn never() -> Self { Self::new(Type::NEVER, true) }
}


impl<'me, 'out, 'ast, 'str> TyChecker<'me, 'out, 'ast, 'str> {
    pub fn run(out: &'out Arena,
               ast: &'me mut AST<'ast>,
               block: &[NodeId],
               string_map: &'me mut StringMap<'str>) -> Self {

        let mut ns = NamespaceMap::new();
        let mut analyzer = TyChecker {
            output: out,
            scopes: ScopeMap::new(),
            syms: SymbolMap::new(out, &mut ns, string_map),
            string_map,
            namespaces: ns,
            errors: KVec::new(),
            type_info: TyInfo {
                exprs: KVec::new(),
                stmts: KVec::new(),
                decls: KVec::new(),
                funcs: HashMap::new(),
            },
            ast,
            startups: Vec::new(),
        };

        {
            analyzer.type_info.exprs.inner_mut_unck().resize(analyzer.ast.exprs().len(), None);
            analyzer.type_info.stmts.inner_mut_unck().resize(analyzer.ast.stmts().len(), None);
            analyzer.type_info.decls.inner_mut_unck().resize(analyzer.ast.decls().len(), None);
        }

        let core_ns = {
            let mut namespace = Namespace::new(analyzer.string_map.insert("::core"));

            macro_rules! add_sym {
                ($n: ident) => {
                    namespace.add_sym(StringMap::$n, SymbolId::$n);
                };
            }

            add_sym!(I8);
            add_sym!(I16);
            add_sym!(I32);
            add_sym!(I64);
            add_sym!(U8);
            add_sym!(U16);
            add_sym!(U32);
            add_sym!(U64);
            add_sym!(F32);
            add_sym!(F64);
            add_sym!(BOOL);
            add_sym!(PTR);
            add_sym!(OPTION);
            add_sym!(RESULT);
            add_sym!(STR);
            add_sym!(RANGE);

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::OPTION));
                namespace.add_sym(StringMap::SOME, ns.get_sym(StringMap::SOME).unwrap().unwrap());
                namespace.add_sym(StringMap::NONE, ns.get_sym(StringMap::NONE).unwrap().unwrap());
            }

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::RESULT));
                namespace.add_sym(StringMap::OK , ns.get_sym(StringMap::OK ).unwrap().unwrap());
                namespace.add_sym(StringMap::ERR, ns.get_sym(StringMap::ERR).unwrap().unwrap());
            }

            analyzer.namespaces.push(namespace)
        };

        let scope = Scope::new(None, ScopeKind::ImplicitNamespace(core_ns));
        let scope = analyzer.scopes.push(scope);
        let empty = analyzer.string_map.insert("");
        analyzer.block(empty, scope, block);

        for v in analyzer.syms.vars.iter() {
            if v.1.sub.is_none() {
                let error = Error::UnableToInfer(analyzer.ast.range(v.1.node));
                Self::error_ex(&mut analyzer.errors, &mut analyzer.type_info, v.1.node, error)
            }
        }

        analyzer
    }


    fn error(&mut self, node: impl Into<NodeId>, error: Error) {
        Self::error_ex(&mut self.errors, &mut self.type_info, node, error)
    }


    fn error_ex(errors: &mut KVec<SemaError, Error>, ty_info: &mut TyInfo, node: impl Into<NodeId>, error: Error) {
        let error = errors.push(error);
        let error = ErrorId::Sema(error);
        match node.into() {
            NodeId::Expr(id) => {
                let val = &mut ty_info.exprs[id];
                match val {
                    Some(v) => match v {
                        ExprInfo::Result { .. } => *val = Some(ExprInfo::Errored(error)),
                        ExprInfo::Errored(_) => (),
                    },
                    None => *val = Some(ExprInfo::Errored(error)),
                };
            },

            NodeId::Decl(v) => ty_info.set_decl(v, error),
            NodeId::Stmt(v) => ty_info.set_stmt(v, error),
            NodeId::Err(_) => (),
        }
    }
    

    fn dt_to_gen(&mut self, scope: Scope, dt: DataType,
                 gens: &[StringIndex]) -> Result<Generic<'out>, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::UNIT, &[]))),


            DataTypeKind::Never => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::NEVER, &[]))),


            DataTypeKind::Tuple(_) => todo!(),


            DataTypeKind::Within(ns, dt) => {
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces, &self.syms)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                if ns.1 { return Err(Error::Bypass) }

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns.0));
                self.dt_to_gen(scope, *dt, gens)
            },


            DataTypeKind::CustomType(name, generics) => {
                if gens.contains(&name) { return Ok(Generic::new(dt.range(),
                                                    GenericKind::Generic(name))) }

                let Some(base) = scope.find_sym(name, &self.scopes, &mut self.syms, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let base = base?;

                let genc = self.syms.sym_gens_size(base);

                if genc != generics.len() {
                    return Err(Error::GenericLenMismatch {
                        source: dt.range(), found: generics.len(), expected: genc });
                }

                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*self.output, generics.len());

                    for g in generics {
                        vec.push(self.dt_to_gen(scope, *g, gens)?);
                    }
                    vec
                };

                Ok(Generic::new(dt.range(),
                                GenericKind::Sym(base, generics.leak())))
            },
        }
    }


    fn dt_to_ty(&mut self, scope_id: ScopeId, id: impl Into<NodeId> + Copy,
                dt: DataType) -> Result<Type, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Type::UNIT),
            DataTypeKind::Never => Ok(Type::NEVER),


            DataTypeKind::Within(ns, dt) => {
                let scope = self.scopes.get(scope_id);
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces, &self.syms)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                if ns.1 { return Err(Error::Bypass) }

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns.0));
                let scope = self.scopes.push(scope);
                self.dt_to_ty(scope, id, *dt)
            },


            DataTypeKind::CustomType(name, generics_list) => {
                let scope = self.scopes.get(scope_id);
                if scope.find_gen(name, &self.scopes).is_some() {
                    return Err(Error::GenericOnGeneric { source: dt.range() });
                }


                let Some(base) = scope.find_sym(name, &self.scopes, &mut self.syms, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let base = base?;

                let base_sym = self.syms.sym(base);

                let pool = Arena::tls_get_temp();
                let mut generics = sti::vec::Vec::with_cap_in(&*pool, base_sym.generics.len());
                if generics_list.is_empty() {
                    for _ in base_sym.generics {
                        generics.push(self.syms.new_var(id, dt.range()))
                    }

                } else {
                    for g in generics_list {
                        generics.push(self.dt_to_ty(scope_id, id, *g)?);
                    }

                    if generics.len() != base_sym.generics.len() {
                        return Err(Error::GenericLenMismatch {
                            source: dt.range(), found: generics.len(), expected: base_sym.generics.len() });
                    }
                };

                let ty = self.syms.get_ty(base, &*generics);
                Ok(ty)
            },


            DataTypeKind::Tuple(_) => todo!(),
        }
    }
}


impl TyInfo {
    pub fn set_stmt(&mut self, stmt: StmtId, info: ErrorId) {
        let val = &mut self.stmts[stmt];
        if val.is_none() {
            *val = Some(info)
        }
    }
    
    pub fn set_expr(&mut self, expr: ExprId, info: AnalysisResult) {
        let val = &mut self.exprs[expr];
        if val.is_none() || !matches!(info.ty, Type::Ty(SymbolId::ERR, GenListId::EMPTY)) {
            *val = Some(ExprInfo::Result { ty: info.ty, is_mut: info.is_mut })
        }
    }


    pub fn set_decl(&mut self, decl: DeclId, info: ErrorId) {
        let val = &mut self.decls[decl];
        if val.is_none() {
            *val = Some(info)
        }
    }


    pub fn set_func_call(&mut self, expr: ExprId, call: (SymbolId, GenListId)) {
        self.funcs.insert(expr, call);
    }


    pub fn expr(&self, expr: ExprId) -> Result<Type, ErrorId> {
        match self.exprs[expr].unwrap() {
            ExprInfo::Result { ty, .. } => return Ok(ty),
            ExprInfo::Errored(e) => return Err(e),
        }
    }


    pub fn stmt(&self, stmt: StmtId) -> Option<ErrorId> {
        self.stmts[stmt]
    }


    pub fn decl(&self, decl: DeclId) -> Option<ErrorId> {
        self.decls[decl]
    }
}
