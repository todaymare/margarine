use std::{collections::HashMap, fmt::Write};

use common::{buffer::Buffer, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use errors::Error;
use ::errors::{ErrorId, SemaError};
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::{decl::DeclId, expr::ExprId, stmt::StmtId, NodeId, AST}, dt::{DataType, DataTypeKind}};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, keyed::KVec, string::String, vec::Vec, write};
use syms::{ty::Sym, sym_map::{Generic, GenericKind, GenListId, SymbolId, SymbolMap}};

use crate::{scope::ScopeKind, syms::{containers::Container, Symbol}};

pub mod scope;
pub mod namespace;
pub mod errors;
pub mod analysis;
//pub mod codegen;
pub mod global;
pub mod syms;
pub mod codegen;

#[derive(Debug)]
pub struct TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    output      : &'out Arena,
    temp        : &'temp Arena,
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
        ty    : Sym,
    },

    Errored(ErrorId),
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Sym,
}

impl AnalysisResult {
    pub fn new(ty: Sym) -> Self { Self { ty } }
    pub fn error() -> Self { Self::new(Sym::ERROR) }
    pub fn never() -> Self { Self::new(Sym::NEVER) }
}


impl<'me, 'out, 'temp, 'ast, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub fn run(out: &'out Arena,
               temp: &'temp Arena,
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
            temp,
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

            add_sym!(I64);
            add_sym!(F64);
            add_sym!(BOOL);
            add_sym!(PTR);
            add_sym!(OPTION);
            add_sym!(RESULT);
            add_sym!(STR);
            add_sym!(RANGE);
            add_sym!(TYPE_ID);

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

        for v in analyzer.syms.vars().iter() {
            if !matches!(v.1.sub(), syms::sym_map::VarSub::Concrete(_)) {
                let error = Error::UnableToInfer(analyzer.ast.range(v.1.node()));
                Self::error_ex(&mut analyzer.errors, &mut analyzer.type_info, v.1.node(), error)
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


            DataTypeKind::Hole => Err(Error::CantUseHoleHere { source: dt.range() }),


            DataTypeKind::Never => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::NEVER, &[]))),


            DataTypeKind::Tuple(tys) => {
                let pool = Arena::tls_get_rec();
                let (fields, generics) = {
                    let mut fields = Buffer::new(&*pool, tys.len());
                    let mut generics = Buffer::new(self.output, tys.len());
                    for ty in tys {
                        let g = self.dt_to_gen(scope, ty.1, gens)?;
                        fields.push(ty.0);
                        generics.push(g);
                    }

                    (fields.leak(), generics.leak())
                };

                let sym = self.tuple_sym(dt.range(), fields);

                Ok(Generic::new(dt.range(), GenericKind::Sym(sym, generics)))
            },


            DataTypeKind::List(ty) => {
                let ty = self.dt_to_gen(scope, *ty, gens)?;
                let gens = self.output.alloc_new([ty]);

                Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::LIST, gens)))
            },


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
                    let mut vec = Buffer::new(&*self.output, generics.len());

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
                dt: DataType) -> Result<Sym, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Sym::UNIT),
            DataTypeKind::Never => Ok(Sym::NEVER),
            DataTypeKind::Hole => Ok(self.syms.new_var(id, dt.range())),


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
                if let Some(sym) = scope.find_gen(name, &self.scopes) {
                    // @info: i got no idea why i had this error
                    // i have a feeling it might fuck me up later
                    //
                    // it did fuck me over
                    //return Err(Error::GenericOnGeneric { source: dt.range() });
                    return Ok(sym)
                }


                let Some(base) = scope.find_sym(name, &self.scopes, &mut self.syms, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let base = base?;

                let base_sym = self.syms.sym(base);

                let pool = Arena::tls_get_temp();
                let mut generics = Buffer::new(&*pool, base_sym.generics().len());
                if generics_list.is_empty() {
                    for _ in base_sym.generics() {
                        generics.push(self.syms.new_var(id, dt.range()));
                    }

                } else {
                    for g in generics_list {
                        generics.push(self.dt_to_ty(scope_id, id, *g)?);
                    }

                    if generics.len() != base_sym.generics().len() {
                        return Err(Error::GenericLenMismatch {
                            source: dt.range(), found: generics.len(),
                            expected: base_sym.generics().len() });
                    }
                };

                let ty = self.syms.get_ty(base, &*generics);
                Ok(ty)
            },


            DataTypeKind::List(ty) => {
                let ty = self.dt_to_ty(scope_id, id, *ty)?;

                let gens = self.syms.add_gens(self.output.alloc_new([(StringMap::T, ty)]));
                Ok(Sym::Ty(SymbolId::LIST, gens))
            },


            DataTypeKind::Tuple(vals) => {
                let pool = Arena::tls_get_rec();
                let (fields, generics) = {
                    let mut fields = Buffer::new(&*pool, vals.len());
                    let mut generics = Buffer::new(self.output, vals.len());
                    let mut str = String::new_in(&*pool);
                    for (index, ty) in vals.iter().enumerate() {
                        str.clear();
                        write!(str, "{index}");
                        let index = self.string_map.insert(&str);

                        let g = self.dt_to_ty(scope_id, id, ty.1)?;
                        fields.push(ty.0);
                        generics.push((index, g));
                    }

                    (fields.leak(), generics.leak())
                };

                let sym = self.tuple_sym(dt.range(), fields);
                let generics = self.syms.add_gens(generics);

                Ok(Sym::Ty(sym, generics))
            },
        }
    }


    fn tuple_sym(&mut self, range: SourceRange, fields: &[OptStringIndex]) -> SymbolId {
        let pool = Arena::tls_get_temp();
        let pending = self.syms.pending(&mut self.namespaces, StringMap::INVALID_IDENT, fields.len());
        let (fields, gens) = {
            let mut sym_fields = Buffer::new(self.output, fields.len());
            let mut gens = Buffer::new(self.output, fields.len());
            let mut str = String::new_in(&*pool);
            for (index, name) in fields.iter().enumerate() {
                str.clear();
                write!(str, "{index}");
                let index = self.string_map.insert(&str);
                gens.push(index);
                sym_fields.push((*name, Generic::new(range, GenericKind::Generic(index))));
            }

            (sym_fields.leak(), gens.leak())
        };


        let cont = Container::new(fields, syms::containers::ContainerKind::Tuple);
        let sym = Symbol::new(StringMap::TUPLE, gens, syms::SymbolKind::Container(cont));
        self.syms.add_sym(pending, sym);

        pending
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
        if val.is_none() || !matches!(info.ty, Sym::Ty(SymbolId::ERR, GenListId::EMPTY)) {
            *val = Some(ExprInfo::Result { ty: info.ty })
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


    pub fn expr(&self, expr: ExprId) -> Result<Sym, ErrorId> {
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
