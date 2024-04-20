use std::collections::HashMap;

use common::string_map::{StringIndex, StringMap};
use errors::Error;
use ::errors::SemaError;
use funcs::{FunctionId, FunctionMap};
use llvm_api::{builder::Builder, tys::IsType, values::Value, Context, Function, Module};
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::Node, DataType, DataTypeKind};
use scope::{Scope, ScopeId, ScopeMap};
use sti::{arena::Arena, hash::DefaultSeed, keyed::KVec, packed_option::PackedOption};
use types::{Generic, GenericKind, SymbolMap, Type, TypeSymbol, TypeSymbolId};

use crate::scope::ScopeKind;

pub mod scope;
pub mod namespace;
pub mod funcs;
pub mod types;
pub mod errors;
pub mod analysis;

#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'ast, 'str> {
    output    : &'out Arena,
    pub string_map: &'me mut StringMap<'str>,

    pub module    : Module,

    scopes    : ScopeMap<'out>,
    namespaces: NamespaceMap,
    pub types     : SymbolMap<'out>,
    funcs     : FunctionMap<'out, 'ast>,

    pub errors    : KVec<SemaError, Error>,
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty    : Type,
    value : Value,
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(ty: Type, value: Value, is_mut: bool) -> Self { Self { ty, value, is_mut } }
}


impl<'me, 'out, 'ast, 'str> Analyzer<'me, 'out, 'ast, 'str> {
    pub fn run(out: &'out Arena,
               ast: &[Node<'ast>],
               string_map: &'me mut StringMap<'str>) -> (Self, Context) {
        let mut ctx = Context::new();
        let module = Module::new(&mut ctx, "main");
        ctx.module_mut(module).target(llvm_api::Target::Wasm);
        let mut analyzer = Analyzer {
            output: out,
            string_map,
            module,
            scopes: ScopeMap::new(),
            namespaces: NamespaceMap::new(),
            types: SymbolMap::new(out, &mut ctx),
            funcs: FunctionMap::new(out),
            errors: KVec::new(),
        };

        let core_ns = {
            let mut namespace = Namespace::new(analyzer.string_map.insert("::core"));

            macro_rules! add_sym {
                ($n: ident) => {
                    namespace.add_sym(StringMap::$n, TypeSymbolId::$n);
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

        let ret = ctx.zst();
        let mut builder = Function::new(&mut ctx, module, "init", ret.ty(), &[]);
        let scope = Scope::new(None, ScopeKind::ImplicitNamespace(core_ns));
        let scope = analyzer.scopes.push(scope);
        let empty = analyzer.string_map.insert("");
        analyzer.block(&mut builder, empty, scope, ast);

        let ret = builder.unit();
        builder.ret(ret);
        builder.build();

        dbg!(&analyzer);
        (analyzer, ctx)
    }


    fn error(&mut self, builder: &mut Builder, error: Error) -> AnalysisResult {
        let id = self.errors.push(error);
        self.place_error(builder, ::errors::ErrorId::Sema(id));
        self.empty_error(builder)
    }


    fn place_error(&mut self, builder: &mut Builder, err: ::errors::ErrorId) {
        // TODO: Print and panic on error
    }


    fn empty_error(&mut self, builder: &mut Builder) -> AnalysisResult {
        AnalysisResult::new(Type::ERROR, builder.unit(), true)
    }


    fn never(&mut self, builder: &mut Builder) -> AnalysisResult {
        AnalysisResult::new(Type::NEVER, builder.unit(), true)
    }

    
    fn gen_to_ty(&mut self, ctx: &mut Context, 
                 gen: Generic, gens: &HashMap<StringIndex, Type>) -> Result<Type, Error> {
        match gen.kind {
            GenericKind::Generic(v) => gens.get(&v)
                                            .map(|x| self.types.get_ty_val(*x).symbol())
                                            .map(|x| self.get_ty(ctx, x, &[]))
                                            .ok_or(Error::UnknownType(v, gen.range)),

            GenericKind::Symbol { symbol, generics } => {
                let pool = Arena::tls_get_rec();
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());
                    for g in generics {
                        vec.push(self.gen_to_ty(ctx, *g, gens).unwrap_or(Type::ERROR));
                    }
                    vec
                };
                
                Ok(self.get_ty(ctx, symbol, &generics))
            },
        }
    }


    fn dt_to_gen(&mut self, ctx: &mut Context, scope: Scope,
                 dt: DataType, gens: &[StringIndex]) -> Result<Generic<'out>, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Generic::new(dt.range(), GenericKind::Symbol {
                                        symbol: TypeSymbolId::UNIT, generics: &[] })),


            DataTypeKind::Never => Ok(Generic::new(dt.range(), GenericKind::Symbol {
                                        symbol: TypeSymbolId::NEVER, generics: &[] })),


            DataTypeKind::Tuple(v) => {
                let pool = Arena::tls_get_rec();
                let names = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, v.len());
                    for i in v { vec.push(i.0) }
                    vec.leak()
                };

                let tuple = self.tuple_sym(names);

                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*self.output, v.len());

                    for g in v {
                        vec.push(self.dt_to_gen(ctx, scope, g.1, gens)?);
                    }
                    vec
                };

                Ok(Generic::new(dt.range(),
                                GenericKind::Symbol { symbol: tuple, generics: generics.leak() }))
            },


            DataTypeKind::Within(ns, dt) => {
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns));
                self.dt_to_gen(ctx, scope, *dt, gens)
            },


            DataTypeKind::CustomType(name, generics) => {
                if gens.contains(&name) { return Ok(Generic::new(dt.range(),
                                                    GenericKind::Generic(name))) }

                let Some(base) = scope.find_ty(name, &self.scopes, &self.types, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*self.output, generics.len());

                    for g in generics {
                        vec.push(self.dt_to_gen(ctx, scope, *g, gens)?);
                    }
                    vec
                };

                Ok(Generic::new(dt.range(),
                                GenericKind::Symbol { symbol: base, generics: generics.leak() }))
            },
        }
    }


    fn dt_to_ty(&mut self, ctx: &mut Context, scope_id: ScopeId,
                dt: DataType) -> Result<Type, Error> {
        match dt.kind() {
            DataTypeKind::Unit => Ok(Type::UNIT),
            DataTypeKind::Never => todo!(),


            DataTypeKind::Within(ns, dt) => {
                let scope = self.scopes.get(scope_id);
                let Some(ns) = scope.find_ns(ns, &self.scopes, &self.namespaces)
                else { return Err(Error::NamespaceNotFound { namespace: ns, source: dt.range() }) };

                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(ns));
                let scope = self.scopes.push(scope);
                self.dt_to_ty(ctx, scope, *dt)
            },


            DataTypeKind::CustomType(name, generics) => {
                let scope = self.scopes.get(scope_id);
                let Some(base) = scope.find_ty(name, &self.scopes, &self.types, &self.namespaces)
                else { return Err(Error::UnknownType(name, dt.range())) };

                let pool = Arena::tls_get_temp();
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());

                    for g in generics {
                        vec.push(self.dt_to_ty(ctx, scope_id, *g)?);
                    }
                    vec
                };

                let ty = self.get_ty(ctx, base, &*generics);
                Ok(ty)
            },


            DataTypeKind::Tuple(v) => {
                let pool = Arena::tls_get_rec();
                let mut vec = sti::vec::Vec::with_cap_in(&*pool, v.len());
                for i in v {
                    let ty = self.dt_to_ty(ctx, scope_id, i.1)?;
                    vec.push((i.0, ty));
                }

                Ok(self.tuple(ctx, &*vec))
            },
        }
    }
}

