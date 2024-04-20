use std::{collections::HashMap, fmt::Write, hash::{Hash, Hasher}};

use common::{copy_slice_in, source::SourceRange, string_map::{OptStringIndex, StringIndex, StringMap}};
use llvm_api::{builder::Local, tys::IsType, Context};
use parser::nodes::decl::StructKind;
use sti::{arena::Arena, define_key, format_in, hash::fxhash::FxHasher32, keyed::{KVec, Key}};

use crate::{scope::{Scope, ScopeId, VariableScope}, Analyzer};

define_key!(u32, pub TypeSymbolId);
define_key!(u32, pub TypeId);


#[derive(Debug, Clone, Copy)]
pub struct TypeSymbol<'me> {
    pub name: StringIndex,
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
    pub symbol  : TypeSymbolId,
    llvm_ty : llvm_api::tys::Type,

    // technically we don't need this
    // as it can be found from evaluating
    // the generics. but it's a nice optimisation
    name    : StringIndex,

    // assumes the generics are in the same
    // order as the symbol's generics
    generics: &'me [Type],
}


#[derive(Debug)]
pub struct SymbolMap<'me> {
    symbols     : KVec   <TypeSymbolId      , Option<Symbol<'me>>>,
    tys         : KVec   <TypeId            , TypeValue<'me>>,
    named_tuples: HashMap<&'me [OptStringIndex], TypeSymbolId>,
    arena       : &'me Arena,
}


#[derive(Debug, Clone, Copy)]
pub struct Type(TypeId);


#[derive(Debug)]
struct Symbol<'me> {
    symbol: TypeSymbol<'me>,
    maps  : HashMap<u32, TypeId>,
}


impl<'me> SymbolMap<'me> {
    pub fn new(arena: &'me Arena, ctx: &mut Context) -> Self {
        let mut slf = SymbolMap {
            symbols: KVec::new(),
            tys    : KVec::new(),
            named_tuples : HashMap::new(),
            arena,
        };
        let map = &[];
        
        // register types
        macro_rules! register_int {
            ($func: ident, $name: ident, $size: literal) => {
                let sym = slf.pending();
                slf.add_sym(sym, TypeSymbol::new(StringMap::$name, &[], TypeSymbolKind::BuiltIn));
                let tyid = slf.tys.push(TypeValue::new(sym, ctx.$func($size).ty(), StringMap::$name, map));
                slf.insert_to_sym(sym, &[], tyid);
                assert_eq!(sym, TypeSymbolId::$name);
                assert_eq!(tyid, Type::$name.0);
            };
        }

        register_int!(signed_int  , I8 , 8 );
        register_int!(signed_int  , I16, 16);
        register_int!(signed_int  , I32, 32);
        register_int!(signed_int  , I64, 64);

        register_int!(unsigned_int, U8 , 8 );
        register_int!(unsigned_int, U16, 16);
        register_int!(unsigned_int, U32, 32);
        register_int!(unsigned_int, U64, 64);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::F32, &[], TypeSymbolKind::BuiltIn));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.f32().ty(), StringMap::F32, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::F32);
        assert_eq!(tyid, Type::F32.0);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::F64, &[], TypeSymbolKind::BuiltIn));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.f64().ty(), StringMap::F64, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::F64);
        assert_eq!(tyid, Type::F64.0);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::UNIT, &[], TypeSymbolKind::BuiltIn));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.zst().ty(), StringMap::UNIT, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::UNIT);
        assert_eq!(tyid, Type::UNIT.0);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::ERROR, &[], TypeSymbolKind::BuiltIn));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.zst().ty(), StringMap::ERROR, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::ERROR);
        assert_eq!(tyid, Type::ERROR.0);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::NEVER, &[], TypeSymbolKind::BuiltIn));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.zst().ty(), StringMap::NEVER, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::NEVER);
        assert_eq!(tyid, Type::NEVER.0);

        let sym = slf.pending();
        slf.add_sym(sym, TypeSymbol::new(StringMap::BOOL, &[], TypeSymbolKind::Enum(Enum::Tag(&[StringMap::FALSE, StringMap::TRUE]))));
        let tyid = slf.tys.push(TypeValue::new(sym, ctx.unsigned_int(1).ty(), StringMap::BOOL, map));
        slf.insert_to_sym(sym, &[], tyid);
        assert_eq!(sym, TypeSymbolId::BOOL);
        assert_eq!(tyid, Type::BOOL.0);

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
    pub fn get_ty_val(&self, ty: Type) -> TypeValue<'me> {
        self.tys[ty.0]
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


