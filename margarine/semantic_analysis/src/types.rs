use std::{collections::HashMap, hash::{Hash, Hasher}, fmt::Write};

use common::{copy_slice_in, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use sti::{arena::Arena, define_key, hash::fxhash::FxHasher32, keyed::KVec};

use crate::TyChecker;

define_key!(u32, pub TypeSymbolId);
define_key!(u32, pub TypeId);
define_key!(u32, pub VarId);


#[derive(Debug, Clone, Copy)]
pub struct TypeSymbol<'me> {
    name    : StringIndex,
    generics: &'me [StringIndex],
    kind    : TypeSymbolKind<'me>,
}


#[derive(Debug, Clone, Copy)]
pub enum TypeSymbolKind<'me> {
    Structure(Structure<'me>),
    Enum(Enum<'me>),

    /// This kind comes with the following assumptions
    /// * Type contains **NO** generics
    /// * The type's mapping on the symbol is already done
    BuiltIn,
}


#[derive(Debug, Clone, Copy)]
pub struct Structure<'me> {
    pub fields: &'me [StructureField<'me>],
    pub is_tuple: bool,
}


#[derive(Debug, Clone, Copy)]
pub struct StructureField<'me> {
    pub name    : OptStringIndex,
    pub symbol  : Generic<'me>,
}


#[derive(Debug, Clone, PartialEq, Copy)]
pub struct Generic<'me> {
    pub range: SourceRange,
    pub kind : GenericKind<'me>
}

impl<'me> Generic<'me> {
    pub fn new(range: SourceRange, kind: GenericKind<'me>) -> Self { Self { range, kind } }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GenericKind<'me> {
    Generic(StringIndex),
    Symbol {
        symbol: TypeSymbolId,
        generics: &'me [Generic<'me>],
    },
}


#[derive(Debug, Clone, Copy)]
pub enum Enum<'me> {
    Tag(&'me [StringIndex]),
}


#[derive(Debug, Clone, Copy)]
pub struct TypeValue<'me> {
    symbol  : TypeSymbolId,

    // if is_init is true then generics
    // may not contain any `Type::Var`s
    is_init : bool,

    // assumes the generics are in the same
    // order as the symbol's generics
    generics: &'me [Type],
}


#[derive(Debug)]
pub struct SymbolMap<'me> {
    symbols     : KVec   <TypeSymbolId, Option<Symbol<'me>>>,
    tys         : KVec   <TypeId      , TypeValue<'me>>,
    vars        : KVec   <VarId       , (StringIndex, Option<Type>)>,
    arena       : &'me Arena,
}


#[derive(Debug, Clone, Copy)]
pub enum Type {
    Ty(TypeId),
    Var(VarId),
}


#[derive(Debug)]
struct Symbol<'me> {
    symbol: TypeSymbol<'me>,
    maps  : HashMap<u32, TypeId>,
}


