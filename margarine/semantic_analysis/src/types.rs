use std::cell::Cell;

use common::string_map::{StringMap, StringIndex};
use sti::{define_key, vec::Vec, hash::HashMap, keyed::KVec, traits::MapIt, prelude::Arena};

define_key!(u32, pub TypeId);

#[derive(Debug, Clone, Copy)]
pub enum Type {
    Int,
    UInt,
    Float,
    Any,
    Unit,
    Never,
    Custom(TypeId),
}


impl Type {
    pub fn display<'a>(
        self,
        string_map: &'a StringMap<'a>,
        types: &TypeMap,
    ) -> &'a str {
        match self {
            Type::Int => "int",
            Type::UInt => "uint",
            Type::Float => "float",
            Type::Any => "any",
            Type::Unit => "unit",
            Type::Never => "never",
            Type::Custom(t) => string_map.get(types.get(t).display_name),
        }
    }

}


#[derive(Debug)]
pub struct TypeSymbol<'a> {
    display_name: StringIndex, 
    fields: &'a [Field],

    align: usize,
    size : usize,
}


#[derive(Debug)]
pub struct TypeMap<'a> {
    map: KVec<TypeId, TypeSymbol<'a>>,
}


impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        Self { map: KVec::new() }
    }


    #[inline(always)]
    pub fn put(&mut self, ty: TypeSymbol<'a>) -> TypeId {
        self.map.push(ty)
    }


    /*
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
            Type::Never => (0, 0),
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
    */


    pub fn get(&self, val: TypeId) -> &TypeSymbol {
        self.map.get(val).unwrap()
    }


    pub fn align(&self, ty: Type) -> usize {
        match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Never => todo!(),
            Type::Custom(v) => self.get(v).align,
        }
    }


    pub fn size(&self, ty: Type) -> usize {
        match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Never => todo!(),
            Type::Custom(v) => self.get(v).size,
        }
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Field {
    name: StringIndex,
    ty: Type,
    offset: usize,
}

impl Field {
    pub fn new(name: StringIndex, ty: Type, offset: usize) -> Self { Self { name, ty, offset } }
}


struct TypeBuilder<'a> {
    mappings: HashMap<StringIndex, Option<(&'a [(StringIndex, BuilderType)], Cell<Option<TypeId>>)>>,
}


impl TypeBuilder<'_> {
    pub fn finish(mut self, map: &mut TypeMap) {
        for (name, _) in self.mappings.iter() {
            self.resolve_type(*name, map)
        }
    }
    /*
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
        
    */

    pub fn resolve_type(&mut self, name: StringIndex, map: &mut TypeMap) {
        let Some((fields, _)) = self.mappings.get(&name).unwrap()
        else { panic!("the type is not fully initialised") };

        let align = fields.map_it(|f| self.align(f.1, map)).max().unwrap_or(1);

        let mut cursor = 0;
        let mut new_fields = Vec::with_cap_in(map.map.inner().alloc(), fields.len());
        for f in fields.iter() {
            cursor = sti::num::ceil_to_multiple_pow2(cursor, self.align(f.1, map));
            let offset = cursor;
            cursor += self.size(f.1, map);
            
            let ty = match f.1 {
                BuilderType::Type(v) => v,
                BuilderType::SelfRef(v) => Type::Custom(self.mappings.get(&v).unwrap().unwrap().1.unwrap()),
            };

            new_fields.push(Field::new(name, ty, offset))
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);

        let val = self.mappings.get_mut(&name).unwrap();
        val.unwrap().1.replace(map.put(TypeSymbol {
            display_name: name,
            fields: new_fields.leak(),
            align,
            size,  
        }));

    }


    pub fn align(&mut self, ty: BuilderType, map: &mut TypeMap) -> usize {
        let ty = match ty {
            BuilderType::Type(v) => v,
            BuilderType::SelfRef(v) => Type::Custom(match self.mappings.get(&v).unwrap().unwrap().1 {
                Some(v) => v,
                None => {
                    self.resolve_type(v, map);
                    self.mappings.get(&v).unwrap().unwrap().1.unwrap()
                }
            }),
        };

        map.align(ty)
    }


    pub fn size(&mut self, ty: BuilderType, map: &mut TypeMap) -> usize {
        let ty = match ty {
            BuilderType::Type(v) => v,
            BuilderType::SelfRef(v) => Type::Custom(match self.mappings.get(&v).unwrap().unwrap().1 {
                Some(v) => v,
                None => {
                    self.resolve_type(v, map);
                    self.mappings.get(&v).unwrap().unwrap().1.unwrap()
                }
            }),
        };

        map.size(ty)
   }
}


#[derive(Clone, Copy)]
pub enum BuilderType {
    Type(Type),
    SelfRef(StringIndex),
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
