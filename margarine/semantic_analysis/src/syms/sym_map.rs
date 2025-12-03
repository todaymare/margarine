use std::collections::HashSet;

use common::{copy_slice_in, source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use errors::ErrorId;
use parser::nodes::{decl::DeclId, NodeId};
use sti::{arena::Arena, define_key, ext::FromIn, vec::KVec};

use crate::{errors::Error, namespace::{Namespace, NamespaceId, NamespaceMap}, syms::{containers::{Container, ContainerKind}, func::{FunctionArgument, FunctionKind, FunctionTy}, SymbolKind}};

use super::{ty::Sym, Symbol};

define_key!(pub SymbolId(pub u32));
define_key!(pub GenListId(pub u32));
define_key!(pub VarId(pub u32));
define_key!(pub ClosureId(pub u32));

pub struct SymbolMap<'me> {
    syms : KVec<SymbolId, (Result<Symbol<'me>, usize>, NamespaceId)>,
    gens : KVec<GenListId, &'me [(StringIndex, Sym)]>,
    vars : KVec<VarId, Var>,
    closures: KVec<ClosureId, Closure>,
    arena: &'me Arena,
}


#[derive(Debug)]
pub struct Closure {
    pub captured_variables: HashSet<StringIndex>,
}


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Var {
    sub: VarSub,
    node: NodeId,
    range: SourceRange,
}


#[derive(Debug, Clone, Copy)]
pub enum VarSub {
    Concrete(Sym),
    None,
}


#[derive(Clone, Copy, Debug, ImmutableData)]
pub struct Generic<'me> {
    range: SourceRange,
    kind : GenericKind<'me>,
    err  : Option<ErrorId>,
}


#[derive(Clone, Copy, Debug)]
pub enum GenericKind<'me> {
    Generic(StringIndex),
    Sym(SymbolId, &'me [Generic<'me>]),
}


impl<'me> SymbolMap<'me> {
    #[inline(always)]
    pub fn pending(&mut self, ns_map: &mut NamespaceMap,
                   path: StringIndex, gen_count: usize) -> SymbolId {

        self.syms.push((Err(gen_count), ns_map.push(Namespace::new(path))))
    }


    pub fn insert_closure_capture(&mut self, closure: ClosureId, name: StringIndex) {
        self.closures[closure].captured_variables.insert(name);

    }


