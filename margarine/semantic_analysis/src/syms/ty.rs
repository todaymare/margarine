use std::hash::{Hash, Hasher};

use common::string_map::{StringIndex, StringMap};
use sti::{format_in, hash::fxhash::FxHasher32, traits::FromIn};

use crate::{errors::Error, syms::{containers::ContainerKind, SymbolKind}};

use super::sym_map::{GenListId, SymbolId, SymbolMap, VarId, VarSub};

#[derive(Clone, Copy, Debug)]
pub enum Sym {
    Ty (SymbolId, GenListId),
    Var(VarId),
}


impl Sym {
    pub fn display<'str>(self, string_map: &StringMap<'str>, map: &mut SymbolMap) -> &'str str {
        self.display_ex(string_map, map, None)
    }
    

    fn display_ex<'str>(self, string_map: &StringMap<'str>,
                            map: &mut SymbolMap, def: Option<StringIndex>) -> &'str str {
        match self.instantiate_shallow(map) {
            Sym::Ty(sym, gens) => {
                let mut str = sti::string::String::new_in(string_map.arena());
                let sym = map.sym(sym);

                let gens = map.gens()[gens];
                let is_tuple = matches!(sym.kind, SymbolKind::Container(cont)
                                                    if cont.kind() == ContainerKind::Tuple);

                if is_tuple {
                    let SymbolKind::Container(cont) = sym.kind
                    else { unreachable!() };

                    str.push_char('(');
                    
                    for (i, f) in cont.fields().iter().enumerate() {
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


            Sym::Var(v) => {
                if let Some(def) = def {
                    return string_map.get(def)
                }

                match map.vars()[v].sub() {
                    VarSub::Concrete(v) => v.display_ex(string_map, map, def),
                    VarSub::Integer => "{integer}",
                    VarSub::Float => "{float}",
                    VarSub::None => "{unknown}",
                }
            },
        }
    }

    pub fn eq(self, map: &mut SymbolMap, oth: Sym) -> bool {
        let a = self.instantiate_shallow(map);
        let b = oth.instantiate_shallow(map);
        match (a, b) {
            (Sym::Ty(symida, gena), Sym::Ty(symidb, genb)) => {
                if matches!(symida, SymbolId::ERR | SymbolId::NEVER) { return true; }
                if matches!(symidb, SymbolId::ERR | SymbolId::NEVER) { return true; }

                let gena = instantiate_gens(map, gena);
                let gena = map.gens()[gena];

                let genb = instantiate_gens(map, genb);
                let genb = map.gens()[genb];

                if symida == symidb {
                    return gena.iter().zip(genb.iter()).all(|(ta, tb)| ta.1.eq(map, tb.1));
                }

                let syma = map.sym(symida);
                let symb = map.sym(symidb);

                match (syma.kind, symb.kind) {
                    (SymbolKind::Function(fa), SymbolKind::Function(fb)) => {
                        if fa.args().len() != fb.args().len() { return false; }

                        let reta = fa.ret().to_ty(gena, map).unwrap_or(Sym::ERROR);
                        let retb = fb.ret().to_ty(genb, map).unwrap_or(Sym::ERROR);

                        if !reta.eq(map, retb) {
                            return false;
                        }

                        for (aa, ab) in fa.args().iter().zip(fb.args().iter()) {
                            let aa = aa.symbol().to_ty(gena, map).unwrap_or(Sym::ERROR);
                            let ab = ab.symbol().to_ty(genb, map).unwrap_or(Sym::ERROR);

                            if !aa.eq(map, ab) {
                                return false;
                            }
                        }
                    },


                    (SymbolKind::Container(ca), SymbolKind::Container(cb)) => {
                        // is a tuple
                        if ca.kind() != ContainerKind::Tuple
                            || cb.kind() != ContainerKind::Tuple { return false; }

                        if ca.fields().len() != cb.fields().len() { return false; }

                        for (fa, fb) in ca.fields().iter().zip(cb.fields().iter()) {
                            let tfa = fa.1.to_ty(gena, map).unwrap_or(Sym::ERROR);
                            let tfb = fb.1.to_ty(genb, map).unwrap_or(Sym::ERROR);

                            if !tfa.eq(map, tfb) { return false; }
                        }

                        return true;
                    },

                    _ => return false,
                }

                debug_assert_eq!(gena.len(), genb.len());
                return true
            },

            (Sym::Var(ida), Sym::Var(idb)) if ida == idb => { return true }

            (Sym::Var(ida), _) => {
                if ida.occurs_in(map, b) { return false }

                let var = map.vars()[ida].sub();
 
                match var {
                    VarSub::Concrete(ta) if !matches!(ta, Sym::Ty(SymbolId::ERR | SymbolId::NEVER, _)) => b.eq(map, ta),

                    VarSub::Integer if !b.is_int(map) => false,
                    VarSub::Float if !b.is_float(map) => false,

                    _ => {
                        map.vars_mut()[ida].set_sub(VarSub::Concrete(b));
                        true
                    },
                }
            },


            (_, Sym::Var(idb)) => {
                if idb.occurs_in(map, a) { return false }

                let var = map.vars()[idb].sub();
                match var {
                    VarSub::Concrete(ta) if !matches!(ta, Sym::Ty(SymbolId::ERR | SymbolId::NEVER, _)) => b.eq(map, ta),

                    VarSub::Integer if !a.is_int(map) => false,
                    VarSub::Float if !a.is_float(map) => false,

                    _ => {
                        map.vars_mut()[idb].set_sub(VarSub::Concrete(a));
                        true
                    },
                }
            },
        }
    }


    pub fn is_err(self, map: &mut SymbolMap) -> bool {
        if let Ok(sym) = self.sym(map) { sym == SymbolId::ERR }
        else { false }
    }


    pub fn is_never(self, map: &mut SymbolMap) -> bool {
        if let Ok(sym) = self.sym(map) { sym == SymbolId::NEVER }
        else { false }
    }
    

    pub fn ne(self, map: &mut SymbolMap, oth: Sym) -> bool {
        !self.eq(map, oth)
    }


    pub fn sym(self, map: &mut SymbolMap) -> Result<SymbolId, Error> {
        match self.instantiate_shallow(map) {
            Sym::Ty(sym, _) => Ok(sym),
            Sym::Var(id) => {
                let var = &map.vars()[id];
                Err(Error::UnableToInfer(var.range()))
            },
        }
    }


    pub fn gens<'a>(self, map: &SymbolMap<'a>) -> GenListId {
        match self.instantiate_shallow(map) {
            Sym::Ty(_, v) => v,
            Sym::Var(_) => GenListId::EMPTY,
        }
    }


    pub fn instantiate(self, map: &mut SymbolMap, depth: usize) -> Sym {
        if depth == 100 { panic!() }
        match self {
            Sym::Ty(sym, gens) => {
                Sym::Ty(sym, instantiate_gens(map, gens))
            },


            Sym::Var(v) => {
                match map.vars()[v].sub() {
                    VarSub::Concrete(v) => v.instantiate(map, depth + 1),
                    _ => self,
                }
            },
        }
    }


    pub fn instantiate_shallow(self, map: &SymbolMap) -> Sym {
        match self {
            Sym::Ty(_, _) => self,

            Sym::Var(v) => {
                match map.vars()[v].sub() {
                    VarSub::Concrete(v) => v,
                    _ => self
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
        let init = self.instantiate(map, 0);
        match init {
            Sym::Ty(v, g) => {
                v.hash(hasher);

                let arr = map.gens()[g];
                for g in arr.iter() {
                    g.1.hash_ex(map, hasher)
                }
            },


            Sym::Var(_) => unreachable!(),
        }

    }


    pub fn is_num(self, map: &mut SymbolMap) -> bool {
        self.is_int(map) || self.is_float(map)
    }


    pub fn is_int(self, map: &mut SymbolMap) -> bool {
        let ty = self.instantiate_shallow(map);
        match ty {
            Sym::Ty(v, _) => v.is_int(),
            Sym::Var(v) => {
                let var = map.vars_mut()[v].sub();
                match var {
                    VarSub::Concrete(v) => v.is_int(map),
                    VarSub::Integer => true,
                    VarSub::Float => false,
                    VarSub::None => {
                        map.vars_mut()[v].set_sub(VarSub::Integer);
                        true
                    },
                }
            },
        }
    }


    pub fn is_float(self, map: &mut SymbolMap) -> bool {
        let ty = self.instantiate_shallow(map);
        match ty {
            Sym::Ty(v, _) => v.is_float(),
            Sym::Var(v) => {
                let var = map.vars_mut()[v].sub();
                match var {
                    VarSub::Concrete(v) => v.is_int(map),
                    VarSub::Integer => false,
                    VarSub::Float => true,
                    VarSub::None => {
                        map.vars_mut()[v].set_sub(VarSub::Float);
                        true
                    },
                }
            },
        }
    }
}


#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct TypeHash(u32);


fn instantiate_gens(map: &mut SymbolMap, gen: GenListId) -> GenListId {
    let gens = map.gens()[gen];
    let arena = map.arena();
    let vec = sti::vec::Vec::from_in(arena, gens.iter().map(|g| (g.0, g.1.instantiate(map, 0))));
    map.add_gens(vec.leak())
}


impl Sym {
    pub const UNIT : Self = Self::Ty(SymbolId::UNIT , GenListId::EMPTY);
    pub const I8   : Self = Self::Ty(SymbolId::I8   , GenListId::EMPTY);
    pub const I16  : Self = Self::Ty(SymbolId::I16  , GenListId::EMPTY);
    pub const I32  : Self = Self::Ty(SymbolId::I32  , GenListId::EMPTY);
    pub const I64  : Self = Self::Ty(SymbolId::I64  , GenListId::EMPTY);
    pub const ISIZE: Self = Self::Ty(SymbolId::ISIZE, GenListId::EMPTY);
    pub const U8   : Self = Self::Ty(SymbolId::U8   , GenListId::EMPTY);
    pub const U16  : Self = Self::Ty(SymbolId::U16  , GenListId::EMPTY);
    pub const U32  : Self = Self::Ty(SymbolId::U32  , GenListId::EMPTY);
    pub const U64  : Self = Self::Ty(SymbolId::U64  , GenListId::EMPTY);
    pub const USIZE: Self = Self::Ty(SymbolId::USIZE, GenListId::EMPTY);
    pub const F32  : Self = Self::Ty(SymbolId::F32  , GenListId::EMPTY);
    pub const F64  : Self = Self::Ty(SymbolId::F64  , GenListId::EMPTY);
    pub const BOOL : Self = Self::Ty(SymbolId::BOOL , GenListId::EMPTY);
    pub const ERROR: Self = Self::Ty(SymbolId::ERR, GenListId::EMPTY);
    pub const NEVER: Self = Self::Ty(SymbolId::NEVER, GenListId::EMPTY);
    pub const RANGE: Self = Self::Ty(SymbolId::RANGE, GenListId::EMPTY);
    pub const STR  : Self = Self::Ty(SymbolId::STR  , GenListId::EMPTY);
}


