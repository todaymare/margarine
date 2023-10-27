use common::string_map::StringIndex;
use sti::{define_key, hash::HashMap, keyed::KVec};

use crate::types::TypeId;

define_key!(u32, pub NamespaceId);


pub struct Namespace {
    types: HashMap<StringIndex, TypeId>
}


impl Namespace {
    pub fn find_type(&self, id: StringIndex) -> Option<TypeId> {
        self.types.get(&id).copied()
    }
}


pub struct NamespaceMap {
    map: KVec<NamespaceId, Namespace>,
}


impl NamespaceMap {
    #[inline(always)]
    pub fn get(&self, id: NamespaceId) -> &Namespace {
        self.map.get(id).unwrap()
    }
}
