use std::collections::{HashMap, HashSet};

use common::{copy_slice_in, source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use errors::ErrorId;
use parser::nodes::{decl::{DeclId, Visibility}, NodeId};
use sti::{arena::Arena, define_key, ext::FromIn, vec::KVec};

use crate::{namespace::{Namespace, NamespaceId, NamespaceMap}, syms::{containers::{Container, ContainerKind}, func::{FunctionArgument, FunctionKind, FunctionTy}, SymbolKind, Trait, TraitSynthesis}};

use super::{ty::Type, Symbol};

pub use common::symbol_id::SymbolId;
define_key!(pub GenListId(pub u32));
define_key!(pub VarId(pub u32));
define_key!(pub ClosureId(pub u32));

#[derive(Clone, Copy, Debug)]
pub struct TraitImplEntry<'me> {
    pub namespace: NamespaceId,
    pub trait_ty: Generic<'me>,
    pub receiver: Generic<'me>,
    pub generics: &'me [BoundedGeneric<'me>],
    pub declaration: Option<DeclId>,
    pub bound_error: Option<ErrorId>,
}

pub type TraitMap<'me> = HashMap<SymbolId, Vec<TraitImplEntry<'me>>>;

pub struct SymbolMap<'me> {
    syms : KVec<SymbolId, (Result<Symbol<'me>, usize>, NamespaceId, TraitMap<'me>)>,
    gens : KVec<GenListId, &'me [(BoundedGeneric<'me>, Type)]>,
    vars : KVec<VarId, Var>,
    closures: KVec<ClosureId, Closure>,
    arena: &'me Arena,
}


#[derive(Debug)]
pub struct Closure {
    pub captured_variables: HashSet<(StringIndex, Type)>,
}


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Var {
    name: Option<StringIndex>,
    sub: VarSub,
    node: NodeId,
    range: SourceRange,
}


#[derive(Debug, Clone, Copy)]
pub enum VarSub {
    Concrete(Type),
    None,
}


#[derive(Clone, Copy, Debug, ImmutableData)]
pub struct Generic<'me> {
    range: SourceRange,
    kind : GenericKind<'me>,
}


#[derive(Clone, Copy, Debug, PartialEq, ImmutableData)]
pub struct BoundedGeneric<'me> {
    pub name: StringIndex,
    pub bounds: &'me [Generic<'me>],
}


impl<'me> BoundedGeneric<'me> {
    pub const T : Self = Self::new(StringMap::T, &[]);
    pub const A : Self = Self::new(StringMap::A, &[]);


    pub const fn new(name: StringIndex, bounds: &'me [Generic<'me>]) -> Self {
        Self { name, bounds }
    }
}


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GenericKind<'me> {
    Generic(BoundedGeneric<'me>),
    Sym(SymbolId, &'me [Generic<'me>]),
}


impl<'me> SymbolMap<'me> {
    #[inline(always)]
    pub fn pending(&mut self, ns_map: &mut NamespaceMap, parent: Option<NamespaceId>,
                      path: StringIndex, gen_count: usize) -> SymbolId {
        self.syms.push((Err(gen_count), ns_map.push(Namespace::new(path), parent), HashMap::new()))
    }


    pub fn insert_closure_capture(&mut self, closure: ClosureId, name: StringIndex, ty: Type) {
        self.closures[closure].captured_variables.insert((name, ty));
    }


