#![feature(slice_partition_dedup)]
use std::collections::HashMap;

use common::{buffer::Buffer, source::SourceRange, string_map::{StringIndex, StringMap}};
use errors::Error;
use ::errors::{ErrorId, SemaError};
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::{decl::DeclId, expr::ExprId, stmt::StmtId, NodeId, AST}, dt::{DataType, DataTypeKind}};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, key::Key, vec::{KVec, Vec}};
use syms::{ty::Sym, sym_map::{Generic, GenericKind, GenListId, SymbolId, SymbolMap}};

use crate::{scope::ScopeKind, syms::{containers::Container, func::{FunctionArgument, FunctionTy}, sym_map::ClosureId, Symbol}};

pub mod scope;
pub mod namespace;
pub mod errors;
pub mod analysis;
pub mod syms;
pub mod codegen;

pub struct TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    output      : &'out Arena,
    temp        : &'temp Arena,
    pub string_map: &'me mut StringMap<'str>,
    ast         : &'me AST<'ast>,

    scopes      : ScopeMap<'out>,
    namespaces  : NamespaceMap,
    pub syms    : SymbolMap<'out>,
    type_info   : TyInfo,
    pub startups: Vec<SymbolId>,
    pub tests   : Vec<SymbolId>,

    pub errors     : KVec<SemaError, Error>,
    base_scope  : ScopeId,
}


#[derive(Debug)]
pub struct TyInfo {
    exprs: KVec<ExprId, Option<ExprInfo>>,
    stmts: KVec<StmtId, Option<ErrorId>>,
    decls: KVec<DeclId, Option<ErrorId>>,
    funcs: HashMap<ExprId, (SymbolId, GenListId)>,
    idents: HashMap<ExprId, Option<SymbolId>>,
}


#[derive(Debug, Clone, Copy)]
pub enum ExprInfo {
    Result {
        ty: Sym,
    },

