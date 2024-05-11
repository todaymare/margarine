use std::{hash::{Hash, Hasher}, ops::Deref};

use common::string_map::{StringIndex, StringMap};
use sti::{format_in, hash::fxhash::FxHasher32, traits::FromIn};

use crate::{errors::Error, types::{containers::ContainerKind, SymbolKind}};

use super::{containers::Container, GenListId, SymbolId, SymbolMap, VarId};

#[derive(Clone, Copy, Debug)]
pub enum Type {
    Ty (SymbolId, GenListId),
    Var(VarId),
}


impl Type {
    pub fn display<'str>(self, string_map: &StringMap<'str>, map: &mut SymbolMap) -> &'str str {
        self.display_ex(string_map, map, None)
    }
    

    fn display_ex<'str>(self, string_map: &StringMap<'str>,
                            map: &mut SymbolMap, def: Option<StringIndex>) -> &'str str {
        match self.instantiate_shallow(map) {
            Type::Ty(sym, gens) => {
                let mut str = sti::string::String::new_in(string_map.arena());
                let sym = map.sym(sym);

                let gens = map.tys[gens];
                let is_tuple = matches!(sym.kind, SymbolKind::Container(Container { kind: ContainerKind::Tuple, .. }));

                if is_tuple {
                    let SymbolKind::Container(cont) = sym.kind
                    else { unreachable!() };

                    str.push_char('(');
                    
                    for (i, f) in cont.fields.iter().enumerate() {
                        if i != 0 { str.push(", ") }

                        let ty = f.1.to_ty(gens, map).unwrap();
                        str.push(ty.display(string_map, map))
                    }

                    str.push_char(')');

                } else {
                    str.push(string_map.get(sym.name));
                    if !gens.is_empty() {
                        str.push_char('<');

                        for (i, g) in gens.iter().enumerate() {
                            if i != 0 { str.push(", ") }

                            str.push(g.1.display_ex(string_map, map, Some(g.0)));
                        }

                        str.push_char('>');
                    }
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
            (Type::Ty(symida, gena), Type::Ty(symidb, genb)) => {
                if matches!(symida, SymbolId::ERROR | SymbolId::NEVER) { return true; }
                if matches!(symidb, SymbolId::ERROR | SymbolId::NEVER) { return true; }

                let gena = instantiate_gens(map, gena);
                let gena = map.tys[gena];

                let genb = instantiate_gens(map, genb);
                let genb = map.tys[genb];

                if symida == symidb {
                    return gena.iter().zip(genb.iter()).all(|(ta, tb)| ta.1.eq(map, tb.1));
                }

                let syma = map.sym(symida);
                let symb = map.sym(symidb);

                match (syma.kind, symb.kind) {
                    (SymbolKind::Function(fa), SymbolKind::Function(fb)) => {
                        if fa.args.len() != fb.args.len() { return false; }

                        let reta = fa.ret.to_ty(gena, map).unwrap_or(Type::ERROR);
                        let retb = fb.ret.to_ty(genb, map).unwrap_or(Type::ERROR);

                        if !reta.eq(map, retb) {
                            return false;
                        }

                        for (aa, ab) in fa.args.iter().zip(fb.args.iter()) {
                            let aa = aa.symbol.to_ty(gena, map).unwrap_or(Type::ERROR);
                            let ab = ab.symbol.to_ty(genb, map).unwrap_or(Type::ERROR);

                            if !aa.eq(map, ab) {
                                return false;
                            }
                        }
                    },


                    (SymbolKind::Container(ca), SymbolKind::Container(cb)) => {
                        // is a tuple
                        if ca.kind != ContainerKind::Tuple
                            || cb.kind != ContainerKind::Tuple { return false; }

                        if ca.fields.len() != cb.fields.len() { return false; }

                        for (fa, fb) in ca.fields.iter().zip(cb.fields.iter()) {
                            let tfa = fa.1.to_ty(gena, map).unwrap_or(Type::ERROR);
                            let tfb = fb.1.to_ty(genb, map).unwrap_or(Type::ERROR);

                            if !tfa.eq(map, tfb) { return false; }
                        }

                        return true;
                    },

                    _ => return false,
                }

                debug_assert_eq!(gena.len(), genb.len());
                return true
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


    pub fn hash(self, map: &mut SymbolMap) -> TypeHash {
        let mut hasher = FxHasher32::new();
        self.hash_ex(map, &mut hasher);
        TypeHash(hasher.hash)
    }


    fn hash_ex(self, map: &mut SymbolMap, hasher: &mut impl Hasher) {
        let init = self.instantiate(map);
        match init {
            Type::Ty(v, g) => {
                v.hash(hasher);

                let arr = map.tys[g];
                for g in arr.iter() {
                    g.1.hash_ex(map, hasher)
                }
            },


            Type::Var(_) => unreachable!(),
        }

    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TypeHash(u32);


fn instantiate_gens(map: &mut SymbolMap, gen: GenListId) -> GenListId {
    let gens = map.tys[gen];
    let vec = sti::vec::Vec::from_in(map.arena, gens.iter().map(|g| (g.0, g.1.instantiate(map))));
    map.add_gens(vec.leak())
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
    pub const STR  : Self = Self::Ty(SymbolId::STR  , GenListId::EMPTY);
}


