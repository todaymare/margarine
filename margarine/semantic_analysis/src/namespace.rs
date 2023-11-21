use common::string_map::StringIndex;
use sti::{define_key, hash::HashMap, keyed::KVec};

use crate::{types::TypeId, funcs::FuncId};

define_key!(u32, pub NamespaceId);


#[derive(Debug)]
pub struct Namespace {
    types: HashMap<StringIndex, TypeId>,
    funcs: HashMap<StringIndex, FuncId>
}


impl Namespace {
    pub fn new() -> Self {
        Namespace {
            types: HashMap::new(),
            funcs: HashMap::new(),
        }
    }
    

    pub fn add_type(&mut self, name: StringIndex, ty: TypeId) {
        let prev_value = self.types.insert(name, ty);
        assert!(prev_value.is_none());
    }


    pub fn add_func(&mut self, name: StringIndex, func: FuncId) {
        let prev_value = self.funcs.insert(name, func);
        assert!(prev_value.is_none());
    }


    pub fn get_type(&self, id: StringIndex) -> Option<TypeId> {
        self.types.get(&id).copied()
    }


    pub fn get_func(&self, id: StringIndex) -> Option<FuncId> {
        self.funcs.get(&id).copied()
    }

}


#[derive(Debug)]
pub struct NamespaceMap {
    map: KVec<NamespaceId, Namespace>,
}


impl NamespaceMap {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
        }
    }


    #[inline(always)]
    pub fn get(&self, id: NamespaceId) -> &Namespace {
        self.map.get(id).unwrap()
    }


    #[inline(always)]
    pub fn put(&mut self, ns: Namespace) -> NamespaceId {
        self.map.push(ns)
    }
}