    Errored(ErrorId),
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Sym,
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(ty: Sym) -> Self { Self { ty, is_mut: true } }
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
                idents: HashMap::new(),
            },
            ast,
            startups: Vec::new(),
            tests: Vec::new(),
            temp,
            base_scope: ScopeId::MIN,
        };

        {
            analyzer.type_info.exprs.resize(analyzer.ast.exprs().len(), None);
            analyzer.type_info.stmts.resize(analyzer.ast.stmts().len(), None);
            analyzer.type_info.decls.resize(analyzer.ast.decls().len(), None);
        }

        let core_ns = {
            let mut namespace = Namespace::new(analyzer.string_map.insert("::core"));

            macro_rules! add_sym {
                ($n: ident) => {
                    namespace.add_sym(SourceRange::ZERO, StringMap::$n, SymbolId::$n)
                };
            }

            let _ = add_sym!(I64);
            let _ = add_sym!(F64);
            let _ = add_sym!(BOOL);
            let _ = add_sym!(PTR);
            let _ = add_sym!(OPTION);
            let _ = add_sym!(RESULT);
            let _ = add_sym!(STR);
            let _ = add_sym!(RANGE);
            let _ = add_sym!(TYPE_ID);

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::OPTION));
                let _ = namespace.add_sym(SourceRange::ZERO, StringMap::SOME, ns.get_sym(StringMap::SOME).unwrap().unwrap());
                let _ = namespace.add_sym(SourceRange::ZERO, StringMap::NONE, ns.get_sym(StringMap::NONE).unwrap().unwrap());
            }

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::RESULT));
                let _ = namespace.add_sym(SourceRange::ZERO, StringMap::OK , ns.get_sym(StringMap::OK ).unwrap().unwrap());
                let _ = namespace.add_sym(SourceRange::ZERO, StringMap::ERR, ns.get_sym(StringMap::ERR).unwrap().unwrap());
            }

            analyzer.namespaces.push(namespace)
        };

        let scope = Scope::new(None, ScopeKind::ImplicitNamespace(core_ns));
        let scope = analyzer.scopes.push(scope);
        analyzer.base_scope = scope;
        let empty = analyzer.string_map.insert("");
        analyzer.block(empty, scope, block);

        for v in analyzer.syms.vars().iter() {
            if !matches!(v.sub(), syms::sym_map::VarSub::Concrete(_)) {
                let error = Error::UnableToInfer(analyzer.ast.range(v.node()));
                Self::error_ex(&mut analyzer.errors, &mut analyzer.type_info, v.node(), error);
            }
        }

        analyzer
    }


    fn error(&mut self, node: impl Into<NodeId>, error: Error) -> ErrorId {
        Self::error_ex(&mut self.errors, &mut self.type_info, node, error)
    }


    fn error_ex(errors: &mut KVec<SemaError, Error>, ty_info: &mut TyInfo, node: impl Into<NodeId>, error: Error) -> ErrorId {
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
        };

        error
    }
    

    fn dt_to_gen(&mut self, scope: Scope, dt: DataType,
                 gens: &[StringIndex]) -> Result<Generic<'out>, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::UNIT, &[]), None)),


            DataTypeKind::Hole => Err(Error::CantUseHoleHere { source: dt.range() }),


            DataTypeKind::Never => Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::NEVER, &[]), None)),


            DataTypeKind::Fn(args, ret) => {
                let fields = {
                    let mut fields = Buffer::new(&*self.output, args.len());
                    for (i, ty) in args.iter().enumerate() {
                        let g = self.dt_to_gen(scope, *ty, gens)?;
                        let func = FunctionArgument::new(self.string_map.num(i), g);
                        fields.push(func);
                    }


                    fields.leak()
                };

                let mut gs = Buffer::new(&*self.output, gens.len());
                for g in gens {
                    gs.push(Generic::new(dt.range(), GenericKind::Generic(*g), None));
                }

                let ret = self.dt_to_gen(scope, *ret, gens)?;

                let closure = self.syms.new_closure();
                let sym = self.func_sym(closure, fields, ret, Vec::from_slice_in(self.output, gens).leak());
                Ok(Generic::new(dt.range(), GenericKind::Sym(sym, gs.leak()), None))
            }


            DataTypeKind::Tuple(tys) => {
                let pool = self.output;
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

                Ok(Generic::new(dt.range(), GenericKind::Sym(sym, generics), None))
            },


            DataTypeKind::List(ty) => {
                let ty = self.dt_to_gen(scope, *ty, gens)?;
                let gens = self.output.alloc_new([ty]);

                Ok(Generic::new(dt.range(), GenericKind::Sym(SymbolId::LIST, gens), None))
            },


            DataTypeKind::Within(ns_name, ty) => {
                let ns = scope.find_sym(
                    ns_name, &self.scopes, 
                    &mut self.syms, &self.namespaces
                );

                let Some(ns) = ns
                else { 
                    return Err(Error::NamespaceNotFound { 
                        source: ty.range(), 
                        namespace: ns_name,
                    }) 
                };

                let Ok(ns) = ns
                else {
                    return Err(Error::Bypass);
                };

                let ns = self.syms.sym_ns(ns);


                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns));
                self.dt_to_gen(scope, *ty, gens)
            },


            DataTypeKind::CustomType(name, generics) => {
                if gens.contains(&name) { return Ok(Generic::new(dt.range(),
                                                    GenericKind::Generic(name), None)) }

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
                                GenericKind::Sym(base, generics.leak()), None))
            },
        }
    }


    fn dt_to_ty(&mut self, scope_id: ScopeId, id: impl Into<NodeId> + Copy,
                dt: DataType) -> Result<Sym, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Sym::UNIT),
            DataTypeKind::Never => Ok(Sym::NEVER),
            DataTypeKind::Hole => Ok(self.syms.new_var(id, dt.range())),


            DataTypeKind::Within(ns_name, ty) => {
                let scope = self.scopes.get(scope_id);

                let ns = scope.find_sym(
                    ns_name, &self.scopes, 
                    &mut self.syms, &self.namespaces
                );

                let Some(ns) = ns
                else { 
                    return Err(Error::NamespaceNotFound { 
                        source: ty.range(), 
                        namespace: ns_name,
                    }) 
                };

                let Ok(ns) = ns
                else {
                    return Err(Error::Bypass);
                };

                let ns = self.syms.sym_ns(ns);

                let scope = Scope::new(scope_id, ScopeKind::ImplicitNamespace(ns));
                let scope = self.scopes.push(scope);
                self.dt_to_ty(scope, id, *ty)
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

                let pool = self.output;
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


            DataTypeKind::Fn(args, ret) => {
                let fields = {
                    let mut fields = Buffer::new(&*self.output, args.len());
                    for (i, ty) in args.iter().enumerate() {
                        let g = self.dt_to_gen(self.scopes.get(scope_id), *ty, &[])?;
                        let func = FunctionArgument::new(self.string_map.num(i), g);
                        fields.push(func);
                    }


                    fields.leak()
                };

                let ret = self.dt_to_gen(self.scopes.get(scope_id), *ret, &[])?;

                let closure = self.syms.new_closure();
                let sym = self.func_sym(closure, fields, ret, &[]);
                Ok(Sym::Ty(sym, GenListId::EMPTY))
            }


            DataTypeKind::Tuple(vals) => {
                let pool = self.output;
                let (fields, generics) = {
                    let mut fields = Buffer::new(&*pool, vals.len());
                    let mut generics = Buffer::new(self.output, vals.len());
                    for (index, ty) in vals.iter().enumerate() {
                        let index = self.string_map.num(index);

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


    fn tuple_sym(&mut self, range: SourceRange, fields: &[Option<StringIndex>]) -> SymbolId {
        let pending = self.syms.pending(&mut self.namespaces, StringMap::INVALID_IDENT, fields.len());
        let (fields, gens) = {
            let mut sym_fields = Buffer::new(self.output, fields.len());
            let mut gens = Buffer::new(self.output, fields.len());
            for (index, name) in fields.iter().enumerate() {
                let str = self.string_map.num(index);
                gens.push(str);

                let name = name.unwrap_or_else(|| self.string_map.num(index));
                sym_fields.push((name, Generic::new(range, GenericKind::Generic(str), None)));
            }

            (sym_fields.leak(), gens.leak())
        };


        let cont = Container::new(fields, syms::containers::ContainerKind::Tuple);
        let sym = Symbol::new(StringMap::TUPLE, gens, syms::SymbolKind::Container(cont));
        self.syms.add_sym(pending, sym);

        pending
    }


    fn func_sym(
        &mut self,
        closure: ClosureId,
        fields: &'out [FunctionArgument<'out>],
        ret: Generic<'out>,
        gens: &'out [StringIndex],
    ) -> SymbolId {

        let func = FunctionTy::new(fields, ret, syms::func::FunctionKind::Closure(closure), None);
        let sym = Symbol::new(StringMap::CLOSURE, &gens, syms::SymbolKind::Function(func));
        let id = self.syms.pending(&mut self.namespaces, StringMap::CLOSURE, gens.len());
        self.syms.add_sym(id, sym);

        id
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


    pub fn set_ident(&mut self, expr: ExprId, call: Option<SymbolId>) {
        self.idents.insert(expr, call);
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