impl<'me> SymbolMap<'me> {
    pub fn new(arena: &'me Arena) -> Self {
        let mut slf = SymbolMap {
            symbols: KVec::new(),
            tys    : KVec::new(),
            vars   : KVec::new(),
            arena,
        };
        let map = &[];
        
        // register types
        macro_rules! register {
            ($func: ident, $name: ident $(, $size: literal)?) => {
                let sym = slf.pending();
                slf.add_sym(sym, TypeSymbol::new(StringMap::$name, &[], TypeSymbolKind::BuiltIn));
                let tyid = slf.tys.push(TypeValue::new(sym, map));
                slf.insert_to_sym(sym, &[], tyid);
                assert_eq!(sym, TypeSymbolId::$name);
                assert_eq!(Some(tyid), Type::$name.tyid(&slf));
            };
        }

        register!(signed_int  , I8 , 8 );
        register!(signed_int  , I16, 16);
        register!(signed_int  , I32, 32);
        register!(signed_int  , I64, 64);

        register!(unsigned_int, U8 , 8 );
        register!(unsigned_int, U16, 16);
        register!(unsigned_int, U32, 32);
        register!(unsigned_int, U64, 64);

        register!(f32, F32);
        register!(f64, F64);
        register!(zst, UNIT);
        register!(zst, ERROR);
        register!(zst, NEVER);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::BOOL, &[], TypeSymbolKind::Enum(Enum::Tag(&[StringMap::FALSE, StringMap::TRUE]))));
        let tyid = slf.tys.push(TypeValue::new(sym, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::BOOL);
        assert_eq!(Some(tyid), Type::BOOL.tyid(&slf));

        slf

    }


    #[inline(always)]
    pub fn pending(&mut self) -> TypeSymbolId {
        self.symbols.push(None)
    }


    #[inline(always)]
    pub fn add_sym(&mut self, tsi: TypeSymbolId, sym: TypeSymbol<'me>) {
        self.symbols[tsi] = Some(Symbol { symbol: sym, maps: HashMap::new() })
    }


    #[inline(always)]
    pub fn get_ty_val(&self, ty: TypeId) -> TypeValue<'me> {
        self.tys[ty]
    }


    #[inline(always)]
    pub fn get_sym(&self, sym: TypeSymbolId) -> TypeSymbol<'me> {
        self.symbols[sym].as_ref().unwrap().symbol
    }


    pub fn insert_to_sym(&mut self, sym: TypeSymbolId, gens: &[Type], ty_id: TypeId) {
        let hash = hash_ty_list(self, gens);

        assert!(self.symbols[sym].as_mut().unwrap().maps.insert(hash, ty_id).is_none());
    }


    pub fn get_from_sym(&mut self, sym: TypeSymbolId, gens: &[Type]) -> Option<TypeId> {
        let hash = hash_ty_list(self, gens);
        self.symbols[sym].as_mut().unwrap().maps.get(&hash).copied()
    }
}


impl<'me, 'out, 'ast, 'str> TyChecker<'me, 'out, 'ast, 'str> {
    #[inline(always)]
    pub fn get_ty(&mut self,
                  sym: TypeSymbolId,
                  gens: &[Type]) -> Type {
        if let Some(val) = self.types.get_from_sym(sym, gens) { return Type::Ty(val) }
        let entry = self.types.symbols[sym].as_mut().unwrap();

        assert!(!matches!(entry.symbol.kind(), TypeSymbolKind::BuiltIn), "{entry:?}");
        assert_eq!(entry.symbol.generics.len(), gens.len());

        if let TypeSymbolKind::Structure(strct) = entry.symbol.kind {
            if strct.is_tuple { return self.tuple_from_sym(sym, gens) }
        };
        
        // move `gens` into the arena
        let gens = copy_slice_in(self.types.arena, gens);

        let entry = self.types.symbols[sym].as_mut().unwrap();
        let value = match entry.symbol.kind {
            TypeSymbolKind::Structure(strct) => {
                debug_assert!(!strct.is_tuple);
                TypeValue {
                    symbol: sym, 
                    generics: gens,
                    is_init: false,
                }

            },
            TypeSymbolKind::Enum(_) => todo!(),
            TypeSymbolKind::BuiltIn => panic!("{entry:?}"),
        };
        
        let ty = self.types.tys.push(value);
        self.types.insert_to_sym(sym, gens, ty);
        Type::Ty(ty)
    }