    pub fn add_enum(&mut self, id: SymbolId, ns_map: &mut NamespaceMap,
                    string_map: &mut StringMap, range: SourceRange,
                    name: StringIndex, mappings: &'me [(StringIndex, Generic<'me>)],
                    generics: &'me [StringIndex], decl: Option<DeclId>) {

        let sk = SymbolKind::Container(Container::new(mappings, ContainerKind::Enum));
        let sym = Symbol::new(name, generics, sk);
        self.add_sym(id, sym);

        let ns = self.sym_ns(id);

        let ret = {
            let mut vec = sti::vec::Vec::with_cap_in(self.arena, generics.len());
            for g in generics {
                vec.push(Generic::new(range, GenericKind::Generic(*g), None));
            }

            let gens = vec.leak();
            Generic::new(range, GenericKind::Sym(id, gens), None)
        };

        for (index, i) in mappings.iter().enumerate() {
            let mapping_name = i.0;
            let func_name = string_map.concat(name, mapping_name);

            let is_unit = i.1.sym().map(|x| x == SymbolId::UNIT).unwrap_or(false);

            let args = if is_unit { [].as_slice() }
                       else { &*self.arena.alloc_new([FunctionArgument::new(StringMap::VALUE, i.1)]) };
            let sym = FunctionTy::new(args, ret, FunctionKind::Enum { sym: id, index }, decl);
            let sym = Symbol::new(func_name, generics, SymbolKind::Function(sym));
            let id = self.pending(ns_map, func_name, generics.len());
            self.add_sym(id, sym);

            let ns = ns_map.get_ns_mut(ns);

            ns.add_sym(range, mapping_name, id).unwrap();
        }
    }


    pub fn add_sym(&mut self, id: SymbolId, sym: Symbol<'me>) { 
        let gen_len = self.syms[id].0.unwrap_err();
        assert_eq!(sym.generics.len(), gen_len);

        self.syms[id].0 = Ok(sym)
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


    pub fn new_var(&mut self, node: impl Into<NodeId>, range: SourceRange) -> Sym {
        self.new_var_ex(node, range, VarSub::None)
    }


    pub fn new_var_ex(&mut self, node: impl Into<NodeId>, range: SourceRange, sub: VarSub) -> Sym {
        Sym::Var(self.vars.push(Var { sub, node: node.into(), range }))
    }


    pub fn get_gens(&mut self, g: GenListId) -> &'me [(StringIndex, Sym)] {
        self.gens[g]
    }


    pub fn add_gens(&mut self, generics: &'me [(StringIndex, Sym)]) -> GenListId {
        if generics.is_empty() { return GenListId::EMPTY }
        self.gens.push(generics)
    }


    pub fn get_ty(&mut self, ty: SymbolId, generics: &[Sym]) -> Sym {
        let sym = self.sym(ty);
        let vec = sti::vec::Vec::from_in(self.arena, sym.generics.iter().copied().zip(generics.iter().copied()));
        let generics = if generics.is_empty() { GenListId::EMPTY }
                       else { self.add_gens(copy_slice_in(self.arena, vec.leak())) };
        Sym::Ty(ty, generics)
    }


    pub fn arena(&self) -> &'me Arena {
        self.arena
    }


    pub fn gens(&self) -> &KVec<GenListId, &'me [(StringIndex, Sym)]> {
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
    pub fn new(range: SourceRange, kind: GenericKind<'me>, err: Option<ErrorId>) -> Self { Self { range, kind, err } }

    pub fn sym(self) -> Option<SymbolId> {
        match self.kind {
            GenericKind::Generic(_) => None,
            GenericKind::Sym(v, _) => Some(v),
        }
    }
    

    pub fn to_ty(self, gens: &[(StringIndex, Sym)], map: &mut SymbolMap) -> Result<Sym, Error> {
        match self.kind {
            GenericKind::Generic(v) => {
                Ok(gens.iter()
                    .find(|x| x.0 == v)
                    .copied()
                    .map(|x| x.1)
                    .expect("COMPILER ERROR: a generic name can't be missing as \
                            if it was the case it would've been a custom type"))
            },


            GenericKind::Sym(symbol, generics) => {
                let pool = map.arena();
                let generics = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());
                    for g in generics {
                        vec.push(g.to_ty(gens, map)?);
                    }
                    vec
                };
                
                Ok(map.get_ty(symbol, &generics))
            },
        }
    }
}


