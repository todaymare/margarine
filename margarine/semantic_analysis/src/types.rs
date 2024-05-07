use std::str;

use common::{copy_slice_in, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use sti::{arena::Arena, define_key, format_in, keyed::KVec, string::String, traits::FromIn};

use crate::{errors::Error, namespace::{Namespace, NamespaceId, NamespaceMap}, TyChecker};

define_key!(u32, pub SymbolId);
define_key!(u32, pub GenListId);
define_key!(u32, pub VarId);

#[derive(Debug)]
pub struct SymbolMap<'me> {
    syms : KVec<SymbolId, Option<(Symbol<'me>, NamespaceId)>>,
    tys  : KVec<GenListId, &'me [(StringIndex, Type)]>,
    vars : KVec<VarId, Var>,
    arena: &'me Arena,
    
}


#[derive(Debug, Clone, Copy)]
pub struct Symbol<'me> {
    pub name    : StringIndex,
    pub generics: &'me [StringIndex],
    pub fields  : &'me [(OptStringIndex, Generic<'me>)],
    pub kind    : SymbolKind,
}


#[derive(Debug, Clone, Copy)]
pub enum SymbolKind {
    /// Assumptions
    /// * All fields are named
    Struct,
    /// Assumptions
    /// * All fields are named
    Enum,
    Tuple,
}


#[derive(Debug)]
pub struct Var {
    sub: Option<Type>,
    range: SourceRange,
}


#[derive(Clone, Copy, Debug)]
pub enum Type {
    Ty (SymbolId, GenListId),
    Var(VarId),
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
    pub fn new(arena: &'me Arena, ns_map: &mut NamespaceMap) -> Self {
        let mut slf = Self { syms: KVec::new(), vars: KVec::new(), arena, tys: KVec::new() };
        macro_rules! init {
            ($name: ident) => {
                let pending = slf.pending();
                assert_eq!(pending, SymbolId::$name);
                slf.add_sym(ns_map, pending, Symbol::new(StringMap::$name, &[], &[], SymbolKind::Struct));
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
            let pending = slf.pending();
            assert_eq!(pending, SymbolId::BOOL);
            let fields = [
                (StringMap::TRUE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
                (StringMap::FALSE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            slf.add_sym(ns_map, pending, Symbol::new(StringMap::BOOL, &[], arena.alloc_new(fields), SymbolKind::Enum));
        }

        init!(ERROR);
        init!(NEVER);

        // rc 
        {
            let pending = slf.pending();
            assert_eq!(pending, SymbolId::PTR);
            let fields = [
                (StringMap::COUNT.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::U64, &[]))),
                (StringMap::VALUE.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
            ];

            slf.add_sym(ns_map, pending, Symbol::new(StringMap::PTR, &[StringMap::T], arena.alloc_new(fields), SymbolKind::Struct));
        }

        // range
        {
            let pending = slf.pending();
            assert_eq!(pending, SymbolId::RANGE);
            let fields = [
                (StringMap::LOW.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::I64, &[]))),
                (StringMap::HIGH.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::I64))),
            ];

            slf.add_sym(ns_map, pending, Symbol::new(StringMap::RANGE, &[], arena.alloc_new(fields), SymbolKind::Struct));
        }


        // option 
        {
            let pending = slf.pending();
            assert_eq!(pending, SymbolId::OPTION);
            let fields = [
                (StringMap::SOME.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
                (StringMap::NONE.some(), Generic::new(SourceRange::ZERO, GenericKind::Sym(SymbolId::UNIT, &[]))),
            ];

            slf.add_sym(ns_map, pending, Symbol::new(StringMap::OPTION, &[StringMap::T], arena.alloc_new(fields), SymbolKind::Enum));
        }


        // result 
        {
            let pending = slf.pending();
            assert_eq!(pending, SymbolId::RESULT);
            let fields = [
                (StringMap::OK.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::T))),
                (StringMap::ERROR.some(), Generic::new(SourceRange::ZERO, GenericKind::Generic(StringMap::A))),
            ];

