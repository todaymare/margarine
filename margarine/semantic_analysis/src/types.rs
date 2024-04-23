use common::{copy_slice_in, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use sti::{arena::Arena, define_key, keyed::KVec};

use crate::{errors::Error, TyChecker};

define_key!(u32, pub SymbolId);
define_key!(u32, pub GenListId);
define_key!(u32, pub VarId);

#[derive(Debug)]
pub struct SymbolMap<'me> {
    syms : KVec<SymbolId, Option<Symbol<'me>>>,
    tys  : KVec<GenListId, &'me [Type]>,
    vars : KVec<VarId, Var>,
    arena: &'me Arena,
    
}


#[derive(Debug)]
pub struct Symbol<'me> {
    name    : StringIndex,
    generics: &'me [StringIndex],
    fields  : &'me [(OptStringIndex, Generic<'me>)],
    kind    : SymbolKind,
}


#[derive(Debug)]
pub enum SymbolKind {
    Struct,
    Tuple,
    Enum,
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
    pub fn new(arena: &'me Arena) -> Self {
        let mut slf = Self { syms: KVec::new(), vars: KVec::new(), arena, tys: KVec::new() };
        macro_rules! init {
            ($name: ident) => {
                let pending = slf.pending();
                slf.add_sym(pending, Symbol::new(StringMap::$name, &[], &[], SymbolKind::Struct));
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
            slf.add_sym(pending,Symbol::new(StringMap::BOOL, &[], &[],SymbolKind::Struct));
        }

        init!(ERROR);
        init!(NEVER);

        slf
    }

    #[inline(always)]
    pub fn pending(&mut self) -> SymbolId { self.syms.push(None) }

    pub fn add_sym(&mut self, id: SymbolId, sym: Symbol<'me>) { 
        debug_assert!(self.syms[id].is_none());
        self.syms[id] = Some(sym)
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
}


impl Type {
    pub fn display<'str>(self, string_map: &StringMap<'str>, map: &mut SymbolMap) -> &'str str {
        let sym = match self.sym(map) {
            Ok(v) => v,
            Err(_) => unreachable!(),
        };

        string_map.get(map.syms[sym].as_ref().unwrap().name)
    }

    pub fn eq(self, map: &mut SymbolMap, oth: Type) -> bool {
        let a = self.instantiate(map);
        let b = oth.instantiate(map);
        match (a, b) {
            (Type::Ty(syma, gena), Type::Ty(symb, genb)) => {
                // NON TUPLE
                if syma == symb { 
                    let gena = map.tys[gena];
                    let genb = map.tys[genb];

                    debug_assert_eq!(gena.len(), genb.len());
                    if !gena.iter().zip(genb.iter()).all(|(ta, tb)| ta.eq(map, *tb)) {
                        return false;
                    }
                    return true
                }


                true
            },

            (Type::Var(ida), Type::Var(idb)) if ida == idb => { return true }

            (Type::Var(ida), _) => {
                if ida.occurs_in(map, b) { return false }

                let var = &mut map.vars[ida].sub;
                match *var {
                    Some(ta) => b.eq(map, ta),
                    None => {
                        *var = Some(b);
                        true
                    },
                }
            },

            (_, Type::Var(idb)) => {
                if idb.occurs_in(map, a) { return false }

                let var = &mut map.vars[idb].sub;
                match *var {
                    Some(tb) => a.eq(map, tb),
                    None => {
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
        match self.instantiate(map) {
            Type::Ty(sym, _) => Ok(sym),
            Type::Var(id) => {
                let var = &map.vars[id];
                Err(Error::UnableToInfer(var.range))
            },
        }
    }


    pub fn instantiate(self, map: &mut SymbolMap) -> Type {
        match self {
            Type::Ty(sym, gens) => {
                let gens = map.tys[gens];
                let mut vec = sti::vec::Vec::with_cap_in(map.arena, gens.len());
                gens.iter().for_each(|g| vec.push(g.instantiate(map)));

                Type::Ty(sym, map.tys.push(vec.leak()))
            },


            Type::Var(v) => {
                match map.vars[v].sub {
                    Some(v) => v.instantiate(map),
                    None => self,
                }
            },
        }
    }
}


impl VarId {
    fn occurs_in(self, map: &SymbolMap, ty: Type) -> bool {
        match ty {
            Type::Ty(_, gens) => map.tys[gens].iter().any(|x| self.occurs_in(map, *x)),
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
        let generics = copy_slice_in(self.output, generics);
        Type::Ty(ty, self.types.tys.push(generics))
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
}


impl GenListId {
    pub const EMPTY: Self = Self(0);
}


impl<'me> GenericKind<'me> {
    pub const ERROR : Self = Self::Sym(SymbolId::ERROR, &[]);
}