    pub fn traits(&mut self, sym: SymbolId) -> &mut TraitMap<'me> {
        &mut self.syms[sym].2
    }


    pub fn trait_implementation(
        &mut self,
        ty: Type,
        trait_ty: Type,
    ) -> Option<(NamespaceId, std::vec::Vec<(BoundedGeneric<'me>, Type)>)> {
        let Type::Ty(trait_id, _) = trait_ty.instantiate_shallow(self)
        else { return None; };
        if !self.trait_arguments_satisfy_bounds(trait_ty) {
            return None;
        }

        let sym = ty.sym(self).ok()?;
        let impls = self.syms[sym].2.get(&trait_id)?.clone();
        let mut implementation = None;

        for entry in impls {
            let mut bindings = std::vec::Vec::with_capacity(entry.generics.len());
            if !self.match_impl_type(entry.receiver, ty, entry.generics, &mut bindings) {
                continue;
            }
            if !self.match_impl_type(entry.trait_ty, trait_ty, entry.generics, &mut bindings) {
                continue;
            }

            let bounds_ok =
            entry.generics.iter().all(|generic| {
                let Some((_, bound_ty)) = bindings.iter().find(|(value, _)| value.name == generic.name)
                else { return false };

                generic.bounds.iter().all(|bound| {
                    let bound_trait_ty = bound.to_ty(&bindings, self);
                    self.type_implements_trait_generic(*bound_ty, bound_trait_ty)
                })
            });

            if bounds_ok {
                if implementation.is_some() {
                    return None;
                }
                implementation = Some((entry.namespace, bindings));
            }
        }

        implementation
    }


    pub(crate) fn trait_arguments_satisfy_bounds(&mut self, trait_ty: Type) -> bool {
        let Type::Ty(trait_id, _) = trait_ty.instantiate_shallow(self)
        else { return false; };
        let trait_generics = self.sym(trait_id).generics();
        let trait_args_id = trait_ty.gens(self);
        let trait_args = self.get_gens(trait_args_id);
        if trait_generics.len() != trait_args.len() {
            return false;
        }

        for (generic, (_, actual)) in trait_generics.iter().zip(trait_args) {
            for bound in generic.bounds {
                let bound_trait_ty = bound.to_ty(trait_args, self);
                if !self.type_implements_trait_generic(*actual, bound_trait_ty) {
                    return false;
                }
            }
        }

        true
    }

    pub(crate) fn trait_argument_bound_failure(
        &mut self,
        trait_ty: Type,
    ) -> Option<(usize, Type, SymbolId)> {
        let Type::Ty(trait_id, _) = trait_ty.instantiate_shallow(self)
        else { return None };
        let trait_generics = self.sym(trait_id).generics();
        let trait_args_id = trait_ty.gens(self);
        let trait_args = self.get_gens(trait_args_id);
        if trait_generics.len() != trait_args.len() {
            return None;
        }

        for (index, (generic, (_, actual))) in trait_generics.iter().zip(trait_args).enumerate() {
            for bound in generic.bounds {
                let bound_trait_ty = bound.to_ty(trait_args, self);
                if !self.type_implements_trait_generic(*actual, bound_trait_ty) {
                    let Type::Ty(bound_id, _) = bound_trait_ty.instantiate_shallow(self)
                    else { continue };
                    return Some((index, *actual, bound_id));
                }
            }
        }

        None
    }


    pub fn type_implements_trait_generic(&mut self, ty: Type, trait_ty: Type) -> bool {
        let Type::Ty(trait_id, _) = trait_ty.instantiate_shallow(self) 
        else {
            return false;
        };

        if self.trait_implementation(ty, trait_ty).is_some() {
            return true;
        }

        let SymbolKind::Trait(trait_sym) = self.sym(trait_id).kind()
        else {
            return false;
        };

        match trait_sym.synthesis {
            TraitSynthesis::None => false,
            TraitSynthesis::UniversalNoop => true,
        }
    }

    pub(crate) fn match_impl_type(
        &mut self,
        pattern: Generic<'me>,
        actual: Type,
        impl_gens: &[BoundedGeneric<'me>],
        bindings: &mut std::vec::Vec<(BoundedGeneric<'me>, Type)>,
    ) -> bool {
        match pattern.kind {
            GenericKind::Generic(generic) => {
                if !impl_gens.iter().any(|value| value.name == generic.name) {
                    return false;
                }

                match bindings.iter().find(|(value, _)| value.name == generic.name) {
                    Some((_, bound)) => (*bound).eq(self, actual),
                    None => {
                        let generic = impl_gens.iter()
                            .find(|value| value.name == generic.name)
                            .copied()
                            .unwrap();
                        bindings.push((generic, actual));
                        true
                    },
                }
            },

            GenericKind::Sym(sym, gens) => {
                let Type::Ty(actual_sym, actual_gens) = actual.instantiate_shallow(self)
                else { return false };

                let actual_gens: std::vec::Vec<_> = self.get_gens(actual_gens)
                    .iter()
                    .map(|value| value.1)
                    .collect();
                if sym != actual_sym || gens.len() != actual_gens.len() {
                    return false;
                }

                gens.iter().zip(actual_gens).all(|(pattern, actual)| {
                    self.match_impl_type(*pattern, actual, impl_gens, bindings)
                })
            },
        }
    }


    pub fn add_enum(&mut self, ns_map: &mut NamespaceMap,
                    string_map: &mut StringMap,
                    id: SymbolId, range: SourceRange,
                    name: StringIndex, mappings: &'me [(StringIndex, Generic<'me>)],
                    generics: &'me [BoundedGeneric<'me>], decl: Option<DeclId>) {

        let sk = SymbolKind::Container(Container::new(mappings, ContainerKind::Enum));
        let sym = Symbol::new(name, generics, sk);
        self.add_sym(id, sym);

        let ns = self.sym_ns(id);

        let ret = {
            let mut vec = sti::vec::Vec::with_cap_in(self.arena, generics.len());
            for g in generics {
                vec.push(Generic::new(range, GenericKind::Generic(*g)));
            }

            let gens = vec.leak();
            Generic::new(range, GenericKind::Sym(id, gens))
        };

        for (index, i) in mappings.iter().enumerate() {
            let mapping_name = i.0;
            let func_name = string_map.concat(name, mapping_name);

            let is_unit = i.1.sym().map(|x| x == SymbolId::UNIT).unwrap_or(false);

            let args = if is_unit { [].as_slice() }
                       else { &*self.arena.alloc_new([FunctionArgument::new(StringMap::VALUE, i.1)]) };
            let sym = FunctionTy::new(args, ret, FunctionKind::Enum { sym: id, index }, decl, generics);
            let sym = Symbol::new(func_name, generics, SymbolKind::Function(sym));
            let id = self.pending(ns_map, Some(ns), func_name, generics.len());
            self.add_sym(id, sym);

            let ns = ns_map.get_ns_mut(ns);

            ns.add_sym_unchecked(mapping_name, id, Visibility::Private);
        }
    }


    pub fn add_sym(&mut self, id: SymbolId, sym: Symbol<'me>) { 
        let gen_len = self.syms[id].0.unwrap_err();
        assert_eq!(sym.generics.len(), gen_len);

        self.syms[id].0 = Ok(sym)
    }


    /// True when the symbol failed to resolve (a per-failure error
    /// symbol). Kind-based so codegen can dispatch on it.
    /// Pending (two-pass) entries are not error symbols.
    pub fn is_err_sym(&self, sym: SymbolId) -> bool {
        match self.syms[sym].0 {
            Ok(sym) => matches!(sym.kind(), SymbolKind::Error(_)),
            Err(_) => false,
        }
    }


    /// The symbol at `id` when it has been finalized; None while pending.
    pub fn sym_ok(&self, id: SymbolId) -> Option<Symbol<'me>> {
        self.syms[id].0.ok()
    }


    /// Registers a fresh per-failure error symbol carrying `id`. Every
    /// failed resolution gets its own symbolmap entry with kind
    /// `SymbolKind::Error(id)`.
    pub fn error_sym(&mut self, ns_map: &mut NamespaceMap, id: ErrorId) -> SymbolId {
        let pending = self.pending(ns_map, None, StringMap::ERROR, 0);
        self.add_sym(pending, Symbol::new(StringMap::ERROR, &[], SymbolKind::Error(id)));
        pending
    }


    pub fn as_ns(&self, id: SymbolId) -> NamespaceId {
        assert!(matches!(self.sym(id).kind(), SymbolKind::Namespace));
        self.sym_ns(id)
    }


    pub fn sym(&self, id: SymbolId) -> Symbol<'me> { 
        self.syms[id].0.unwrap()
    }


    pub fn cached_fn(&mut self, id: SymbolId) { 
        let SymbolKind::Function(func) = &mut self.syms[id].0.as_mut().unwrap().kind
        else { unreachable!() };

        func.cached = true;
    }


    pub fn sym_gens_size(&mut self, id: SymbolId) -> usize { 
        match self.syms[id].0 {
            Ok(v) => v.generics.len(),
            Err(v) => v,
        }
    }


    pub fn sym_ns(&self, id: SymbolId) -> NamespaceId { 
        self.syms[id].1
    }


    pub fn new_var(&mut self, node: impl Into<NodeId>, name: impl Into<Option<StringIndex>>, range: SourceRange) -> Type {
        self.new_var_ex(node, name, range, VarSub::None)
    }


    pub fn new_var_ex(&mut self, node: impl Into<NodeId>, name: impl Into<Option<StringIndex>>, range: SourceRange, sub: VarSub) -> Type {
        Type::Var(self.vars.push(Var { sub, node: node.into(), name: name.into(), range }))
    }


    pub fn get_gens(&self, g: GenListId) -> &'me [(BoundedGeneric<'me>, Type)] {
        self.gens[g]
    }


    pub fn add_gens(&mut self, generics: &'me [(BoundedGeneric<'me>, Type)]) -> GenListId {
        if generics.is_empty() { return GenListId::EMPTY }
        self.gens.push(generics)
    }


    pub fn get_ty(&mut self, ty: SymbolId, generics: &[Type]) -> Type {
        let sym = self.sym(ty);
        let vec = sti::vec::Vec::from_in(self.arena, sym.generics.iter().copied().zip(generics.iter().copied()));
        let generics = if generics.is_empty() { GenListId::EMPTY }
                       else { self.add_gens(copy_slice_in(self.arena, vec.leak())) };
        Type::Ty(ty, generics)
    }


    pub fn arena(&self) -> &'me Arena {
        self.arena
    }


    pub fn gens(&self) -> &KVec<GenListId, &'me [(BoundedGeneric<'me>, Type)]> {
        &self.gens
    }


    pub fn vars(&self) -> &KVec<VarId, Var> {
        &self.vars
    }


    pub fn vars_mut(&mut self) -> &mut KVec<VarId, Var> {
        &mut self.vars
    }


    pub fn new_closure(&mut self) -> ClosureId {
        self.closures.push(Closure { captured_variables: HashSet::new() })
    }


    pub fn closure(&self, id: ClosureId) -> &Closure {
        &self.closures[id]
    }



}