    pub fn tuple_sym(&mut self, names: &[OptStringIndex]) -> TypeSymbolId {
        let pool = Arena::tls_get_temp();
        let mut generics = sti::vec::Vec::with_cap_in(self.types.arena, names.len());
        let mut fields   = sti::vec::Vec::with_cap_in(self.types.arena, names.len());
        let mut string   = sti::string::String::new_in(&*pool);

        for (i, name) in names.iter().enumerate() {
            string.clear();
            let _ = write!(string, "{i}");
            let gen_name = self.string_map.insert(&string);

            let symbol = Generic::new(SourceRange::ZERO, GenericKind::Generic(gen_name));
            generics.push(gen_name);
            fields.push(StructureField::new(*name, symbol));
        }

        let structure = Structure::new(true, fields.leak());
        let sym = self.types.pending();
        self.types.add_sym(sym, TypeSymbol::new(StringMap::TUPLE, generics.leak(),
                                                TypeSymbolKind::Structure(structure)));
        sym
    }

    
    pub fn tuple_from_sym(&mut self, sym_id: TypeSymbolId, generics: &[Type]) -> Type {
        let sym  = self.types.get_sym(sym_id);
        debug_assert!(if let TypeSymbolKind::Structure(s) = sym.kind { s.is_tuple } else { false });

        if let Some(val) = self.types.get_from_sym(sym_id, generics) { return Type::Ty(val) }

        let TypeSymbolKind::Structure(strct) = sym.kind
        else { unreachable!() };
        assert!(strct.is_tuple);

        let generics = copy_slice_in(self.types.arena, generics);
        let tv = TypeValue::new(sym_id, generics);
        let tyid = self.types.tys.push(tv);
        self.types.insert_to_sym(sym_id, generics, tyid);

        Type::Ty(tyid)
    }


    #[inline(always)]
    pub fn tuple(&mut self, mapping: &[(OptStringIndex, Type)]) -> Type {
        let names = {
            let mut vec = sti::vec::Vec::with_cap_in(self.types.arena, mapping.len());
            for f in mapping { vec.push(f.0) }
            &*vec.leak()
        };

        let tys = {
            let mut vec = sti::vec::Vec::with_cap_in(self.types.arena, mapping.len());
            for f in mapping { vec.push(f.1) }
            &*vec.leak()
        };

        let sym = self.tuple_sym(names);
        self.tuple_from_sym(sym, tys)
    }
}


impl Type {
    pub const I8   : Self = Self::Ty(TypeId::I8 );
    pub const I16  : Self = Self::Ty(TypeId::I16);
    pub const I32  : Self = Self::Ty(TypeId::I32);
    pub const I64  : Self = Self::Ty(TypeId::I64);
    pub const U8   : Self = Self::Ty(TypeId::U8 );
    pub const U16  : Self = Self::Ty(TypeId::U16);
    pub const U32  : Self = Self::Ty(TypeId::U32);
    pub const U64  : Self = Self::Ty(TypeId::U64);
    pub const F32  : Self = Self::Ty(TypeId::F32);
    pub const F64  : Self = Self::Ty(TypeId::F64);
    pub const UNIT : Self = Self::Ty(TypeId::UNIT);
    pub const ERROR: Self = Self::Ty(TypeId::ERROR);
    pub const NEVER: Self = Self::Ty(TypeId::NEVER);
    pub const BOOL : Self = Self::Ty(TypeId::BOOL);

    pub fn new(ty: TypeId) -> Type { Type::Ty(ty) }

