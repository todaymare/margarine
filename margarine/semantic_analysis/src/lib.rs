use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use errors::Error;
use ::errors::{ErrorId, SemaError};
use funcs::{FunctionMap, FunctionSymbolId};
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::{decl::DeclId, expr::ExprId, stmt::StmtId, NodeId, AST}, dt::{DataType, DataTypeKind}};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, keyed::KVec};
use types::{Generic, GenericKind, SymbolMap, Type, SymbolId};

use crate::scope::ScopeKind;

pub mod scope;
pub mod namespace;
pub mod funcs;
pub mod types;
pub mod errors;
pub mod analysis;

#[derive(Debug)]
pub struct TyChecker<'me, 'out, 'ast, 'str> {
    output    : &'out Arena,
    pub string_map: &'me mut StringMap<'str>,
    ast       : &'me mut AST<'ast>,

    scopes    : ScopeMap<'out>,
    namespaces: NamespaceMap,
    pub types : SymbolMap<'out>,
    funcs     : FunctionMap<'out, 'ast>,
    type_info : TyInfo,

    pub errors    : KVec<SemaError, Error>,
}


#[derive(Debug)]
pub struct TyInfo {
    exprs: KVec<ExprId, Option<ExprInfo>>,
    stmts: KVec<StmtId, Option<ErrorId>>,
    decls: KVec<DeclId, Option<ErrorId>>,
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
            string_map,
            scopes: ScopeMap::new(),
            types: SymbolMap::new(out, &mut ns),
            namespaces: ns,
            funcs: FunctionMap::new(out),
            errors: KVec::new(),
            type_info: TyInfo {
                exprs: KVec::new(),
                stmts: KVec::new(),
                decls: KVec::new(),
            },
            ast,
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

            analyzer.namespaces.push(namespace)
        };

        let scope = Scope::new(None, ScopeKind::ImplicitNamespace(core_ns));
        let scope = analyzer.scopes.push(scope);
        let empty = analyzer.string_map.insert("");
        analyzer.block(empty, scope, block);

        analyzer
    }


    fn error(&mut self, node: impl Into<NodeId>, error: Error) {
        let error = self.errors.push(error);
        let error = ErrorId::Sema(error);
        match node.into() {
            NodeId::Expr(id) => {
                let val = &mut self.type_info.exprs[id];
                match val {
                    Some(v) => match v {
                        ExprInfo::Result { .. } => *val = Some(ExprInfo::Errored(error)),
                        ExprInfo::Errored(_) => (),
                    },
                    None => *val = Some(ExprInfo::Errored(error)),
                };
            },

            NodeId::Decl(v) => self.type_info.set_decl(v, error),
            NodeId::Stmt(v) => self.type_info.set_stmt(v, error),
            NodeId::Err(_) => (),
        }
    }

    
    fn gen_to_ty(&mut self, gen: Generic, gens: &[(StringIndex, Type)]) -> Result<Type, Error> {
        match gen.kind {
            GenericKind::Generic(v) => gens.iter()
                                        .find(|x| x.0 == v)
                                        .copied()
                                        .map(|x| x.1)
                                        .ok_or(Error::UnknownType(v, gen.range)),

            GenericKind::Sym(symbol, generics) => {
                let pool = Arena::tls_get_rec();
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());
                    for g in generics {
                        vec.push(self.gen_to_ty(*g, gens).unwrap_or(Type::ERROR));
                    }
                    vec
                };
                
                Ok(self.get_ty(symbol, &generics))
            },
        }
    }


    fn dt_to_gen(&mut self, scope: Scope, dt: DataType,
                 gens: &[StringIndex]) -> Result<Generic<'out>, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::UNIT, &[]))),


            DataTypeKind::Never => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::NEVER, &[]))),


            DataTypeKind::Tuple(v) => todo!(),


            DataTypeKind::Within(ns, dt) => {
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns));
                self.dt_to_gen(scope, *dt, gens)
            },


            DataTypeKind::CustomType(name, generics) => {
                if gens.contains(&name) { return Ok(Generic::new(dt.range(),
                                                    GenericKind::Generic(name))) }

                let Some(base) = scope.find_ty(name, &self.scopes, &mut self.types, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

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


    fn dt_to_ty(&mut self, scope_id: ScopeId,
                dt: DataType) -> Result<Type, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Type::UNIT),
            DataTypeKind::Never => Ok(Type::UNIT),


            DataTypeKind::Within(ns, dt) => {
                let scope = self.scopes.get(scope_id);
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns));
                let scope = self.scopes.push(scope);
                self.dt_to_ty(scope, *dt)
            },


            DataTypeKind::CustomType(name, generics_list) => {
                let scope = self.scopes.get(scope_id);
                if let Some(gen) = scope.find_gen(name, &self.scopes) {
                    // TODO: error
                    assert!(generics_list.is_empty());
                    return Ok(gen);
                }


                let Some(base) = scope.find_ty(name, &self.scopes, &mut self.types, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let base_sym = self.types.sym(base);

                let pool = Arena::tls_get_temp();
                let mut generics = sti::vec::Vec::with_cap_in(&*pool, base_sym.generics.len());
                if generics_list.is_empty() {
                    for _ in base_sym.generics {
                        generics.push(self.types.new_var(dt.range()))
                    }
                } else {
                    for g in generics_list {
                        generics.push(self.dt_to_ty(scope_id, *g)?);
                    }

                    if generics.len() != base_sym.generics.len() {
                        panic!("todo: add error");
                    }
                };

                let ty = self.get_ty(base, &*generics);
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
        if val.is_none() {
            *val = Some(ExprInfo::Result { ty: info.ty, is_mut: info.is_mut })
        }
    }


    pub fn set_decl(&mut self, decl: DeclId, info: ErrorId) {
        let val = &mut self.decls[decl];
        if val.is_none() {
            *val = Some(info)
        }
    }
}
