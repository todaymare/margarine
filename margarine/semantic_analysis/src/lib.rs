use std::collections::HashMap;

use common::{buffer::Buffer, source::SourceRange, string_map::{StringIndex, StringMap}};
use errors::Error;
use ::errors::{ErrorId, SemaError};
use parser::{dt::{DataType, DataTypeKind}, nodes::{decl::{DeclGeneric, DeclId}, expr::ExprId, stmt::StmtId, NodeId, AST}};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, ext::FromIn, key::Key, vec::{KVec, Vec}};
use syms::{ty::Type, sym_map::{Generic, GenericKind, GenListId, SymbolId, SymbolMap}};
use namespace::{Namespace, NamespaceId, NamespaceMap};

use crate::{namespace::SymbolGetResult, scope::ScopeKind, syms::{containers::Container, func::{FunctionArgument, FunctionTy}, sym_map::{BoundedGeneric, ClosureId, VarId}, Symbol, SymbolKind}};

pub mod scope;
pub mod namespace;
pub mod errors;
pub mod analysis;
pub mod syms;
pub mod llvm_codegen;

pub struct SemaErrors {
    pub errors: KVec<SemaError, Error>,
    pub nodes: KVec<SemaError, NodeId>,
}

impl SemaErrors {
    pub fn new() -> Self {
        Self { errors: KVec::new(), nodes: KVec::new() }
    }

    pub fn push(&mut self, node: NodeId, error: Error) -> ErrorId {
        let id = self.errors.push(error);
        assert_eq!(id.0 as usize, self.nodes.len());
        self.nodes.push(node);
        ErrorId::Sema(id)
    }
}

pub struct TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    output      : &'out Arena,
    temp        : &'temp Arena,
    string_map  : &'me mut StringMap<'str>,
    ast         : &'me AST<'ast>,

    pub scopes      : ScopeMap<'out>,
    pub namespaces  : NamespaceMap,
    pub syms    : SymbolMap<'out>,
    pub type_info   : TyInfo<'out>,
    pub startups: Vec<SymbolId>,
    pub tests   : Vec<(SymbolId, bool)>,
    pub root_namespace: Option<NamespaceId>,

    pub errors     : SemaErrors,
    pub silent_ranges: std::vec::Vec<SourceRange>,
    control_flow: ControlFlowState,
    tuple_syms: std::vec::Vec<SymbolId>,
    base_scope  : ScopeId,
}


#[derive(Default)]
struct ControlFlowState {
    loops: std::vec::Vec<LoopContext>,
}


struct LoopContext {
    has_break: bool,
}


impl ControlFlowState {
    fn enter_loop(&mut self) {
        self.loops.push(LoopContext { has_break: false });
    }


    fn exit_loop(&mut self) -> LoopContext {
        self.loops.pop().expect("exiting a loop that was never entered")
    }


    fn mark_break(&mut self) -> bool {
        let Some(loop_context) = self.loops.last_mut()
        else { return false };

        loop_context.has_break = true;
        true
    }


    fn suspend(&mut self) -> Self {
        std::mem::take(self)
    }


    fn restore(&mut self, previous: Self) {
        *self = previous;
    }
}


#[derive(Debug)]
pub struct TyInfo<'out> {
    pub exprs: KVec<ExprId, Option<ExprInfo>>,
    stmts: KVec<StmtId, Option<ErrorId>>,
    decls: KVec<DeclId, Option<ErrorId>>,
    funcs: HashMap<ExprId, (SymbolId, GenListId)>,
    idents: HashMap<ExprId, Option<SymbolId>>,
    trait_funcs: HashMap<ExprId, Generic<'out>>,
    impls: HashMap<DeclId, (Generic<'out>, Generic<'out>, &'out [BoundedGeneric<'out>])>,
}


#[derive(Debug, Clone, Copy)]
pub struct ExprInfo {
    pub ty: Type,
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Type,
    is_mut: bool,
    is_captured: bool,
}

impl AnalysisResult {
    pub fn new(ty: Type) -> Self { Self { ty, is_mut: true, is_captured: false } }
    pub fn captured(ty: Type) -> Self { Self { ty, is_mut: false, is_captured: true } }
    pub fn never() -> Self { Self::new(Type::NEVER) }
}


impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub fn run(out: &'out Arena,
               temp: &'temp Arena,
               ast: &'me mut AST<'ast>,
               block: &[NodeId],
               string_map: &'me mut StringMap<'str>) -> Self {
        let mut ns = NamespaceMap::new();
        let errors = SemaErrors::new();
        let mut analyzer = TyChecker {
            output: out,
            syms: SymbolMap::new(out, &mut ns, string_map),
            string_map,
            namespaces: ns,
            scopes: ScopeMap::new(),
            errors,
            silent_ranges: std::vec::Vec::new(),
            control_flow: ControlFlowState::default(),
            tuple_syms: std::vec::Vec::new(),
            type_info: TyInfo {
                exprs: KVec::new(),
                stmts: KVec::new(),
                decls: KVec::new(),
                funcs: HashMap::new(),
                idents: HashMap::new(),
                trait_funcs: HashMap::new(),
                impls: HashMap::new(),
            },
            ast,
            startups: Vec::new(),
            tests: Vec::new(),
            root_namespace: None,
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
                    namespace.add_sym_unchecked(StringMap::$n, SymbolId::$n, parser::nodes::decl::Visibility::Public)
                };
            }

            let _ = add_sym!(I64);
            let _ = add_sym!(BYTE);
            let _ = add_sym!(F64);
            let _ = add_sym!(BOOL);
            let _ = add_sym!(PTR);
            let _ = add_sym!(OPTION);
            let _ = add_sym!(RESULT);
            let _ = add_sym!(STR);
            let _ = add_sym!(RANGE);
            let _ = add_sym!(BUILTIN_TYPE_ID);
            let _ = add_sym!(BUILTIN_SIZE_OF);
            let _ = add_sym!(EQ_TRAIT);
            let _ = add_sym!(DESTROY_TRAIT);
            let _ = add_sym!(RC);
            let _ = add_sym!(BUILTIN_RC);
            let _ = add_sym!(RC_GET);
            let _ = add_sym!(RC_SET);
            let _ = add_sym!(PTR_ALLOC);
            let _ = add_sym!(PTR_FREE);
            let _ = add_sym!(PTR_READ);
            let _ = add_sym!(PTR_WRITE);
            let _ = add_sym!(PTR_NULL);
            let _ = add_sym!(PTR_OFFSET);
            let _ = add_sym!(PTR_CAST);
            let _ = add_sym!(PTR_DROP);
            let _ = add_sym!(PTR_WRITE_UNINIT);
            let _ = add_sym!(LIST_CONCAT);
            let _ = add_sym!(LIST_SLICE);
            let _ = add_sym!(LIST_LEN);
            let _ = add_sym!(LIST_ITER);
            let _ = add_sym!(BUILTIN_LIST_ITER);
            let _ = add_sym!(BUILTIN_LIST_ITER_NEXT);
            let _ = add_sym!(BUILTIN_FLOAT_SQRT);

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::OPTION));
                namespace.add_sym_unchecked(StringMap::SOME, ns.get_sym(StringMap::SOME).unwrap().unwrap(), parser::nodes::decl::Visibility::Public);
                namespace.add_sym_unchecked(StringMap::NONE, ns.get_sym(StringMap::NONE).unwrap().unwrap(), parser::nodes::decl::Visibility::Public);
            }

            {
                let ns = analyzer.namespaces.get_ns(analyzer.syms.sym_ns(SymbolId::RESULT));
                namespace.add_sym_unchecked(StringMap::OK, ns.get_sym(StringMap::OK).unwrap().unwrap(), parser::nodes::decl::Visibility::Public);
                namespace.add_sym_unchecked(StringMap::ERR, ns.get_sym(StringMap::ERR).unwrap().unwrap(), parser::nodes::decl::Visibility::Public);
            }

            analyzer.namespaces.push(namespace, None)
        };

        let scope = Scope::new(None, ScopeKind::ImplicitNamespace(core_ns));
        let scope = analyzer.scopes.push(scope);
        analyzer.base_scope = scope;

        let empty = analyzer.string_map.insert("");
        analyzer.block(empty, scope, block);

        let vars = analyzer.syms.vars().len();
        for idx in 0..vars {
            let idx = VarId(idx as _);
            let v = analyzer.syms.vars()[idx];

            if !v.is_concrete(&mut analyzer.syms)
            && v.is_root(&mut analyzer.syms) {
                let error = Error::UnableToInfer(v.range(), v.name());
                let error_id = analyzer.error(v.node(), error);
                let sym = analyzer.syms.error_sym(&mut analyzer.namespaces, error_id);
                let v = &mut analyzer.syms.vars_mut()[idx];
                v.set_sub(syms::sym_map::VarSub::Concrete(Type::Ty(sym, GenListId::EMPTY)));
            }
        }

        analyzer
    }


    /// Materializes the per-failure error type for `err`: a fresh error
    /// symbol carrying the id, so the type is an ordinary error type.
    fn error_type(&mut self, err: ErrorId) -> Type {
        Type::Ty(self.syms.error_sym(&mut self.namespaces, err), GenListId::EMPTY)
    }


    /// The error Generic for `err`: a fresh per-failure error symbol, so a
    /// failed type is an ordinary error-typed value.
    fn error_generic(&mut self, range: SourceRange, err: ErrorId) -> Generic<'out> {
        Generic::new(range, GenericKind::Sym(self.syms.error_sym(&mut self.namespaces, err), &[]))
    }


    fn error(&mut self, node: impl Into<NodeId>, error: Error) -> ErrorId {
        let node = node.into();
        let error = self.errors.push(node, error);
        match node {
            NodeId::Expr(id) => {
                // an errored expression records its error type; the first
                // error wins
                let already_err = self.type_info.exprs[id]
                    .map(|v| v.ty.is_err(&mut self.syms))
                    .unwrap_or(false);
                if !already_err {
                    let ty = self.error_type(error);
                    self.type_info.exprs[id] = Some(ExprInfo { ty });
                }
            },

            NodeId::Decl(v) => self.type_info.set_decl(v, error),
            NodeId::Stmt(v) => self.type_info.set_stmt(v, error),
            NodeId::Err(_) => (),
        };

        error
    }

    fn set_error(&mut self, node: impl Into<NodeId>, error: ErrorId) {
        let node = node.into();
        match node {
            NodeId::Expr(id) => self.type_info.exprs[id] = Some(ExprInfo { ty: self.error_type(error) }),
            NodeId::Decl(id) => self.type_info.set_decl(id, error),
            NodeId::Stmt(id) => self.type_info.set_stmt(id, error),
            NodeId::Err(_) => (),
        };
    }
    

    fn dt_to_gen(
        &mut self, node: impl Into<NodeId> + Copy, scope: Scope<'out>,
        dt: DataType, gens: &[BoundedGeneric<'out>]
    ) -> Generic<'out> {
        let mut used_gens = sti::vec::Vec::from_value(gens.len(), false);
        self.dt_to_gen_ex(node, scope, dt, gens, &mut used_gens)
    }

    /// Resolves a namespace lookup to a symbol, materializing a per-failure
    /// error symbol for failed lookups so the error propagates as a type.
    fn convert_symbol_get_result(
        &mut self,
        node: impl Into<NodeId> + Copy,
        name: StringIndex,
        source: SourceRange,
        result: SymbolGetResult,
    ) -> SymbolId {
        match result {
            SymbolGetResult::Symbol(sym) => sym,
            SymbolGetResult::Errored(err) => self.syms.error_sym(&mut self.namespaces, err),
            SymbolGetResult::Private => {
                let err = self.error(node, Error::PrivateSymbol { source, name });
                self.syms.error_sym(&mut self.namespaces, err)
            },
            SymbolGetResult::Undefined => {
                let err = self.error(node, Error::NamespaceNotFound {
                    source, namespace: name,
                });
                self.syms.error_sym(&mut self.namespaces, err)
            },
        }
    }

    fn dt_to_gen_ex(
        &mut self, node: impl Into<NodeId> + Copy, scope: Scope<'out>, dt: DataType,
        gens: &[BoundedGeneric<'out>], used_gens: &mut [bool]
    ) -> Generic<'out> {
        match dt.kind() {
            DataTypeKind::Unit => Generic::new(dt.range(), GenericKind::Sym(SymbolId::UNIT, &[])),

            DataTypeKind::Hole => {
                let err = self.error(node, Error::CantUseHoleHere { source: dt.range() });
                self.error_generic(dt.range(), err)
            },

            DataTypeKind::Never => Generic::new(dt.range(), GenericKind::Sym(SymbolId::NEVER, &[])),

            DataTypeKind::Fn(args, ret) => {
                let mut func_used_gens = sti::vec::Vec::from_value(gens.len(), false);
                let fields = {
                    let mut fields = Buffer::new(&*self.output, args.len());
                    for (i, arg) in args.iter().enumerate() {
                        let g = self.dt_to_gen_ex(
                            node, scope, arg.data_type(), gens, &mut func_used_gens);
                        let func = FunctionArgument::new_inout(
                            self.string_map.num(i), g, arg.is_inout());
                        fields.push(func);
                    }
                    fields.leak()
                };

                let ret = self.dt_to_gen_ex(node, scope, *ret, gens, &mut func_used_gens);

                for (i, ug) in func_used_gens.iter().enumerate() {
                    if *ug {
                        used_gens[i] = true;
                    }
                }

                let mut gs = Buffer::new(&*self.output, gens.len());
                let mut fg = Buffer::new(&*self.output, gens.len());
                for (i, g) in gens.iter().enumerate() {
                    if func_used_gens[i as u32] {
                        gs.push(Generic::new(dt.range(), GenericKind::Generic(*g)));
                        fg.push(*g);
                    }
                }

                let closure = self.syms.new_closure();
                let fg = fg.leak();
                let sym = self.func_sym(closure, fields, ret, fg, fg);
                Generic::new(dt.range(), GenericKind::Sym(sym, gs.leak()))
            }

            DataTypeKind::Tuple(tys) => {
                let pool = self.output;
                let (fields, generics) = {
                    let mut fields = Buffer::new(&*pool, tys.len());
                    let mut generics = Buffer::new(self.output, tys.len());
                    for ty in tys {
                        let g = self.dt_to_gen_ex(node, scope, ty.1, gens, used_gens);
                        fields.push(ty.0);
                        generics.push(g);
                    }
                    (fields.leak(), generics.leak())
                };

                let sym = self.tuple_sym(dt.range(), fields);
                Generic::new(dt.range(), GenericKind::Sym(sym, generics))
            },

            DataTypeKind::List(ty) => {
                let ty = self.dt_to_gen_ex(node, scope, *ty, gens, used_gens);
                let gens = self.output.alloc_new([ty]);
                Generic::new(dt.range(), GenericKind::Sym(SymbolId::LIST, gens))
            },

            DataTypeKind::Within(ns_name, ty) => {
                let ns = scope.find_sym(
                    ns_name, &self.scopes,
                    &mut self.syms, &self.namespaces
                );
                let ns = self.convert_symbol_get_result(node, ns_name, ty.range(), ns);
                let ns = self.syms.sym_ns(ns);
                let scope = self.scopes.push(scope);
                let scope = Scope::new(scope, ScopeKind::QualifiedNamespace(ns));
                self.dt_to_gen_ex(node, scope, *ty, gens, used_gens)
            },

            DataTypeKind::CustomType(name, generics) => {
                if name == StringMap::SELF_TY
                && let Some(sym) = scope.find_self(&self.scopes) {
                    return sym
                }

                if let Some((i, g)) = gens.iter().enumerate().find(|x| x.1.name() == name) {
                    used_gens[i] = true;
                    return Generic::new(dt.range(), GenericKind::Generic(*g))
                }

                let base = scope.find_sym(
                    name, &self.scopes,
                    &mut self.syms, &self.namespaces
                );
                let base = self.convert_symbol_get_result(node, name, dt.range(), base);
                let genc = self.syms.sym_gens_size(base);

                if genc != generics.len() && !self.syms.is_err_sym(base) {
                    let err = self.error(node, Error::GenericLenMismatch {
                        source: dt.range(), found: generics.len(), expected: genc });
                    return self.error_generic(dt.range(), err);
                }

                let generics = {
                    let mut vec = Buffer::new(&*self.output, generics.len());
                    for g in generics {
                        vec.push(self.dt_to_gen_ex(node, scope, *g, gens, used_gens));
                    }
                    vec
                };

                Generic::new(dt.range(), GenericKind::Sym(base, generics.leak()))
            },

        }
    }




    fn dt_to_ty(&mut self, scope_id: ScopeId, id: impl Into<NodeId> + Copy,
                dt: DataType) -> Type {
        match dt.kind() {
            DataTypeKind::Unit => Type::UNIT,
            DataTypeKind::Never => Type::NEVER,
            DataTypeKind::Hole => self.syms.new_var(id, StringMap::HOLE, dt.range()),


            DataTypeKind::Within(ns_name, ty) => {
                let scope = self.scopes.get(scope_id);

                let result = scope.find_sym(ns_name, &self.scopes, &mut self.syms, &self.namespaces);
                let ns = self.convert_symbol_get_result(id, ns_name, ty.range(), result);
                let ns = self.syms.sym_ns(ns);

                let scope = Scope::new(scope_id, ScopeKind::QualifiedNamespace(ns));
                let scope = self.scopes.push(scope);
                self.dt_to_ty(scope, id, *ty)
            },


            DataTypeKind::CustomType(name, generics_list) => {
                let scope = self.scopes.get(scope_id);
                if let Some(sym) = scope.find_gen(name, &self.scopes) {
                    return sym
                }

                let result = scope.find_sym(name, &self.scopes, &mut self.syms, &self.namespaces);
                let base = self.convert_symbol_get_result(id, name, dt.range(), result);
                let Some(base_sym) = self.syms.sym_ok(base)
                else { return Type::Ty(base, GenListId::EMPTY) };
                if let SymbolKind::Alias(alias) = base_sym.kind()
                && let Some(error) = alias.to_ty(&[], &mut self.syms).as_err(&mut self.syms) {
                    return self.error_type(error);
                }

                let pool = self.output;
                let generics = if generics_list.is_empty() {
                    let mut generics = Buffer::new(&*pool, base_sym.generics().len());
                    for g in base_sym.generics() {
                        generics.push(self.syms.new_var(id, g.name(), dt.range()));
                    }
                    generics
                } else {
                    let mut generics = Buffer::new(&*pool, generics_list.len());
                    for g in generics_list {
                        generics.push(self.dt_to_ty(scope_id, id, *g));
                    }

                    if generics.len() != base_sym.generics().len() && !self.syms.is_err_sym(base) {
                        let err = self.error(id, Error::GenericLenMismatch {
                            source: dt.range(), found: generics.len(),
                            expected: base_sym.generics().len() });
                        return self.error_type(err);
                    }
                    generics
                };

                self.syms.get_ty(base, &*generics)
            },


            DataTypeKind::List(ty) => {
                let ty = self.dt_to_ty(scope_id, id, *ty);

                let gens = self.syms.add_gens(self.output.alloc_new([(BoundedGeneric::T, ty)]));
                Type::Ty(SymbolId::LIST, gens)
            },


            DataTypeKind::Fn(args, ret) => {
                let fields = {
                    let mut fields = Buffer::new(&*self.output, args.len());
                    for (i, arg) in args.iter().enumerate() {
                        let g = self.dt_to_gen(id, self.scopes.get(scope_id), arg.data_type(), &[]);
                        let func = FunctionArgument::new_inout(self.string_map.num(i), g, arg.is_inout());
                        fields.push(func);
                    }


                    fields.leak()
                };

                let ret = self.dt_to_gen(id, self.scopes.get(scope_id), *ret, &[]);

                let closure = self.syms.new_closure();
                let sym = self.func_sym(closure, fields, ret, &[], &[]);
                Type::Ty(sym, GenListId::EMPTY)
            }


            DataTypeKind::Tuple(vals) => {
                let pool = self.output;
                let (fields, generics) = {
                    let mut fields = Buffer::new(&*pool, vals.len());
                    let mut generics = Buffer::new(self.output, vals.len());
                    for (index, ty) in vals.iter().enumerate() {
                        let index = self.string_map.num(index);

                        let g = self.dt_to_ty(scope_id, id, ty.1);
                        fields.push(ty.0);
                        generics.push((BoundedGeneric::new(index, &[]), g));
                    }

                    (fields.leak(), generics.leak())
                };

                let sym = self.tuple_sym(dt.range(), fields);
                let generics = self.syms.add_gens(generics);

                Type::Ty(sym, generics)
            },
        }
    }


    fn resolve_generics(
        &mut self, 
        scope_id: ScopeId, 
        id: NodeId, 
        generics: &[DeclGeneric<'ast>]
    ) -> Result<&'out [BoundedGeneric<'out>], ErrorId> {

        let mut gens = sti::vec::Vec::with_cap_in(self.output, generics.len());

        for &g in generics.iter() {
            gens.push(self.resolve_generic(scope_id, id, g, &gens)?);
        }

        Ok(gens.leak_slice())
    }


    fn resolve_generic(
        &mut self, scope_id: ScopeId, 
        id: NodeId, generic: DeclGeneric<'ast>,
        prev_gens: &[BoundedGeneric<'out>],
    ) -> Result<BoundedGeneric<'out>, ErrorId> {

        let mut bounds = sti::vec::Vec::with_cap_in(self.output, generic.bounds().len());

        for bound in generic.bounds() {
            let g = self.dt_to_gen(id, self.scopes.get(scope_id), *bound, prev_gens);
            bounds.push(g);
        }

        Ok(BoundedGeneric::new(generic.name(), bounds.leak()))
    }


    fn tuple_sym(&mut self, range: SourceRange, fields: &[Option<StringIndex>]) -> SymbolId {
        while self.tuple_syms.len() <= fields.len() {
            let arity = self.tuple_syms.len();
            let pending = self.syms.pending(&mut self.namespaces, None, StringMap::INVALID_IDENT, arity);
            let (fields, gens) = {
                let mut sym_fields = Buffer::new(self.output, arity);
                let mut gens = Buffer::new(self.output, arity);
                for index in 0..arity {
                    let str = self.string_map.num(index);
                    let str = BoundedGeneric::new(str, &[]);
                    gens.push(str);

                    let g = Generic::new(range, GenericKind::Generic(str));
                    sym_fields.push((self.string_map.num(index), g));
                }

                (sym_fields.leak(), gens.leak())
            };

            let cont = Container::new(fields, syms::containers::ContainerKind::Tuple);
            let sym = Symbol::new(StringMap::TUPLE, gens, syms::SymbolKind::Container(cont));
            self.syms.add_sym(pending, sym);
            self.tuple_syms.push(pending);
        }

        self.tuple_syms[fields.len()]
    }


    fn tuple_gens(&mut self, count: usize, source: SourceRange, id: NodeId) -> GenListId {
        let gens = Vec::from_in(self.output, 
            (0..count).map(|i| {
                let name = self.string_map.num(i);
                let s = self.syms.new_var(id, name, source);
                (BoundedGeneric::new(name, &[]), s)
            })
        );

        self.syms.add_gens(gens.leak_slice())
    }


    fn func_sym(
        &mut self,
        closure: ClosureId,
        fields: &'out [FunctionArgument<'out>],
        ret: Generic<'out>,
        symbol_gens: &'out [BoundedGeneric<'out>],
        declared_gens: &'out [BoundedGeneric<'out>],
    ) -> SymbolId {

        let func = FunctionTy::new(
            fields, ret, syms::func::FunctionKind::Closure(closure), None, declared_gens);
        let sym = Symbol::new(StringMap::CLOSURE, symbol_gens, syms::SymbolKind::Function(func));
        let id = self.syms.pending(&mut self.namespaces, None, StringMap::CLOSURE, symbol_gens.len());
        self.syms.add_sym(id, sym);

        id
    }
}