impl<'me> Generic<'me> {
    pub fn new(range: SourceRange, kind: GenericKind<'me>) -> Self { Self { range, kind } }

    pub fn sym(self) -> Option<SymbolId> {
        match self.kind {
            GenericKind::Generic(_) => None,
            GenericKind::Sym(v, _) => Some(v),
        }
    }

    pub fn gens(self) -> Option<&'me [Generic<'me>]> {
        match self.kind {
            GenericKind::Generic(_) => None,
            GenericKind::Sym(_, gens) => Some(gens),
        }
    }

    pub fn collect_generics(self, out: &mut std::vec::Vec<BoundedGeneric<'me>>) {
        match self.kind {
            GenericKind::Generic(g) => {
                if !out.iter().any(|existing| existing.name == g.name) {
                    out.push(g);
                }
            },
            GenericKind::Sym(_, args) => {
                for arg in args {
                    arg.collect_generics(out);
                }
            }
        }
    }
    

    pub fn to_ty(self, gens: &[(BoundedGeneric<'me>, Type)], map: &mut SymbolMap) -> Type {
        match self.kind {
            GenericKind::Generic(v) => {
                gens.iter()
                    .find(|x| x.0.name() == v.name())
                    .copied()
                    .map(|x| x.1)
                    .expect(&format!("COMPILER ERROR: a generic name can't be missing as \
                            if it was the case it would've been a custom type. {v:?}. {gens:?}"))
            },


            GenericKind::Sym(symbol, generics) => {
                let pool = map.arena();
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());
                    for g in generics {
                        vec.push(g.to_ty(gens, map));
                    }
                    vec
                };

                //dbg!(symbol, &generics);
                
                map.get_ty(symbol, &generics)
            },
        }
    }

