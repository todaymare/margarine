use common::string_map::{StringIndex, StringMap};
use wasm::WasmType;

use crate::types::ty_map::TypeId;

use super::{ty_map::TypeMap, ty_sym::{TypeEnumKind, TypeKind, TypeStructStatus}};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,

    Unit,
    Never,
    Error,

    Custom(TypeId),
}


impl Type {
    pub const BOOL : Type = Type::Custom(TypeId::BOOL);
    pub const RANGE: Type = Type::Custom(TypeId::RANGE);
    pub const STR  : Type = Type::Custom(TypeId::STR);
}


impl Type {
    ///
    /// Returns the textual representation of
    /// the type. This does **NOT** have to be unique
    /// 
    pub fn display<'a>(
        self,
        string_map: &StringMap<'a>,
        types: &TypeMap,
    ) -> &'a str {
        match self {
            Type::I8     => "i8",
            Type::I16    => "i16",
            Type::I32    => "i32",
            Type::I64    => "i64",
            Type::U8     => "i8",
            Type::U16    => "u16",
            Type::U32    => "u32",
            Type::U64    => "u64",
            Type::F32    => "f32",
            Type::F64    => "f64",
            Type::Unit   => "()",
            Type::Never  => "never",
            Type::Error  => "error",

            Type::Custom(id) => {
                let display_name = types.path(id);
                string_map.get(display_name)
            },
        }
    }


    pub fn path (
        self,
        types: &TypeMap,
    ) -> StringIndex {
        match self {
            Type::I8  => StringMap::I8,
            Type::I16 => StringMap::I16,
            Type::I32 => StringMap::I32,
            Type::I64 => StringMap::I64,
            Type::U8  => StringMap::U8,
            Type::U16 => StringMap::U16,
            Type::U32 => StringMap::U32,
            Type::U64 => StringMap::U64,
            Type::F32 => StringMap::F32,
            Type::F64 => StringMap::F64,

            Type::Unit => StringMap::UNIT,
            Type::Never => StringMap::NEVER,
            Type::Error => StringMap::ERR,
            Type::Custom(v) => types.path(v),
        }
    }


    ///
    /// Checks for literal equality
    /// Same thing as the result of deriving PartialEq
    ///
    pub fn eq_lit(self, oth: Type) -> bool {
        match (self, oth) {
            | (Type::I8 , Type::I8 ) 
            | (Type::I16, Type::I16) 
            | (Type::I32, Type::I32) 
            | (Type::I64, Type::I64) 
            | (Type::U8 , Type::U8 ) 
            | (Type::U16, Type::U16) 
            | (Type::U32, Type::U32) 
            | (Type::U64, Type::U64) 
            | (Type::F32, Type::F32) 
            | (Type::F64, Type::F64) 
            | (Type::Unit, Type::Unit) 
            | (Type::Never, Type::Never) 
            | (Type::Error, Type::Error) 
            => true,

            (Type::Custom(id1), Type::Custom(id2)) => id1 == id2,

            _ => false,
        }
    }


    ///
    /// Checks for semantic equality
    ///
    /// Semantics:
    /// - If either `self` or `oth` are of type `Error`
    ///   or `Never` return true
    /// - Otherwise, proceed with `Self::eq_lit`
    ///
    pub fn eq_sem(self, oth: Type) -> bool {
        match (self, oth) {
            | (Type::Error, _) | (_, Type::Error)
            | (Type::Never, _) | (_, Type::Never)
            => true,

            _ => self.eq_lit(oth),
        }
    }


    ///
    /// Returns true if the type is an integer
    /// or a float. Otherwise returns false
    ///
    #[inline(always)]
    pub fn is_number(self) -> bool {
        self.is_integer()
        || self.is_float()
    }


    #[inline(always)]
    pub fn is_integer(self) -> bool {
        match self {
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::U8
            | Type::U16
            | Type::U32
            | Type::U64 => true, 

            Type::Never => true,
            Type::Error => true,

            _ => false
        }
    }


    #[inline(always)]
    pub fn is_float(self) -> bool {
        match self {
            | Type::F32
            | Type::F64 => true, 

            Type::Never => true,
            Type::Error => true,

            _ => false
        }
    }


    #[inline(always)]
    pub fn is_signed(self) -> bool {
        match self {
            | Type::I8
            | Type::I16
            | Type::I32
            | Type::I64
            | Type::F32
            | Type::F64 => true, 

            _ => false, 
        }
    }


    pub fn size(self, ty_map: &TypeMap) -> usize {
        match self {
            Type::I8  => 1, 
            Type::I16 => 2, 
            Type::I32 => 4, 
            Type::I64 => 8, 
            Type::U8  => 1, 
            Type::U16 => 2, 
            Type::U32 => 4, 
            Type::U64 => 8, 
            Type::F32 => 4,
            Type::F64 => 8,
            Type::Unit => 0,
            Type::Never => 0,
            Type::Error => 0,

            Type::Custom(v) => {
                let ty = ty_map.get(v);
                ty.size()
            },
        }
    }

    pub fn to_wasm_ty(self, ty_map: &TypeMap) -> WasmType {
        match self {
            Type::I8  => WasmType::I32,
            Type::I16 => WasmType::I32,
            Type::I32 => WasmType::I32,
            Type::I64 => WasmType::I64, 
            Type::U8  => WasmType::I32,
            Type::U16 => WasmType::I32,
            Type::U32 => WasmType::I32,
            Type::U64 => WasmType::I64,
            Type::F32 => WasmType::F32,
            Type::F64 => WasmType::F64,
            
            Type::Unit => WasmType::I64,
            Type::Never => WasmType::I64,
            Type::Error => WasmType::I64,

            Type::Custom(v) => {
                let ty = ty_map.get(v);
                match ty.kind() {
                    TypeKind::Struct(s)
                        => {
                            if s.status == TypeStructStatus::Ptr { return WasmType::I32 }
                            WasmType::Ptr { size: ty.size() }
                        },

                    TypeKind::Enum(v) => {
                        match v.kind() {
                            TypeEnumKind::TaggedUnion(_) => WasmType::Ptr { size: ty.size() },
                            TypeEnumKind::Tag(_) => WasmType::I32,
                        }
                    },

                    TypeKind::Error => WasmType::I64,
                }
            },
        }
    }
}


impl core::hash::Hash for Type {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::I8  => state.write_u8(0),
            Type::I16 => state.write_u8(1),
            Type::I32 => state.write_u8(2),
            Type::I64 => state.write_u8(3),
            Type::U8  => state.write_u8(4),
            Type::U16 => state.write_u8(5),
            Type::U32 => state.write_u8(7),
            Type::U64 => state.write_u8(8),
            Type::F32 => state.write_u8(9),
            Type::F64 => state.write_u8(10),

            Type::Unit => state.write_u8(101),
            Type::Never => state.write_u8(102),
            Type::Error => state.write_u8(103),

            Type::Custom(v) => {
                state.write_u8(200);
                v.hash(state);
            },
        };
    }
}