impl<'me> SymbolMap<'me> {
    pub fn new(arena: &'me Arena, ns_map: &mut NamespaceMap, string_map: &mut StringMap) -> Self {
        let mut slf = Self { syms: KVec::new(), vars: KVec::new(), arena, gens: KVec::new(), closures: KVec::new(), };
        macro_rules! init {
            ($name: ident) => {
                let pending = slf.pending(ns_map, StringMap::$name, 0);
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
            let pending = slf.pending(ns_map, StringMap::BOOL, 0);
            assert_eq!(pending, SymbolId::BOOL);
            let fields = [
                (StringMap::TRUE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]), None)),
                (StringMap::FALSE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]), None)),
            ];

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO,
                         StringMap::BOOL, slf.arena.alloc_new(fields), &[], None);
        }

        init!(ERR);
        init!(NEVER);

        // ptr 
        {
            let pending = slf.pending(ns_map, StringMap::PTR, 1);
            assert_eq!(pending, SymbolId::PTR);
            let fields = [
                (StringMap::COUNT, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]), None)),
                (StringMap::VALUE, Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T), None)),
            ];

            let cont = Container::new(arena.alloc_new(fields), ContainerKind::Struct);
            let kind = SymbolKind::Container(cont);

            slf.add_sym(pending, Symbol::new(StringMap::PTR, &[StringMap::T], kind));
        }

        // range
        {
            let pending = slf.pending(ns_map, StringMap::RANGE, 0);
            assert_eq!(pending, SymbolId::RANGE);
            let fields = [
                (StringMap::MIN, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]), None)),
                (StringMap::MAX, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]), None)),
            ];

            let cont = Container::new(arena.alloc_new(fields), ContainerKind::Struct);
            let kind = SymbolKind::Container(cont);

            slf.add_sym(pending, Symbol::new(StringMap::RANGE, &[], kind));
        }


        // option 
        {
            let pending = slf.pending(ns_map, StringMap::OPTION, 1);
            assert_eq!(pending, SymbolId::OPTION);
            let fields = [
                (StringMap::SOME, Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T), None)),
                (StringMap::NONE, Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]), None)),
            ];

            let gens = slf.arena.alloc_new([StringMap::T]);

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO, 
                         StringMap::OPTION, slf.arena.alloc_new(fields), gens, None);
        }


        // result 
        {
            let pending = slf.pending(ns_map, StringMap::RESULT, 2);
            assert_eq!(pending, SymbolId::RESULT);
            let fields = [
                (StringMap::OK , Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T), None)),
                (StringMap::ERR, Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::A), None)),
            ];

            let gens = slf.arena.alloc_new([StringMap::T, StringMap::A]);

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO, 
                         StringMap::RESULT, slf.arena.alloc_new(fields), gens, None);

        }


        // str 
        {
            let pending = slf.pending(ns_map, StringMap::STR, 0);
            assert_eq!(pending, SymbolId::STR);

            let sym = Symbol::new(StringMap::STR, &[], SymbolKind::Opaque);
            slf.add_sym(pending, sym);
        }


        // type_id
        {
            let pending = slf.pending(ns_map, StringMap::TYPE_ID, 1);
            assert_eq!(pending, SymbolId::TYPE_ID);

            let sym = Symbol::new(
                StringMap::TYPE_ID,
                &[StringMap::T],
                SymbolKind::Function(FunctionTy::new(
                        &[],
                        Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]), None),
                        FunctionKind::TypeId,
                        None,
                )));

            slf.add_sym(pending, sym);
        }

        // list
        {
            let pending = slf.pending(ns_map, StringMap::LIST, 1);
            assert_eq!(pending, SymbolId::LIST);
            slf.add_sym(pending, Symbol::new(StringMap::LIST, &[StringMap::T], SymbolKind::Opaque));
        }



        assert_eq!(slf.gens.push(&[]), GenListId::EMPTY);


        slf
    }
}


impl VarId {
    pub fn occurs_in(self, map: &SymbolMap, ty: Sym) -> bool {
        match ty {
            Sym::Ty(_, gens) => map.gens[gens].iter().any(|x| self.occurs_in(map, x.1)),
            Sym::Var(v) => {
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
    pub fn set_sub(&mut self, sub: VarSub) { 
        self.sub = sub;
    }
}


impl SymbolId {
    pub const UNIT   : Self = Self(0);
    pub const I64    : Self = Self(1);
    pub const F64    : Self = Self(2);
    pub const BOOL   : Self = Self(3); // +2 for variants
    pub const ERR    : Self = Self(6);
    pub const NEVER  : Self = Self(7);
    pub const PTR    : Self = Self(8);
    pub const RANGE  : Self = Self(9);
    pub const OPTION : Self = Self(10); // +2 for variants
    pub const RESULT : Self = Self(13); // +2 for variants
    pub const STR    : Self = Self(16);
    pub const TYPE_ID: Self = Self(17);
    pub const LIST   : Self = Self(18);


    pub fn supports_arith(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }


    pub fn supports_bw(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }


    pub fn supports_ord(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }

    pub fn supports_eq(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::BOOL
            | Self::UNIT
            | Self::ERR
        )
    }


    pub fn is_num(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }


    pub fn is_int(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }

    pub fn is_sint(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }


    pub fn is_float(self) -> bool {
        matches!(self,
            | Self::F64
            | Self::ERR
        )
    }
}


impl GenListId {
    pub const EMPTY: Self = Self(0);
}


impl<'me> GenericKind<'me> {
    pub const ERROR : Self = Self::Sym(SymbolId::ERR, &[]);
}
