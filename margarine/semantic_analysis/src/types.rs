use std::{cell::Cell, marker::PhantomData};

use common::string_map::{StringMap, StringIndex};
use parser::DataType;
use polonius_the_crab::{polonius, polonius_return};
use sti::{define_key, vec::Vec, hash::HashMap, keyed::KVec, traits::MapIt, prelude::Arena, arena_pool::ArenaPool};

use crate::namespace::Namespace;

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
    map: KVec<TypeId, Option<TypeSymbol<'a>>>,
}


impl<'a> TypeMap<'a> {
    pub fn new() -> Self {
        let mut map = KVec::new();

        Self { map  }
    }


    ///
    /// Reserves a spot in the type map and
    /// gives a `TypeId` to that spot
    #[inline(always)]
    pub fn pending(&mut self) -> TypeId {
        self.map.push(None)
    }


    ///
    /// Initialises a pending type
    ///
    /// # Panics
    /// - If the given type is already initialised
    /// - If the `TypeId` is out of bounds
    ///
    #[inline(always)]
    pub fn initialise(&mut self, ty: TypeId, sym: TypeSymbol<'a>) {
        assert!(self.map[ty].replace(sym).is_none(), "type is already initialised")
    }


    pub fn get(&self, val: TypeId) -> &TypeSymbol<'a> {
        self.get_opt(val).unwrap()
    }


    pub fn get_opt(&self, val: TypeId) -> Option<&TypeSymbol<'a>> {
        self.map.get(val).unwrap().as_ref()
    }


    pub fn align(&self, ty: Type) -> Option<usize> {
        Some(match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Never => todo!(),
            Type::Custom(v) => self.get_opt(v)?.align,
        })
    }


    pub fn size(&self, ty: Type) -> Option<usize> {
        Some(match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => todo!(),
            Type::Unit => 1,
            Type::Never => todo!(),
            Type::Custom(v) => self.get_opt(v)?.size,
        })
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


#[derive(Debug)]
pub struct TypeBuilder<'storage> {
    types: HashMap<TypeId, PartialType<'storage>>,
    storage: &'storage Arena,
}


#[derive(Debug)]
enum PartialType<'a> {
    Uninit {
        name: StringIndex,
        fields: Option<&'a mut [PartialField]>,
    },

    Processing,
}


#[derive(Debug)]
pub struct PartialField {
    name: StringIndex,
    ty: Type,
    offset: Option<usize>,
}

impl PartialField {
    pub fn new(name: StringIndex, ty: Type) -> Self { Self { name, ty, offset: None } }
}


impl<'storage> TypeBuilder<'storage> {
    pub fn new(storage: &'storage Arena) -> Self {
        Self {
            types: HashMap::new(),
            storage,
        }
    }


    pub fn add_ty(&mut self, ty: TypeId, name: StringIndex) {
        self.types.insert(ty, PartialType::Uninit { name, fields: None });
    }


    pub fn add_fields(&mut self, ty: TypeId, new_fields: &'storage mut [PartialField]) {
        let PartialType::Uninit { fields, name } = self.types.get_mut(&ty).unwrap()
        else { panic!() };

        assert!(fields.is_none());
        fields.replace(new_fields);
    }


    pub fn finalise<'a>(mut self, out: &'a Arena, map: &mut TypeMap<'a>) {
        let pool = ArenaPool::tls_get_temp();
        let mut vec = Vec::with_cap_in(&*pool, self.types.len());
        self.types.iter().for_each(|x| vec.push(*x.0));

        for ty in &vec {
            self.resolve_type(out, map, *ty);
        }

    }


    pub fn alloc(&self) -> &'storage Arena {
        self.storage
    }
}


impl<'storage> TypeBuilder<'storage> {
    fn resolve_type<'a, 'b>(&mut self, out: &'a Arena, mut map: &'b mut TypeMap<'a>, ty: TypeId) -> &'b TypeSymbol<'a> {
        polonius! {
            |map| -> &'polonius TypeSymbol<'a> {
                if let Some(v) = map.map.get(ty).unwrap().as_ref() {
                    polonius_return!(v);
                }
            }
        }

        let partial_ty = self.types.insert(ty, PartialType::Processing).unwrap();
        let (fields, name) = match partial_ty {
            PartialType::Uninit { fields, name } => (fields.expect("fields are not initialised"), name),

            PartialType::Processing => panic!("cyclic type"),
        };
        println!("{:#?}", self.types);

        let align = fields.map_it(|f| self.align(out, map, f.ty)).max().unwrap_or(1);

        let mut cursor = 0;
        let mut new_fields = Vec::with_cap_in(out, fields.len());

        for field in fields.iter_mut() {
            let align = self.align(out, map, field.ty);
            cursor = sti::num::ceil_to_multiple_pow2(cursor, align);

            let offset = cursor;
            cursor += self.size(out, map, field.ty);

            new_fields.push(Field::new(field.name, field.ty, offset));
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);

        let symbol = TypeSymbol {
            display_name: name,
            fields: new_fields.leak(),
            align,
            size,
        };

        map.initialise(ty, symbol);
        map.get(ty)
    }


    fn align<'a>(&mut self, out: &'a Arena, map: &mut TypeMap<'a>, ty: Type) -> usize {
        if let Some(v) = map.align(ty) {
            return v
        }

        let ty_id = match ty {
            Type::Custom(v) => v,
            _ => unreachable!()
        };

        self.resolve_type(out, map, ty_id).align
    }


    fn size<'a>(&mut self, out: &'a Arena, map: &mut TypeMap<'a>, ty: Type) -> usize {
        if let Some(v) = map.size(ty) {
            return v
        }

        let ty_id = match ty {
            Type::Custom(v) => v,
            _ => unreachable!()
        };

        self.resolve_type(out, map, ty_id).size
    }

}


#[cfg(test)]
mod tests {
    use common::string_map::StringMap;
    use sti::prelude::Arena;

    use super::{TypeMap, Type, Field};

/*
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
*/
}
