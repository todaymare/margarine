use common::string_map::StringMap;
use wasm::WasmType;

use crate::types::ty_map::TypeId;

use super::ty_map::TypeMap;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    Int,
    Float,

    Any,
    Unit,
    Never,
    Error,

    Custom(TypeId),
}


impl Type {
    pub const BOOL : Type = Type::Custom(TypeId::BOOL);
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
            Type::Int    => "int",
            Type::Float  => "float",
            Type::Any    => "any",
            Type::Unit   => "unit",
            Type::Never  => "never",
            Type::Error  => "error",

            Type::Custom(id) => {
                let ty = types.get(id);
                let display_name = ty.display_name();
                string_map.get(display_name)
            },
        }
    }


    ///
    /// Checks for literal equality
    /// Same thing as the result of deriving PartialEq
    ///
    pub fn eq_lit(self, oth: Type) -> bool {
        match (self, oth) {
            | (Type::Int, Type::Int) 
            | (Type::Float, Type::Float) 
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
        self.eq_sem(Type::Int)
        || self.eq_sem(Type::Float)
    }


    pub fn to_wasm_ty(self, ty_map: &TypeMap) -> WasmType {
        match self {
            Type::Int => WasmType::I64, 
            Type::Float => WasmType::F64,
            Type::Any => todo!(),
            Type::Unit => WasmType::I64,
            Type::Never => todo!(),
            Type::Error => todo!(),

            Type::Custom(v) => {
                let ty = ty_map.get(v);
                match ty.kind() {
                    super::ty_sym::TypeSymbolKind::Struct(_)
                        => WasmType::Ptr(ty.size()),

                    super::ty_sym::TypeSymbolKind::Enum(v) => {
                        match v {
                            super::ty_sym::TypeEnum::TaggedUnion(_) => WasmType::Ptr(ty.size()),
                            super::ty_sym::TypeEnum::Tag(_) => WasmType::I64,
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
            Type::Int => state.write_u8(0),
            Type::Float => state.write_u8(1),

            Type::Any => state.write_u8(2),
            Type::Unit => state.write_u8(3),
            Type::Never => state.write_u8(4),
            Type::Error => state.write_u8(5),

            Type::Custom(v) => {
                state.write_u8(6);
                v.hash(state);
            },
        };
    }
}
