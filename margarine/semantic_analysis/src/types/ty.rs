use common::string_map::{StringIndex, StringMap};
use wasm::WasmType;

use crate::types::ty_map::TypeId;

use super::{ty_map::TypeMap, ty_sym::{TypeEnumKind, TypeKind, TypeStructStatus}};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    I64,
    I32,
    F64,

    Any,
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
    pub fn type_id(self) -> TypeId {
        match self {
            Type::I64 => TypeId::I64,
            Type::I32 => TypeId::I32,
            Type::F64 => TypeId::F64,
            Type::Any => TypeId::ANY,
            Type::Unit => TypeId::UNIT,
            Type::Never => TypeId::NEVER,
            Type::Error => TypeId::ERROR,
            Type::Custom(v) => v,
        }
    }

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
            Type::I64    => "int",
            Type::I32    => "i32",
            Type::F64    => "float",
            Type::Any    => "any",
            Type::Unit   => "unit",
            Type::Never  => "never",
            Type::Error  => "error",

            Type::Custom(id) => {
                let ty = types.get(id);
                let display_name = ty.path();
                string_map.get(display_name)
            },
        }
    }


    pub fn path (
        self,
        types: &TypeMap,
    ) -> StringIndex {
        match self {
            Type::I64 => StringMap::INT,
            Type::I32 => unimplemented!(),
            Type::F64 => StringMap::FLOAT,
            Type::Any => StringMap::ANY,
            Type::Unit => StringMap::UNIT,
            Type::Never => unimplemented!(),
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
            | (Type::I64, Type::I64) 
            | (Type::I32, Type::I32) 
            | (Type::F64, Type::F64) 
            | (Type::Any, Type::Any) 
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
        self.eq_sem(Type::I64)
        || self.eq_sem(Type::F64)
    }


    pub fn size(self, ty_map: &TypeMap) -> usize {
        match self {
            Type::I64 => 8, 
            Type::I32 => 8,
            Type::F64 => 8,
            Type::Any => 16,
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
            Type::I64 => WasmType::I64, 
            Type::I32 => WasmType::I32,
            Type::F64 => WasmType::F64,
            Type::Any => todo!(),
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
                }
            },
        }
    }
}


impl core::hash::Hash for Type {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        match self {
            Type::I64 => state.write_u8(0),
            Type::I32 => state.write_u8(1),
            Type::F64 => state.write_u8(2),

            Type::Any => state.write_u8(3),
            Type::Unit => state.write_u8(4),
            Type::Never => state.write_u8(5),
            Type::Error => state.write_u8(6),

            Type::Custom(v) => {
                state.write_u8(7);
                v.hash(state);
            },
        };
    }
}
