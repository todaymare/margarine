use common::string_map::StringIndex;
use sti::{define_key, keyed::KVec, hash::{HashMap, DefaultSeed}, arena::Arena};

use super::{ty_sym::TypeSymbol, ty::Type};

define_key!(u32, pub TypeId);


impl TypeId {
    pub const BOOL : TypeId = TypeId(0);
    pub const STR  : TypeId = TypeId(1);

    pub const I32  : TypeId = TypeId(u32::MAX);
    pub const I64  : TypeId = TypeId(u32::MAX - 1);
    pub const F64  : TypeId = TypeId(u32::MAX - 2);
    pub const ANY  : TypeId = TypeId(u32::MAX - 3);
    pub const UNIT : TypeId = TypeId(u32::MAX - 4);
    pub const NEVER: TypeId = TypeId(u32::MAX - 5);
    pub const ERROR: TypeId = TypeId(u32::MAX - 6);


    pub fn as_u32(self) -> u32 {
        self.0
    }
}


#[derive(Debug)]
pub struct TypeMap<'out> {
    map: KVec<TypeId, Option<TypeSymbol<'out>>>, 
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
        self.map.get(ty_id).as_ref().unwrap().as_ref().map(|x| *x)
    }
}