impl<'out> TyInfo<'out> {
    pub fn set_stmt(&mut self, stmt: StmtId, info: ErrorId) {
        let val = &mut self.stmts[stmt];
        if val.is_none() {
            *val = Some(info)
        }
    }
    
    pub fn set_expr(&mut self, expr: ExprId, ty: Type) {
        self.exprs[expr] = Some(ExprInfo { ty })
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
        if self.idents.contains_key(&expr) {
            return;
        }

        self.idents.insert(expr, call);
    }


    pub fn set_acc(&mut self, expr: ExprId, tra: Generic<'out>) {
        self.trait_funcs.insert(expr, tra);
    }


    pub fn expr(&self, expr: ExprId) -> Type {
        self.exprs[expr].unwrap().ty
    }


    pub fn stmt(&self, stmt: StmtId) -> Option<ErrorId> {
        self.stmts[stmt]
    }


    pub fn decl(&self, decl: DeclId) -> Option<ErrorId> {
        self.decls[decl]
    }
    pub fn ident(&self, expr: ExprId) -> Option<Option<SymbolId>> {
        self.idents.get(&expr).copied()
    }

    pub fn func_call(&self, expr: ExprId) -> Option<(SymbolId, GenListId)> {
        self.funcs.get(&expr).copied()
    }

}
