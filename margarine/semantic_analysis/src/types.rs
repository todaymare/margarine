pub mod containers;
pub mod ty;
pub mod func;

use common::{copy_slice_in, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use sti::{arena::Arena, define_key, keyed::KVec, traits::FromIn};

use crate::{errors::Error, namespace::{Namespace, NamespaceId, NamespaceMap}, types::func::{FunctionArgument, FunctionKind}};

use self::{containers::{Container, ContainerKind}, func::FunctionTy, ty::Type};

define_key!(u32, pub SymbolId);
define_key!(u32, pub GenListId);
define_key!(u32, pub VarId);

#[derive(Debug)]
pub struct SymbolMap<'me> {
    syms : KVec<SymbolId, (Option<Symbol<'me>>, NamespaceId)>,
    tys  : KVec<GenListId, &'me [(StringIndex, Type)]>,
    vars : KVec<VarId, Var>,
    arena: &'me Arena,
    
}


#[derive(Debug, Clone, Copy)]
pub struct Symbol<'me> {
    pub name    : StringIndex,
    pub generics: &'me [StringIndex],
    pub kind    : SymbolKind<'me>,
}


#[derive(Debug, Clone, Copy)]
pub enum SymbolKind<'me> {
    Function(FunctionTy<'me>),
    Container(Container<'me>),
}


#[derive(Debug)]
pub struct Var {
    sub: Option<Type>,
    range: SourceRange,
}


#[derive(Clone, Copy, Debug)]
pub struct Generic<'me> {
    pub range: SourceRange,
    pub kind : GenericKind<'me>,
}


#[derive(Clone, Copy, Debug)]
pub enum GenericKind<'me> {
    Generic(StringIndex),
    Sym(SymbolId, &'me [Generic<'me>]),
}


impl<'me> SymbolMap<'me> {
    pub fn new(arena: &'me Arena, ns_map: &mut NamespaceMap, string_map: &mut StringMap) -> Self {
        let mut slf = Self { syms: KVec::new(), vars: KVec::new(), arena, tys: KVec::new() };
        macro_rules! init {
            ($name: ident) => {
                let pending = slf.pending(ns_map, StringMap::$name);
                assert_eq!(pending, SymbolId::$name);
                let kind = SymbolKind::Container(Container::new(&[], ContainerKind::Struct));
                slf.add_sym(pending, Symbol::new(StringMap::$name, &[], kind));
            };
        }

        init!(UNIT);
        init!(I8);
        init!(I16);
        init!(I32);
        init!(I64);
        init!(U8);
        init!(U16);
        init!(U32);
        init!(U64);
        init!(F32);
        init!(F64);

        // bool
        {
            let pending = slf.pending(ns_map, StringMap::BOOL);
            assert_eq!(pending, SymbolId::BOOL);
            let fields = [
                (StringMap::TRUE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
                (StringMap::FALSE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO,
                         StringMap::BOOL, slf.arena.alloc_new(fields), &[]);
        }

        init!(ERROR);
        init!(NEVER);

        // rc 
        {
            let pending = slf.pending(ns_map, StringMap::PTR);
            assert_eq!(pending, SymbolId::PTR);
            let fields = [
                (StringMap::COUNT.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::U64, &[]))),
                (StringMap::VALUE.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
            ];

            let cont = Container::new(arena.alloc_new(fields), ContainerKind::Struct);
            let kind = SymbolKind::Container(cont);

            slf.add_sym(pending, Symbol::new(StringMap::PTR, &[StringMap::T], kind));
        }

        // range
        {
            let pending = slf.pending(ns_map, StringMap::RANGE);
            assert_eq!(pending, SymbolId::RANGE);
            let fields = [
                (StringMap::LOW.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))),
                (StringMap::HIGH.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::I64))),
            ];

            let cont = Container::new(arena.alloc_new(fields), ContainerKind::Struct);
            let kind = SymbolKind::Container(cont);

            slf.add_sym(pending, Symbol::new(StringMap::RANGE, &[], kind));
        }


        // option 
        {
            let pending = slf.pending(ns_map, StringMap::OPTION);
            assert_eq!(pending, SymbolId::OPTION);
            let fields = [
                (StringMap::SOME.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
                (StringMap::NONE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            let gens = slf.arena.alloc_new([StringMap::T]);

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO, 
                         StringMap::OPTION, slf.arena.alloc_new(fields), gens);
        }


        // result 
        {
            let pending = slf.pending(ns_map, StringMap::RESULT);
            assert_eq!(pending, SymbolId::RESULT);
            let fields = [
                (StringMap::OK.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
                (StringMap::ERROR.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::A))),
            ];

            let gens = slf.arena.alloc_new([StringMap::T, StringMap::A]);

            slf.add_enum(pending, ns_map, string_map, SourceRange::ZERO, 
                         StringMap::RESULT, slf.arena.alloc_new(fields), gens);

        }


        // str 
        {
            let pending = slf.pending(ns_map, StringMap::STR);
            assert_eq!(pending, SymbolId::STR);
            let ptr = Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::U8, &[]));
            let ptr = slf.arena.alloc_new([ptr]); 

            let fields = [
                (StringMap::PTR.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::PTR, ptr))),
                (StringMap::COUNT.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::U32, &[]))),
            ];

            let fields = slf.arena.alloc_new(fields);

            let sym = Symbol::new(StringMap::STR, &[], SymbolKind::Container(Container::new(fields, ContainerKind::Struct)));
            slf.add_sym(pending, sym);
        }

        assert_eq!(slf.tys.push(&[]), GenListId::EMPTY);


        slf
    }

    #[inline(always)]
    pub fn pending(&mut self, ns_map: &mut NamespaceMap, path: StringIndex) -> SymbolId {
        self.syms.push((None, ns_map.push(Namespace::new(path))))
    }


    pub fn add_enum(&mut self, id: SymbolId, ns_map: &mut NamespaceMap,
                    string_map: &mut StringMap, range: SourceRange,
                    name: StringIndex, mappings: &'me [(OptStringIndex, Generic<'me>)],
                    generics: &'me [StringIndex]) {

        assert!(mappings.iter().all(|x| x.0.is_some()));

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

        for i in mappings {
            let mapping_name = i.0.unwrap();
            let func_name = string_map.concat(name, mapping_name);

            let args = self.arena.alloc_new([FunctionArgument::new(StringMap::VALUE, i.1, false)]);
            let sym = FunctionTy::new(args, ret, FunctionKind::Enum { sym: id });
            let sym = Symbol::new(func_name, generics, SymbolKind::Function(sym));
            let id = self.pending(ns_map, func_name);
            self.add_sym(id, sym);

            let ns = ns_map.get_ns_mut(ns);
            ns.add_sym(mapping_name, id);
        }
    }


    pub fn add_sym(&mut self, id: SymbolId, sym: Symbol<'me>) { 
        debug_assert!(self.syms[id].0.is_none());
        self.syms[id].0 = Some(sym)
    }


    pub fn sym(&mut self, id: SymbolId) -> Symbol<'me> { 
        self.syms[id].0.unwrap()
    }


    pub fn sym_ns(&self, id: SymbolId) -> NamespaceId { 
        self.syms[id].1
    }


    pub fn new_var(&mut self, range: SourceRange) -> Type {
        Type::Var(self.vars.push(Var { sub: None, range }))
    }


    pub fn add_gens(&mut self, generics: &'me [(StringIndex, Type)]) -> GenListId {
        if generics.is_empty() { return GenListId::EMPTY }
        self.tys.push(generics)
    }


    pub fn get_ty(&mut self, ty: SymbolId, generics: &[Type]) -> Type {
        let sym = self.sym(ty);
        let vec = sti::vec::Vec::from_in(self.arena, sym.generics.iter().copied().zip(generics.iter().copied()));
        let generics = if generics.is_empty() { GenListId::EMPTY }
                       else { self.add_gens(copy_slice_in(self.arena, vec.leak())) };
        Type::Ty(ty, generics)
    }

}