impl<'me, 'out, 'ast, 'str> Analyzer<'me, 'out, 'ast, 'str> {
    #[inline(always)]
    pub fn get_ty(&mut self, context: &mut Context,
                  sym: TypeSymbolId,
                  gens: &[Type]) -> Type {
        if let Some(val) = self.types.get_from_sym(sym, gens) { return Type(val) }
        let entry = self.types.symbols[sym].as_mut().unwrap();

        assert!(!matches!(entry.symbol.kind(), TypeSymbolKind::BuiltIn), "{entry:?}");
        assert_eq!(entry.symbol.generics.len(), gens.len());

        let pool = Arena::tls_get_temp();
        if let TypeSymbolKind::Structure(strct) = entry.symbol.kind {
            if strct.is_tuple { return self.tuple_from_sym(context, sym, gens) }
        };

        // generate the name for the type
        let name = 'l: {
            if gens.is_empty() { break 'l self.string_map.get(entry.symbol.name) }
            let mut str = sti::string::String::new_in(&*pool);
            str.push(self.string_map.get(entry.symbol.name));
            str.push("<");
 
            for (i, t) in gens.iter().enumerate() {
                if i != 0 { str.push(", "); }

                let ty = self.types.get_ty_val(*t);
                let sym = self.types.get_sym(ty.symbol);
                let name = sym.name;
                let name = self.string_map.get(name);
                str.push(name);
            }

            str.push_char('>');
           
            str.leak()
        };
        
        // move `gens` into the arena
        let entry = self.types.symbols[sym].as_mut().unwrap();
        let gens_map = {
            let mut map = HashMap::with_capacity(gens.len());
            for (n, g) in entry.symbol.generics().iter().zip(gens.iter()) {
                map.insert(*n, *g);
            }
            sti::boks::Box::new_in(self.types.arena, map).leak()
        };
        let gens = copy_slice_in(self.types.arena, gens);

        let entry = self.types.symbols[sym].as_mut().unwrap();
        let value = match entry.symbol.kind {
            TypeSymbolKind::Structure(strct) => {
                debug_assert!(!strct.is_tuple);

                // fiedls
                let mut llvm_fields = sti::vec::Vec::with_cap_in(&*pool, strct.fields.len());
                for field in strct.fields.iter() {
                    let ty = self.gen_to_ty(context, field.symbol, gens_map).unwrap_or(Type::ERROR);
                    let ty = self.types.get_ty_val(ty);
                    llvm_fields.push(ty.llvm_ty());
                }

                // finalise
                let llvm_ty = context.structure(&*name);
                llvm_ty.set_fields(context, &*llvm_fields);

                TypeValue {
                    symbol: sym, llvm_ty: llvm_ty.ty(),
                    generics: gens, name: self.string_map.insert(&*name)
                }

            },
            TypeSymbolKind::Enum(_) => todo!(),
            TypeSymbolKind::BuiltIn => panic!("{entry:?}"),
        };
        
        let ty = self.types.tys.push(value);
        self.types.insert_to_sym(sym, gens, ty);
        Type(ty)
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

    
    pub fn tuple_from_sym(&mut self, ctx: &mut Context, sym_id: TypeSymbolId, generics: &[Type]) -> Type {
        let pool = Arena::tls_get_temp();
        let sym  = self.types.get_sym(sym_id);
        debug_assert!(if let TypeSymbolKind::Structure(s) = sym.kind { s.is_tuple } else { false });

        if let Some(val) = self.types.get_from_sym(sym_id, generics) { return Type(val) }

        let TypeSymbolKind::Structure(strct) = sym.kind
        else { unreachable!() };
        assert!(strct.is_tuple);

        let name = {
            let mut name = sti::string::String::new_in(&*pool);
            name.push_char('(');
            for (i, m) in generics.iter().zip(strct.fields.iter()).enumerate() {
                if i != 0 { name.push(", ") }
                let ty_name = self.types.get_ty_val(*m.0).name;
                let ty_name = self.string_map.get(ty_name);
                let _ = match m.1.name.to_option() {
                    Some(v) => write!(name, "{}: {}", self.string_map.get(v), ty_name),
                    None    => write!(name, "{}", ty_name),
                };
            }
            name.push_char(')');
            name
        };

        let llvm_ty = ctx.structure(&*name);
        let name_id = self.string_map.insert(&*name);

        let llvm_fields = {
            let mut vec = sti::vec::Vec::with_cap_in(&*pool, generics.len());
            for i in generics {
                vec.push(self.types.get_ty_val(*i).llvm_ty)
            }
            vec
        };

        llvm_ty.set_fields(ctx, &*llvm_fields);

        let generics = copy_slice_in(self.types.arena, generics);
        let tv = TypeValue::new(sym_id, llvm_ty.ty(), name_id, generics);
        let tyid = self.types.tys.push(tv);
        self.types.insert_to_sym(sym_id, generics, tyid);

        Type(tyid)
    }


    #[inline(always)]
    pub fn tuple(&mut self, ctx: &mut Context, mapping: &[(OptStringIndex, Type)]) -> Type {
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
        self.tuple_from_sym(ctx, sym, tys)
    }
}


impl Type {
    pub const I8   : Self = Self(TypeId(0 ));
    pub const I16  : Self = Self(TypeId(1 ));
    pub const I32  : Self = Self(TypeId(2 ));
    pub const I64  : Self = Self(TypeId(3 ));
    pub const U8   : Self = Self(TypeId(4 ));
    pub const U16  : Self = Self(TypeId(5 ));
    pub const U32  : Self = Self(TypeId(6 ));
    pub const U64  : Self = Self(TypeId(7 ));
    pub const F32  : Self = Self(TypeId(8 ));
    pub const F64  : Self = Self(TypeId(9 ));
    pub const UNIT : Self = Self(TypeId(10));
    pub const ERROR: Self = Self(TypeId(11));
    pub const NEVER: Self = Self(TypeId(12));
    pub const BOOL : Self = Self(TypeId(13));
    pub const TUPLE: Self = Self(TypeId(13));