    pub fn rec_replace(self, alloc: &'me Arena, gen_name: StringIndex, repl: Generic<'me>) -> Generic<'me> {
        match self.kind {
            GenericKind::Generic(v) => {
                if v.name() == gen_name { repl }
                else { self }
            },


            GenericKind::Sym(symbol, generics) => {
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(alloc, generics.len());
                    for g in generics {
                        vec.push(g.rec_replace(alloc, gen_name, repl));
                    }
                    vec
                };
                
                Generic::new(self.range, GenericKind::Sym(symbol, generics.leak()))
            },
        }
    }
}


impl<'me> SymbolMap<'me> {
    pub fn new(
        arena: &'me Arena, ns_map: &mut NamespaceMap,
        string_map: &mut StringMap
    ) -> Self {

        let mut slf = Self { syms: KVec::new(), vars: KVec::new(), arena, gens: KVec::new(), closures: KVec::new(), };

        assert_eq!(slf.gens.push(&[]), GenListId::EMPTY);

        macro_rules! init {
            ($name: ident) => {
                let pending = slf.pending(ns_map, None, StringMap::$name, 0);
                assert_eq!(pending, SymbolId::$name);
                let kind = SymbolKind::Container(Container::new(&[], ContainerKind::Struct));
                slf.add_sym(pending, Symbol::new(StringMap::$name, &[], kind));
            };
        }

        init!(UNIT);
        init!(I64);
        init!(F64);

        // bool
        {
            let pending = slf.pending(ns_map, None, StringMap::BOOL, 0);
            assert_eq!(pending, SymbolId::BOOL);
            let fields = [
                (StringMap::FALSE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
                (StringMap::TRUE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            slf.add_enum(ns_map, string_map, pending, SourceRange::ZERO,
                         StringMap::BOOL, slf.arena.alloc_new(fields), &[], None);
        }

        init!(NEVER);

        // ptr<T> — opaque raw pointer
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR, 1);
            assert_eq!(pending, SymbolId::PTR);
            slf.add_sym(pending, Symbol::new(StringMap::PTR, arena.alloc_new([t]), SymbolKind::Opaque));
        }

        // range
        {
            let pending = slf.pending(ns_map, None, StringMap::RANGE, 0);
            assert_eq!(pending, SymbolId::RANGE);
            let fields = [
                (StringMap::MIN, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))),
                (StringMap::MAX, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))),
            ];

            let cont = Container::new(arena.alloc_new(fields), ContainerKind::Struct);
            let kind = SymbolKind::Container(cont);

            slf.add_sym(pending, Symbol::new(StringMap::RANGE, &[], kind));
        }


        // option 
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::OPTION, 1);
            assert_eq!(pending, SymbolId::OPTION);
            let fields = [
                (StringMap::SOME, Generic::new(SourceRange::ZERO, GenericKind::Generic(t))),
                (StringMap::NONE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            let gens = slf.arena.alloc_new([t]);

            slf.add_enum(ns_map, string_map, pending, SourceRange::ZERO,
                         StringMap::OPTION, slf.arena.alloc_new(fields), gens, None);
        }


        // result 
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let a = BoundedGeneric::new(StringMap::A, &[]);

            let pending = slf.pending(ns_map, None, StringMap::RESULT, 2);
            assert_eq!(pending, SymbolId::RESULT);
            let fields = [
                (StringMap::OK , Generic::new(SourceRange::ZERO, GenericKind::Generic(t))),
                (StringMap::ERR, Generic::new(SourceRange::ZERO, GenericKind::Generic(a))),
            ];

            let gens = slf.arena.alloc_new([t, a]);

            slf.add_enum(ns_map, string_map, pending, SourceRange::ZERO,
                         StringMap::RESULT, slf.arena.alloc_new(fields), gens, None);

        }


        // str is a nominal wrapper around the byte collection. The
        // compiler still seeds this type before loading the standard library
        // so literals and runtime boundaries can use it during bootstrap.
        {
            let pending = slf.pending(ns_map, None, StringMap::STR, 0);
            assert_eq!(pending, SymbolId::STR);

            let byte = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::BYTE, &[]));
            let bytes = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST, arena.alloc_new([byte])));
            let fields = arena.alloc_new([(StringMap::VALUE, bytes)]);
            let kind = SymbolKind::Container(Container::new(fields, ContainerKind::Struct));
            slf.add_sym(pending, Symbol::new(StringMap::STR, &[], kind));
        }


        // list
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);

            let pending = slf.pending(ns_map, None, StringMap::LIST, 1);
            assert_eq!(pending, SymbolId::LIST);
            slf.add_sym(pending, Symbol::new(StringMap::LIST, arena.alloc_new([t]), SymbolKind::Opaque));
        }



        // $type_id
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);

            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_TYPE_ID, 1);
            assert_eq!(pending, SymbolId::BUILTIN_TYPE_ID);

            let sym = Symbol::new(
                StringMap::BUILTIN_TYPE_ID,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        &[],
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[])),
                        FunctionKind::TypeId,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $size_of
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_SIZE_OF, 1);
            assert_eq!(pending, SymbolId::BUILTIN_SIZE_OF);

            let sym = Symbol::new(
                StringMap::BUILTIN_SIZE_OF,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        &[],
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[])),
                        FunctionKind::SizeOf,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $size_of
        {
            let pending = slf.pending(ns_map, None, StringMap::EQ_TRAIT, 0);
            assert_eq!(pending, SymbolId::EQ_TRAIT);

            let sym = Symbol::new(
                StringMap::EQ_TRAIT,
                &[],
                SymbolKind::Trait(Trait {
                    funcs: arena.alloc_new([
                       (StringMap::EQ_FUNC, FunctionTy::new(
                            arena.alloc_new([
                                FunctionArgument::new(
                                    StringMap::SELF,
                                    Generic::new(SourceRange::ZERO, GenericKind::Generic(BoundedGeneric::new(StringMap::SELF_TY, &[])))
                                ),
                                FunctionArgument::new(
                                    StringMap::VALUE,
                                    Generic::new(SourceRange::ZERO, GenericKind::Generic(BoundedGeneric::new(StringMap::SELF_TY, &[])))
                                )
                            ]),

                            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::BOOL, &[])),

                            FunctionKind::Trait,
                            None,
                            &[],
                        )
                    )]),
                    synthesis: TraitSynthesis::None,
                })
            );

            slf.add_sym(pending, sym);
        }



        // Rc
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::RC, 1);
            assert_eq!(pending, SymbolId::RC);
            slf.add_sym(pending, Symbol::new(StringMap::RC, arena.alloc_new([t]), SymbolKind::Opaque));
        }


        // $rc
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_RC, 1);
            assert_eq!(pending, SymbolId::BUILTIN_RC);

            let args = [
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Generic(t))
                )
            ];

            let ret_gens = arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))]);

            let sym = Symbol::new(
                StringMap::BUILTIN_RC,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::RC, ret_gens)),
                        FunctionKind::Rc,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $rc_get
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::RC_GET, 1);
            assert_eq!(pending, SymbolId::RC_GET);

            let args = [
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::RC, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])))
                )
            ];

            let sym = Symbol::new(
                StringMap::RC_GET,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Generic(t)),
                        FunctionKind::RcGet,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $rc_set
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::RC_SET, 1);
            assert_eq!(pending, SymbolId::RC_SET);

            let rc_ty_generic = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::RC, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, rc_ty_generic),
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Generic(t))
                )
            ];

            let sym = Symbol::new(
                StringMap::RC_SET,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),
                        FunctionKind::RcSet,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }

        // $ptr_alloc<T>(count: int): ptr<T>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_ALLOC, 1);
            assert_eq!(pending, SymbolId::PTR_ALLOC);

            let args = [
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))
                )
            ];

            let ret_gens = arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))]);

            let sym = Symbol::new(
                StringMap::PTR_ALLOC,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, ret_gens)),
                        FunctionKind::PtrAlloc,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_free<T>(p: ptr<T>, count: int)
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_FREE, 1);
            assert_eq!(pending, SymbolId::PTR_FREE);

            let ptr_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_ty),
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))
                )
            ];

            let sym = Symbol::new(
                StringMap::PTR_FREE,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),
                        FunctionKind::PtrFree,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_read<T>(p: ptr<T>): T
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_READ, 1);
            assert_eq!(pending, SymbolId::PTR_READ);

            let args = [
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])))
                )
            ];

            let sym = Symbol::new(
                StringMap::PTR_READ,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Generic(t)),
                        FunctionKind::PtrRead,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_write<T>(p: ptr<T>, value: T)
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_WRITE, 1);
            assert_eq!(pending, SymbolId::PTR_WRITE);

            let ptr_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_ty),
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Generic(t))
                )
            ];

            let sym = Symbol::new(
                StringMap::PTR_WRITE,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),
                        FunctionKind::PtrWrite,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_write_uninit<T>(p: ptr<T>, value: T)
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_WRITE_UNINIT, 1);
            assert_eq!(pending, SymbolId::PTR_WRITE_UNINIT);

            let ptr_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_ty),
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Generic(t))
                )
            ];

            let sym = Symbol::new(
                StringMap::PTR_WRITE_UNINIT,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),
                        FunctionKind::PtrWriteUninit,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_null<T>(): ptr<T>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_NULL, 1);
            assert_eq!(pending, SymbolId::PTR_NULL);

            let ret_gens = arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))]);

            let sym = Symbol::new(
                StringMap::PTR_NULL,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        &[],
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, ret_gens)),
                        FunctionKind::PtrNull,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_offset<T>(p: ptr<T>, off: int): ptr<T>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_OFFSET, 1);
            assert_eq!(pending, SymbolId::PTR_OFFSET);

            let ptr_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let ret_gens = arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))]);

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_ty),
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))
                )
            ];

            let sym = Symbol::new(
                StringMap::PTR_OFFSET,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, ret_gens)),
                        FunctionKind::PtrOffset,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $ptr_cast<T, U>(p: ptr<T>): ptr<U>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let u = BoundedGeneric::new(StringMap::A, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_CAST, 2);
            assert_eq!(pending, SymbolId::PTR_CAST);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let u_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(u));

            let ptr_t = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([t_gen])));

            let ptr_u = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([u_gen])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_t),
            ];

            let sym = Symbol::new(
                StringMap::PTR_CAST,
                arena.alloc_new([t, u]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        ptr_u,
                        FunctionKind::PtrCast,
                        None,
                        arena.alloc_new([t, u]),
                )));

            slf.add_sym(pending, sym);
        }

        // Destroy
        {
            let pending = slf.pending(ns_map, None, StringMap::DESTROY_TRAIT, 0);
            assert_eq!(pending, SymbolId::DESTROY_TRAIT);

            let sym = Symbol::new(
                StringMap::DESTROY_TRAIT,
                &[],
                SymbolKind::Trait(Trait {
                    funcs: arena.alloc_new([
                       (StringMap::DESTROY_FUNC, FunctionTy::new(
                            arena.alloc_new([
                                FunctionArgument::new(
                                    StringMap::SELF,
                                    Generic::new(SourceRange::ZERO, GenericKind::Generic(BoundedGeneric::new(StringMap::SELF_TY, &[])))
                                )
                            ]),

                            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),

                            FunctionKind::Trait,
                            None,
                            &[],
                        )
                    )]),
                    synthesis: TraitSynthesis::UniversalNoop,
                })
            );

            slf.add_sym(pending, sym);
        }


        // $ptr_drop<T>(p: ptr<T>)
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::PTR_DROP, 1);
            assert_eq!(pending, SymbolId::PTR_DROP);

            let ptr_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, arena.alloc_new([Generic::new(SourceRange::ZERO, GenericKind::Generic(t))])));

            let args = [
                FunctionArgument::new(StringMap::VALUE, ptr_ty),
            ];

            let sym = Symbol::new(
                StringMap::PTR_DROP,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                        arena.alloc_new(args),
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[])),
                        FunctionKind::PtrDrop,
                        None,
                        arena.alloc_new([t]),
                )));

            slf.add_sym(pending, sym);
        }


        // $list_concat<T>(left: [T], right: [T]): [T]
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::LIST_CONCAT, 1);
            assert_eq!(pending, SymbolId::LIST_CONCAT);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let list_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST, arena.alloc_new([t_gen])));
            let args = [
                FunctionArgument::new(StringMap::VALUE, list_ty),
                FunctionArgument::new(StringMap::VALUE, list_ty),
            ];

            let sym = Symbol::new(
                StringMap::LIST_CONCAT,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                    arena.alloc_new(args),
                    list_ty,
                    FunctionKind::ListConcat,
                    None,
                    arena.alloc_new([t]),
                )),
            );

            slf.add_sym(pending, sym);
        }


        // Internal tuple type for $list_slice<T>'s Option<([T], [T])> return value.
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let a = BoundedGeneric::new(StringMap::A, &[]);
            let pending = slf.pending(ns_map, None, StringMap::INVALID_IDENT, 2);
            assert_eq!(pending, SymbolId::LIST_SLICE_PAIR);
            let fields = arena.alloc_new([
                (string_map.num(0), Generic::new(SourceRange::ZERO, GenericKind::Generic(t))),
                (string_map.num(1), Generic::new(SourceRange::ZERO, GenericKind::Generic(a))),
            ]);
            let sym = Symbol::new(
                StringMap::TUPLE,
                arena.alloc_new([t, a]),
                SymbolKind::Container(Container::new(fields, ContainerKind::Tuple)),
            );
            slf.add_sym(pending, sym);
        }


        // $list_slice<T>(list: [T], idx: int): Option<([T], [T])>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::LIST_SLICE, 1);
            assert_eq!(pending, SymbolId::LIST_SLICE);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let list_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST, arena.alloc_new([t_gen])));
            let pair_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST_SLICE_PAIR, arena.alloc_new([list_ty, list_ty])));
            let ret = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::OPTION, arena.alloc_new([pair_ty])));
            let args = [
                FunctionArgument::new(StringMap::VALUE, list_ty),
                FunctionArgument::new(StringMap::VALUE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))),
            ];

            let sym = Symbol::new(
                StringMap::LIST_SLICE,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                    arena.alloc_new(args),
                    ret,
                    FunctionKind::ListSlice,
                    None,
                    arena.alloc_new([t]),
                )),
            );

            slf.add_sym(pending, sym);
        }


        
        // $list_len<T>(list: [T]): int
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::LIST_LEN, 1);
            assert_eq!(pending, SymbolId::LIST_LEN);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let list_ty = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST, arena.alloc_new([t_gen])));
            let args = [
                FunctionArgument::new(StringMap::VALUE, list_ty),
            ];
            let sym = Symbol::new(
                StringMap::LIST_LEN,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                    arena.alloc_new(args),
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[])),
                    FunctionKind::ListLen,
                    None,
                    arena.alloc_new([t]),
                )),
            );
            slf.add_sym(pending, sym);
        }


        init!(BYTE);
        // ListIter<T> is an opaque inline stack value.
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::LIST_ITER, 1);
            assert_eq!(pending, SymbolId::LIST_ITER);
            slf.add_sym(
                pending,
                Symbol::new(StringMap::LIST_ITER, arena.alloc_new([t]), SymbolKind::Opaque),
            );
        }

        // $list_iter<T>(list: [T]): ListIter<T>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_LIST_ITER, 1);
            assert_eq!(pending, SymbolId::BUILTIN_LIST_ITER);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let list_ty =
            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST, arena.alloc_new([t_gen])));
            let iter_ty =
            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST_ITER, arena.alloc_new([t_gen])));
            let args = [FunctionArgument::new(StringMap::VALUE, list_ty)];

            let sym =
            Symbol::new(
                StringMap::BUILTIN_LIST_ITER,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                    arena.alloc_new(args),
                    iter_ty,
                    FunctionKind::ListIter,
                    None,
                    arena.alloc_new([t]),
                )),
            );
            slf.add_sym(pending, sym);
        }

        // $list_iter_next<T>(&iter: ListIter<T>): Option<T>
        {
            let t = BoundedGeneric::new(StringMap::T, &[]);
            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_LIST_ITER_NEXT, 1);
            assert_eq!(pending, SymbolId::BUILTIN_LIST_ITER_NEXT);

            let t_gen = Generic::new(SourceRange::ZERO, GenericKind::Generic(t));
            let iter_ty =
            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::LIST_ITER, arena.alloc_new([t_gen])));
            let ret_ty =
            Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::OPTION, arena.alloc_new([t_gen])));
            let args = [FunctionArgument::new_inout(StringMap::VALUE, iter_ty, true)];

            let sym =
            Symbol::new(
                StringMap::BUILTIN_LIST_ITER_NEXT,
                arena.alloc_new([t]),
                SymbolKind::Function(FunctionTy::new(
                    arena.alloc_new(args),
                    ret_ty,
                    FunctionKind::ListIterNext,
                    None,
                    arena.alloc_new([t]),
                )),
            );
            slf.add_sym(pending, sym);
        }
        // $float_sqrt
        {
            let pending = slf.pending(ns_map, None, StringMap::BUILTIN_FLOAT_SQRT, 0);
            assert_eq!(pending, SymbolId::BUILTIN_FLOAT_SQRT);

            let args = arena.alloc_new([
                FunctionArgument::new(
                    StringMap::VALUE,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::F64, &[])),
                )
            ]);

            let sym = Symbol::new(
                StringMap::BUILTIN_FLOAT_SQRT,
                &[],
                SymbolKind::Function(FunctionTy::new(
                    args,
                    Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::F64, &[])),
                    FunctionKind::FloatSqrt,
                    None,
                    &[],
                )),
            );

            slf.add_sym(pending, sym);
        }

        slf
    }
}


impl VarId {
    pub fn occurs_in(self, map: &SymbolMap, ty: Type) -> bool {
        match ty {
            Type::Ty(_, gens) => map.gens[gens].iter().any(|x| self.occurs_in(map, x.1)),
            Type::Var(v) => {
                if self == v { return true }

                let sub = map.vars[v].sub;
                match sub {
                    VarSub::Concrete(ty) => self.occurs_in(map, ty),
                    _ => false
                }
            },
        }
    }
}


impl Var {
    pub fn is_concrete(&self, map: &mut SymbolMap) -> bool {
        let VarSub::Concrete(ty) = self.sub
        else { return false };

        matches!(ty.instantiate_shallow(map), Type::Ty(..))
    }


    pub fn is_root(&self, map: &mut SymbolMap) -> bool {
        let VarSub::Concrete(ty) = self.sub
        else { return true };

        assert!(matches!(ty.instantiate_shallow(map), Type::Var(_)));
        return false
    }


    pub fn set_sub(&mut self, sub: VarSub) { 
        self.sub = sub;
    }
}





impl GenListId {
    pub const EMPTY: Self = Self(0);
}


impl PartialEq for Generic<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}
