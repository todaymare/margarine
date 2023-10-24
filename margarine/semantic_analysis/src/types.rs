use common::string_map::{StringMap, StringIndex};
use sti::{define_key, hash::HashMap, keyed::KVec, traits::MapIt};

define_key!(u32, pub TypeId);

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
    UInt,
    Float,
    Any,
    Unit,
    Custom(TypeId),
}


impl Type {
    pub fn display<'a>(
        self,
        string_map: &'a StringMap<'a>,
        types: TypeMap,
    ) -> &'a str {
        match self {
            Type::Int => "int",
            Type::UInt => "uint",
            Type::Float => "float",
            Type::Any => "any",
            Type::Unit => "unit",
            Type::Custom(t) => string_map.get(types.map.get(t).unwrap().display_name),
        }
    }

}


pub struct TypeSymbol<'a> {
    display_name: StringIndex, 
    fields: &'a [Field],

    align: usize,
    size : usize,
}


pub struct TypeMap<'a> {
    map: KVec<TypeId, TypeSymbol<'a>>,
}


impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        Self { map: KVec::new() }
    }


    pub fn insert(&mut self, display_name: StringIndex, fields: &'a mut [Field]) -> TypeId {
        let align = fields.map_it(|f| self.align(f.ty)).max().unwrap_or(1);

        let mut cursor = 0;
        for f in fields.iter_mut() {
            cursor = sti::num::ceil_to_multiple_pow2(cursor, self.align(f.ty));
            f.offset = cursor;
            cursor += self.size(f.ty);
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);

        self.map.push(TypeSymbol {
            display_name,
            fields,
            align,
            size,
        })
    }


    pub fn align(&self, ty: Type) -> usize {
        match ty {
            Type::Int => core::mem::align_of::<i64>(),
            Type::UInt => core::mem::align_of::<u64>(),
            Type::Float => core::mem::align_of::<f64>(),
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Custom(v) => self.get(v).unwrap().align,
        }
    }


    pub fn size(&self, ty: Type) -> usize {
        match ty {
            Type::Int => core::mem::size_of::<i64>(),
            Type::UInt => core::mem::size_of::<u64>(),
            Type::Float => core::mem::size_of::<f64>(),
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Custom(v) => self.get(v).unwrap().size,
        }
    }


    pub fn get(&self, val: TypeId) -> Option<&TypeSymbol> {
        self.map.get(val)
    }
}


pub struct Field {
    name: StringIndex,
    ty: Type,
    offset: usize,
}

impl Field {
    pub fn new(name: StringIndex, ty: Type) -> Self { Self { name, ty, offset: 0 } }
}


#[cfg(test)]
mod tests {
    use common::string_map::StringMap;
    use sti::prelude::Arena;

    use super::{TypeMap, Type, Field};

    #[test]
    fn structure_c_repr() {
        let arena = Arena::new();
        let mut string_map = StringMap::new(&arena);
        let test = string_map.insert("test");
        let mut binding = [Field::new(test, Type::Int), Field::new(test, Type::Float)];
        let mut type_map = TypeMap::new();
        let id = type_map.insert(test, &mut binding);
        let ty = type_map.get(id).unwrap();

        #[repr(C)]
        struct TestAgainst { _v1: i64, _v2: f64 }

        assert_eq!(ty.align, core::mem::align_of::<TestAgainst>());
        assert_eq!(ty.size , core::mem::size_of::<TestAgainst>());
    }
}