            slf.add_sym(ns_map, pending, Symbol::new(StringMap::OPTION, &[StringMap::T, StringMap::A], arena.alloc_new(fields), SymbolKind::Enum));
        }

        assert_eq!(slf.tys.push(&[]), GenListId::EMPTY);


        slf
    }

    #[inline(always)]
    pub fn pending(&mut self) -> SymbolId {
        self.syms.push(None)
    }


    pub fn add_sym(&mut self, ns_map: &mut NamespaceMap, id: SymbolId, sym: Symbol<'me>) { 
        debug_assert!(self.syms[id].is_none());
        let ns = Namespace::new(sym.name);
        let ns = ns_map.push(ns);
        self.syms[id] = Some((sym, ns))
    }


    pub fn sym(&mut self, id: SymbolId) -> Symbol<'me> { 
        self.syms[id].unwrap().0
    }


    pub fn sym_ns(&mut self, id: SymbolId) -> NamespaceId { 
        self.syms[id].unwrap().1
    }

    pub fn new_var(&mut self, range: SourceRange) -> Type {
        Type::Var(self.vars.push(Var { sub: None, range }))
    }
}


impl<'me> Symbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex],
               fields: &'me [(OptStringIndex, Generic<'me>)], kind: SymbolKind) -> Self {
        Self { name, generics, kind, fields }
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
}


impl Type {
    pub fn display<'str>(self, string_map: &StringMap<'str>, map: &mut SymbolMap) -> &'str str {
        self.display_ex(string_map, map, None)
    }
    

    fn display_ex<'str>(self, string_map: &StringMap<'str>,
                            map: &mut SymbolMap, def: Option<StringIndex>) -> &'str str {
        match self.instantiate_shallow(map) {
            Type::Ty(sym, gens) => {
                let mut str = String::new_in(string_map.arena());
                str.push(string_map.get(map.sym(sym).name));

                let gens = map.tys[gens];
                if !gens.is_empty() {
                    str.push_char('<');

                    for (i, g) in gens.iter().enumerate() {
                        if i != 0 { str.push(", ") }

                        str.push(g.1.display_ex(string_map, map, Some(g.0)));
                    }

                    str.push_char('>');
                }

                str.leak()
            },


            Type::Var(v) => {
                if let Some(def) = def {
                    return string_map.get(def)
                }

                format_in!(string_map.arena(), "{}", v.0).leak()
            },
        }
    }

    pub fn eq(self, map: &mut SymbolMap, oth: Type) -> bool {
        let a = self.instantiate_shallow(map);
        let b = oth.instantiate_shallow(map);
        match (a, b) {
            (Type::Ty(syma, gena), Type::Ty(symb, genb)) => {
                if matches!(syma, SymbolId::ERROR | SymbolId::NEVER) { return true; }
                if matches!(symb, SymbolId::ERROR | SymbolId::NEVER) { return true; }

                // NON TUPLE
                if syma == symb { 
                    let gena = instantiate_gens(map, gena);
                    let gena = map.tys[gena];
                    let genb = instantiate_gens(map, genb);
                    let genb = map.tys[genb];

                    debug_assert_eq!(gena.len(), genb.len());
                    if !gena.iter().zip(genb.iter()).all(|(ta, tb)| ta.1.eq(map, tb.1)) {
                        return false;
                    }
                    return true
                }


                false
            },

            (Type::Var(ida), Type::Var(idb)) if ida == idb => { return true }

            (Type::Var(ida), _) => {
                if ida.occurs_in(map, b) { return false }

                let var = &mut map.vars[ida].sub;
                match *var {
                    Some(ta) if !matches!(ta, Type::Ty(SymbolId::ERROR | SymbolId::NEVER, _)) => b.eq(map, ta),
                    _ => {
                        *var = Some(b);
                        true
                    },
                }
            },

            (_, Type::Var(idb)) => {
                if idb.occurs_in(map, a) { return false }

                let var = &mut map.vars[idb].sub;
                match *var {
                    Some(tb) if !matches!(tb, Type::Ty(SymbolId::ERROR | SymbolId::NEVER, _)) => a.eq(map, tb),
                    _ => {
                        *var = Some(a);
                        true
                    },
                }
            },
        }
    }


    pub fn ne(self, map: &mut SymbolMap, oth: Type) -> bool {
        !self.eq(map, oth)
    }


    pub fn sym(self, map: &mut SymbolMap) -> Result<SymbolId, Error> {
        match self.instantiate_shallow(map) {
            Type::Ty(sym, _) => Ok(sym),
            Type::Var(id) => {
                let var = &map.vars[id];
                Err(Error::UnableToInfer(var.range))
            },
        }
    }


    pub fn gens<'a>(self, map: &SymbolMap<'a>) -> &'a [(StringIndex, Type)] {
        match self.instantiate_shallow(map) {
            Type::Ty(_, v) => {
                map.tys[v]
            },
            Type::Var(_) => &[],
        }
    }


    pub fn instantiate(self, map: &mut SymbolMap) -> Type {
        match self {
            Type::Ty(sym, gens) => {
                Type::Ty(sym, instantiate_gens(map, gens))
            },


            Type::Var(v) => {
                match map.vars[v].sub {
                    Some(v) => v.instantiate(map),
                    None => self,
                }
            },
        }
    }


    pub fn instantiate_shallow(self, map: &SymbolMap) -> Type {
        match self {
            Type::Ty(_, _) => self,

            Type::Var(v) => {
                match map.vars[v].sub {
                    Some(v) => v.instantiate_shallow(map),
                    None => self,
                }
            },
        }
    }
}