impl<'me> Symbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex], kind: SymbolKind<'me>) -> Self {
        Self { name, generics, kind }
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
    

    pub fn to_ty(self, gens: &[(StringIndex, Type)], map: &mut SymbolMap) -> Result<Type, Error> {
        match self.kind {
            GenericKind::Generic(v) => gens.iter()
                                        .find(|x| x.0 == v)
                                        .copied()
                                        .map(|x| x.1)
                                        .ok_or(Error::UnknownType(v, self.range)),


            GenericKind::Sym(symbol, generics) => {
                let pool = Arena::tls_get_rec();
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


impl VarId {
    fn occurs_in(self, map: &SymbolMap, ty: Type) -> bool {
        match ty {
            Type::Ty(_, gens) => map.tys[gens].iter().any(|x| self.occurs_in(map, x.1)),
            Type::Var(v) => {
                if self == v { return true }
                let sub = map.vars[v].sub;
                match sub {
                    Some(ty) => self.occurs_in(map, ty),
                    None => self == v,
                }
            },
        }
    }
}



impl SymbolId {
    pub const UNIT  : Self = Self(0);
    pub const I8    : Self = Self(1);
    pub const I16   : Self = Self(2);
    pub const I32   : Self = Self(3);
    pub const I64   : Self = Self(4);
    pub const U8    : Self = Self(5);
    pub const U16   : Self = Self(6);
    pub const U32   : Self = Self(7);
    pub const U64   : Self = Self(8);
    pub const F32   : Self = Self(9);
    pub const F64   : Self = Self(10);
    pub const BOOL  : Self = Self(11); // +2 for variants
    pub const ERROR : Self = Self(14);
    pub const NEVER : Self = Self(15);
    pub const PTR   : Self = Self(16);
    pub const RANGE : Self = Self(17);
    pub const OPTION: Self = Self(18); // +2 for variants
    pub const RESULT: Self = Self(21); // +2 for variants
    pub const STR   : Self = Self(24);


    pub fn supports_arith(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::F32
            | Self::F64
        )
    }


    pub fn supports_bw(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
        )
    }


    pub fn supports_ord(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::F32
            | Self::F64
        )
    }

    pub fn supports_eq(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::F32
            | Self::F64
            | Self::BOOL
            | Self::UNIT
        )
    }


    pub fn is_num(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
            | Self::F32
            | Self::F64
        )
    }


    pub fn is_int(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
            | Self::U8
            | Self::U16
            | Self::U32
            | Self::U64
        )
    }

    pub fn is_sint(self) -> bool {
        matches!(self,
              Self::I8
            | Self::I16
            | Self::I32
            | Self::I64
        )
    }
}


impl GenListId {
    pub const EMPTY: Self = Self(0);
}


impl<'me> GenericKind<'me> {
    pub const ERROR : Self = Self::Sym(SymbolId::ERROR, &[]);
}