    pub fn new(ty: TypeId) -> Type { Type(ty) }

    pub fn display<'str>(self,
                         string_map: &StringMap<'str>,
                         symbol_map: &SymbolMap) -> &'str str {
        let name = symbol_map.tys[self.0].name;
        string_map.get(name)
    }


    fn hash(self, ty_map: &SymbolMap, hasher: &mut impl Hasher) {
        let this = ty_map.get_ty_val(self);
        let this_sym = ty_map.get_sym(this.symbol());
        let TypeSymbolKind::Structure(this_strct) = this_sym.kind
        else { false.hash(hasher); return self.0.hash(hasher) };
        if !this_strct.is_tuple { false.hash(hasher); return self.0.hash(hasher) }

        true.hash(hasher);
        this_sym.generics.hash(hasher);
        this.generics.iter().for_each(|x| x.hash(ty_map, hasher));
    }


    pub fn eq(self, oth: Type, ty_map: &SymbolMap) -> bool {
        if self.0 == oth.0 { return true }

        let this = ty_map.get_ty_val(self);
        let this_sym = ty_map.get_sym(this.symbol());
        let TypeSymbolKind::Structure(this_strct) = this_sym.kind
        else { return false };
        if !this_strct.is_tuple { return false }

        let oth = ty_map.get_ty_val(oth );
        let oth_sym = ty_map.get_sym(this.symbol());
        let TypeSymbolKind::Structure(oth_strct) = oth_sym.kind
        else { return false };
        if !oth_strct.is_tuple { return false }

        if this_sym.generics != oth_sym.generics { return false }
        if this_strct.fields.len() != oth_strct.fields.len() { return false }
        if this.generics.iter().zip(oth.generics.iter()).all(|(a, b)| a.eq(*b, ty_map)) { return false }
        true
    }


    pub fn ne(self, oth: Type, ty_map: &SymbolMap) -> bool {
        !self.eq(oth, ty_map)
    }


    pub fn supports_arith(self) -> bool {
        true
        || self.0 == Self::I8.0
        || self.0 == Self::I16.0
        || self.0 == Self::I32.0
        || self.0 == Self::I64.0
        || self.0 == Self::U8.0
        || self.0 == Self::U16.0
        || self.0 == Self::U32.0
        || self.0 == Self::U64.0
        || self.0 == Self::F32.0
        || self.0 == Self::F64.0
    }


    pub fn supports_ord(self) -> bool {
        true
        || self.0 == Self::I8.0
        || self.0 == Self::I16.0
        || self.0 == Self::I32.0
        || self.0 == Self::I64.0
        || self.0 == Self::U8.0
        || self.0 == Self::U16.0
        || self.0 == Self::U32.0
        || self.0 == Self::U64.0
        || self.0 == Self::F32.0
        || self.0 == Self::F64.0
    }


    pub fn supports_eq(self) -> bool {
        true
        || self.0 == Self::I8.0
        || self.0 == Self::I16.0
        || self.0 == Self::I32.0
        || self.0 == Self::I64.0
        || self.0 == Self::U8.0
        || self.0 == Self::U16.0
        || self.0 == Self::U32.0
        || self.0 == Self::U64.0
        || self.0 == Self::F32.0
        || self.0 == Self::F64.0
    }


    pub fn supports_bw(self) -> bool {
        true
        || self.0 == Self::I8.0
        || self.0 == Self::I16.0
        || self.0 == Self::I32.0
        || self.0 == Self::I64.0
        || self.0 == Self::U8.0
        || self.0 == Self::U16.0
        || self.0 == Self::U32.0
        || self.0 == Self::U64.0
    }


    pub fn is_sint(self) -> bool {
        true
        || self.0 == Self::I8.0
        || self.0 == Self::I16.0
        || self.0 == Self::I32.0
        || self.0 == Self::I64.0
    }


    pub fn is_uint(self) -> bool {
        true
        || self.0 == Self::U8.0
        || self.0 == Self::U16.0
        || self.0 == Self::U32.0
        || self.0 == Self::U64.0
    }


    pub fn is_f32(self) -> bool {
        self.0 == Self::F32.0
    }


    pub fn is_f64(self) -> bool {
        self.0 == Self::F32.0
    }

    
    pub fn shortcircuit(self) -> bool {
        true
        || self.0 == Self::NEVER.0
        || self.0 == Self::ERROR.0
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


impl<'me> TypeSymbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex], kind: TypeSymbolKind<'me>) -> Self { Self { name, generics, kind } }

    #[inline(always)]
    pub fn generics(&self) -> &'me [StringIndex] { self.generics }

    #[inline(always)]
    pub fn kind(&self) -> TypeSymbolKind<'_> { self.kind }
}


impl<'me> TypeValue<'me> {
    pub fn new(symbol: TypeSymbolId, llvm_ty: llvm_api::tys::Type, name: StringIndex, generics: &'me [Type]) -> Self { Self { symbol, llvm_ty, name, generics } }


    #[inline(always)]
    pub fn symbol(&self) -> TypeSymbolId {
        self.symbol
    }


    #[inline(always)]
    pub fn llvm_ty(&self) -> llvm_api::tys::Type {
        self.llvm_ty
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