    pub fn display<'str>(self,
                         string_map: &StringMap<'str>,
                         symbol_map: &SymbolMap) -> &'str str {
        match self.instantiate_shallow(symbol_map) {
            Type::Ty(id) => {
                let name = symbol_map.tys[id].symbol;
                let name = symbol_map.get_sym(name).name;
                string_map.get(name)
            },

            Type::Var(var) => {
                let name = symbol_map.vars[var].0;
                string_map.get(name)
            },
        }
    }


    pub fn is_recursive(self, symbol_map: &SymbolMap, var: VarId) -> bool {
        match self {
            Type::Ty (id) => {
                let sym = symbol_map.get_ty_val(id);
                if !sym.is_init {
                    for g in sym.generics {
                        if g.is_recursive(symbol_map, var) { return true }
                    }
                }
                false
            },

            Type::Var(v) => v == var,
        }
    }


    pub fn instantiate(self, symbol_map: &mut SymbolMap) -> Type {
        match self {
            Type::Ty (id) => {
                let sym = symbol_map.get_ty_val(id);
                if !sym.is_init {
                    let mut generics = sti::vec::Vec::with_cap_in(symbol_map.arena, sym.generics.len());
                    for g in sym.generics {
                        generics.push(g.instantiate(symbol_map))
                    }

                    let sym = &mut symbol_map.tys[id];
                    sym.generics = generics.leak();
                    sym.is_init  = true;
                }
                self
            },

            Type::Var(v) => symbol_map.vars[v].1.unwrap_or(self),
        }
    }


    pub fn instantiate_shallow(self, symbol_map: &SymbolMap) -> Type {
        if let Type::Var(id) = self {
            if let Some(value) = symbol_map.vars[id].1 { return value }
        }
        self
    }

    
    pub fn assign(self, symbol_map: &mut SymbolMap, var: VarId) -> bool {
        if matches!(self, Type::Var(v) if v == var) { return true }
        if self.is_recursive(symbol_map, var) { return false }
        let sym = &mut symbol_map.vars[var];
        match sym.1 {
            Some(v) => return self.eq(symbol_map, v),
            None => sym.1 = Some(self),
        }
        true

    }


    pub fn eq(self, symbol_map: &mut SymbolMap, oth: Type) -> bool {
        let a = self.instantiate_shallow(symbol_map);
        let b = oth.instantiate_shallow(symbol_map);

        match (a, b) {
            (Type::Ty(ida), Type::Ty(idb)) => {
                if ida == idb { return true }

                // supports structural equality
                let vala = symbol_map.tys[ida];
                let syma = symbol_map.get_sym(vala.symbol);
                let TypeSymbolKind::Structure(structa) = syma.kind
                else { return false };
                if !structa.is_tuple { return false };

                let valb = symbol_map.tys[idb];
                let symb = symbol_map.get_sym(valb.symbol);
                let TypeSymbolKind::Structure(structb) = symb.kind
                else { return false };
                if !structb.is_tuple { return false };

                // structural equality
                if vala.generics.len() != valb.generics.len() { return false }
                for (a, b) in vala.generics.iter().zip(valb.generics.iter()) {
                    if !a.eq(symbol_map, *b) { return false }
                }

                true
            },

            (Type::Var(ida), _) => oth.assign(symbol_map, ida),
            (_, Type::Var(idb)) => self.assign(symbol_map, idb),
        }
    }


    fn hash(self, ty_map: &SymbolMap, hasher: &mut impl Hasher) {
        let slf = match self {
            Type::Ty(v) => v,
            Type::Var(_) => todo!(),
        };
        let this = ty_map.get_ty_val(slf);
        let this_sym = ty_map.get_sym(this.symbol());
        let TypeSymbolKind::Structure(this_strct) = this_sym.kind
        else { false.hash(hasher); return slf.hash(hasher) };
        if !this_strct.is_tuple { false.hash(hasher); return slf.hash(hasher) }

        true.hash(hasher);
        this_sym.generics.hash(hasher);
        this.generics.iter().for_each(|x| x.hash(ty_map, hasher));
    }


    pub fn ne(self, symbol_map: &mut SymbolMap, oth: Type) -> bool {
        !self.eq(symbol_map, oth)
    }


    pub fn tyid(self, symbol_map: &SymbolMap) -> Option<TypeId> {
        match self.instantiate_shallow(symbol_map) {
            Type::Ty(v) => Some(v),
            Type::Var(_) => None,
        }
    }


    pub fn supports_arith(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::I8
        || id == TypeId::I16
        || id == TypeId::I32
        || id == TypeId::I64
        || id == TypeId::U8
        || id == TypeId::U16
        || id == TypeId::U32
        || id == TypeId::U64
        || id == TypeId::F32
        || id == TypeId::F64
    }


    pub fn supports_ord(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::I8
        || id == TypeId::I16
        || id == TypeId::I32
        || id == TypeId::I64
        || id == TypeId::U8
        || id == TypeId::U16
        || id == TypeId::U32
        || id == TypeId::U64
        || id == TypeId::F32
        || id == TypeId::F64
    }


    pub fn supports_eq(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::I8
        || id == TypeId::I16
        || id == TypeId::I32
        || id == TypeId::I64
        || id == TypeId::U8
        || id == TypeId::U16
        || id == TypeId::U32
        || id == TypeId::U64
        || id == TypeId::F32
        || id == TypeId::F64
    }


    pub fn supports_bw(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::I8
        || id == TypeId::I16
        || id == TypeId::I32
        || id == TypeId::I64
        || id == TypeId::U8
        || id == TypeId::U16
        || id == TypeId::U32
        || id == TypeId::U64
    }


    pub fn is_sint(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::I8
        || id == TypeId::I16
        || id == TypeId::I32
        || id == TypeId::I64
    }


    pub fn is_uint(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        true
        || id == TypeId::U8
        || id == TypeId::U16
        || id == TypeId::U32
        || id == TypeId::U64
    }


    pub fn is_f32(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        id == TypeId::F32
    }


    pub fn is_f64(self, symbol_map: &SymbolMap) -> bool {
        let Some(id) = self.tyid(symbol_map) else { return false };
        id == TypeId::F32
    }

}


