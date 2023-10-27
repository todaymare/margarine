use std::cell::Cell;

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
            Type::Custom(t) => string_map.get(types.get(t).display_name),
        }
    }

}


pub struct TypeSymbol<'a> {
    display_name: StringIndex, 
    fields: &'a [Field],

    align_and_size: Cell<Option<(usize, usize)>>,
}


pub struct TypeMap<'a> {
    map: KVec<TypeId, Option<TypeSymbol<'a>>>,
}


impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        Self { map: KVec::new() }
    }


    /*
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
    */


    #[inline(always)]
    pub fn uninit(&mut self) -> TypeId {
        self.map.push(None)
    }


    pub fn initialise(&mut self, id: TypeId, display_name: StringIndex, fields: &'a mut [Field]) {
        let val = self.map.get_mut(id).unwrap();
        assert!(val.is_none(), "value is already initialised");

        val.replace(TypeSymbol {
            display_name,
            fields,
            align_and_size: None.into(),
        });
    }


    pub fn calc_size(&self, id: TypeId) -> (usize, usize) {
        let val = self.get(id);
        let fields = &*val.fields;

        let align = fields.map_it(|f| self.align(f.ty)).max().unwrap_or(1);

        let mut cursor = 0;
        for f in fields.iter() {
            cursor = sti::num::ceil_to_multiple_pow2(cursor, self.align(f.ty));
            f.offset.replace(cursor);
            cursor += self.size(f.ty);
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);

        let val = self.map.get(id).unwrap().as_ref().unwrap();
        val.align_and_size.replace(Some((align, size)).into());
        (align, size)
    }


    pub fn align_and_size(&self, ty: Type) -> (usize, usize) {
        match ty {
            Type::Int => (core::mem::align_of::<i64>(), core::mem::size_of::<i64>()),
            Type::UInt => (core::mem::align_of::<u64>(), core::mem::size_of::<u64>()),
            Type::Float => (core::mem::align_of::<f64>(), core::mem::size_of::<f64>()),
            Type::Any => todo!(),
            Type::Unit => (1, 1),
            Type::Custom(v) => {
                let val = self.get(v).align_and_size.get();
                match val {
                    Some(v) => v,
                    None => self.calc_size(v),
                }
            },
        }
    }


    pub fn align(&self, ty: Type) -> usize {
        self.align_and_size(ty).0
    }


    pub fn size(&self, ty: Type) -> usize {
        self.align_and_size(ty).1
    }


    pub fn get(&self, val: TypeId) -> &TypeSymbol {
        self.map.get(val).map(|v| v.as_ref().expect("value is not initialised")).unwrap()
    }
}


pub struct Field {
    name: StringIndex,
    ty: Type,
    offset: Cell<usize>,
}

impl Field {
    pub fn new(name: StringIndex, ty: Type) -> Self { Self { name, ty, offset: 0.into() } }
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
        let id = type_map.uninit();
        type_map.initialise(id, test, &mut binding);

        #[repr(C)]
        struct TestAgainst { _v1: i64, _v2: f64 }

        assert_eq!(type_map.calc_size(id), (core::mem::align_of::<TestAgainst>(), core::mem::size_of::<TestAgainst>()));
    }
}