fn instantiate_gens(map: &mut SymbolMap, gen: GenListId) -> GenListId {
    let gens = map.tys[gen];
    let vec = sti::vec::Vec::from_in(map.arena, gens.iter().map(|g| (g.0, g.1.instantiate(map))));
    map.tys.push(vec.leak())
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


impl<'out> TyChecker<'_, 'out, '_, '_> {
    pub fn get_ty(&mut self, ty: SymbolId, generics: &[Type]) -> Type {
        let sym = self.types.sym(ty);
        let vec = sti::vec::Vec::from_in(self.types.arena, sym.generics.iter().copied().zip(generics.iter().copied()));
        let generics = if generics.is_empty() { GenListId::EMPTY }
                       else { self.types.tys.push(copy_slice_in(self.types.arena, vec.leak())) };
        Type::Ty(ty, generics)
    }
}


impl SymbolId {
    pub const UNIT : Self = Self(0);
    pub const I8   : Self = Self(1);
    pub const I16  : Self = Self(2);
    pub const I32  : Self = Self(3);
    pub const I64  : Self = Self(4);
    pub const U8   : Self = Self(5);
    pub const U16  : Self = Self(6);
    pub const U32  : Self = Self(7);
    pub const U64  : Self = Self(8);
    pub const F32  : Self = Self(9);
    pub const F64  : Self = Self(10);
    pub const BOOL : Self = Self(11);
    pub const ERROR: Self = Self(12);
    pub const NEVER: Self = Self(13);
    pub const PTR  : Self = Self(14);
    pub const RANGE: Self = Self(15);
    pub const OPTION: Self = Self(16);
    pub const RESULT: Self = Self(17);


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


impl Type {
    pub const UNIT : Self = Self::Ty(SymbolId::UNIT , GenListId::EMPTY);
    pub const I8   : Self = Self::Ty(SymbolId::I8   , GenListId::EMPTY);
    pub const I16  : Self = Self::Ty(SymbolId::I16  , GenListId::EMPTY);
    pub const I32  : Self = Self::Ty(SymbolId::I32  , GenListId::EMPTY);
    pub const I64  : Self = Self::Ty(SymbolId::I64  , GenListId::EMPTY);
    pub const U8   : Self = Self::Ty(SymbolId::U8   , GenListId::EMPTY);
    pub const U16  : Self = Self::Ty(SymbolId::U16  , GenListId::EMPTY);
    pub const U32  : Self = Self::Ty(SymbolId::U32  , GenListId::EMPTY);
    pub const U64  : Self = Self::Ty(SymbolId::U64  , GenListId::EMPTY);
    pub const F32  : Self = Self::Ty(SymbolId::F32  , GenListId::EMPTY);
    pub const F64  : Self = Self::Ty(SymbolId::F64  , GenListId::EMPTY);
    pub const BOOL : Self = Self::Ty(SymbolId::BOOL , GenListId::EMPTY);
    pub const ERROR: Self = Self::Ty(SymbolId::ERROR, GenListId::EMPTY);
    pub const NEVER: Self = Self::Ty(SymbolId::NEVER, GenListId::EMPTY);
    pub const RANGE: Self = Self::Ty(SymbolId::RANGE, GenListId::EMPTY);
}


impl GenListId {
    pub const EMPTY: Self = Self(0);
}


impl<'me> GenericKind<'me> {
    pub const ERROR : Self = Self::Sym(SymbolId::ERROR, &[]);
}