impl TypeSymbolId {
    pub const I8   : Self = Self(0 );
    pub const I16  : Self = Self(1 );
    pub const I32  : Self = Self(2 );
    pub const I64  : Self = Self(3 );
    pub const U8   : Self = Self(4 );
    pub const U16  : Self = Self(5 );
    pub const U32  : Self = Self(6 );
    pub const U64  : Self = Self(7 );
    pub const F32  : Self = Self(8 );
    pub const F64  : Self = Self(9 );
    pub const UNIT : Self = Self(10);
    pub const ERROR: Self = Self(11);
    pub const NEVER: Self = Self(12);
    pub const BOOL : Self = Self(13);
    pub const TUPLE: Self = Self(14);
}


impl TypeId {
    pub const I8   : Self = Self(0 );
    pub const I16  : Self = Self(1 );
    pub const I32  : Self = Self(2 );
    pub const I64  : Self = Self(3 );
    pub const U8   : Self = Self(4 );
    pub const U16  : Self = Self(5 );
    pub const U32  : Self = Self(6 );
    pub const U64  : Self = Self(7 );
    pub const F32  : Self = Self(8 );
    pub const F64  : Self = Self(9 );
    pub const UNIT : Self = Self(10);
    pub const ERROR: Self = Self(11);
    pub const NEVER: Self = Self(12);
    pub const BOOL : Self = Self(13);
    pub const TUPLE: Self = Self(14);
}



impl<'me> TypeSymbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex], kind: TypeSymbolKind<'me>) -> Self { Self { name, generics, kind } }

    #[inline(always)]
    pub fn generics(&self) -> &'me [StringIndex] { self.generics }

    #[inline(always)]
    pub fn kind(&self) -> TypeSymbolKind<'_> { self.kind }
}


impl<'me> TypeValue<'me> {
    pub fn new(symbol: TypeSymbolId, generics: &'me [Type]) -> Self { Self { symbol, generics, is_init: false } }


    #[inline(always)]
    pub fn symbol(&self) -> TypeSymbolId {
        self.symbol
    }

    pub fn generics(&self) -> &[Type] {
        self.generics
    }
}


impl<'me> Structure<'me> {
    pub fn new(is_tuple: bool, fields: &'me [StructureField<'me>]) -> Self { Self { fields, is_tuple } }
}


impl<'me> StructureField<'me> {
    pub fn new(name: OptStringIndex, symbol: Generic<'me>) -> Self { Self { name, symbol } }

    #[inline(always)]
    pub fn symbol(&self) -> Generic { self.symbol }
}

pub fn hash_ty_list(map: &SymbolMap, ty: &[Type]) -> u32 {
    let mut hasher = FxHasher32::new();
    ty.len().hash(&mut hasher);
    ty.iter().for_each(|x| x.hash(map, &mut hasher));
    hasher.hash
}

