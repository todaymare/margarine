use sti::{define_key, keyed::KVec};

use super::ty_sym::TypeSymbol;

define_key!(u32, pub TypeId);


impl TypeId {
    pub const BOOL : TypeId = TypeId(0);
}


#[derive(Debug)]
pub struct TypeMap<'out> {
    map: KVec<TypeId, Option<TypeSymbol<'out>>>
}


impl<'out> TypeMap<'out> {
    pub fn new() -> Self {
        Self { map: KVec::new() }
    }

    #[inline(always)]
    pub fn pending(&mut self) -> TypeId { self.map.push(None) }

    #[inline(always)]
    pub fn put(&mut self, ty_id: TypeId, sym: TypeSymbol<'out>) { 
        let old = self.map[ty_id].replace(sym);
        assert!(old.is_none(), "replaced an already initialised value");
    }

    #[inline(always)]
    pub fn get(&self, ty_id: TypeId) -> TypeSymbol<'out> {
        self.get_opt(ty_id).unwrap()
    }

    #[inline(always)]
    pub fn get_opt(&self, ty_id: TypeId) -> Option<TypeSymbol<'out>> {
        *self.map.get(ty_id).unwrap()
    }
}
