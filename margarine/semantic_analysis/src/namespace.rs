use common::string_map::StringIndex;
use sti::{define_key, hash::HashMap, keyed::KVec};

use crate::{types::{TypeId, Type, TypeMap}, funcs::FuncId};

define_key!(u32, pub NamespaceId);


#[derive(Debug)]
pub struct Namespace {
    types: HashMap<StringIndex, TypeId>,
    funcs: HashMap<StringIndex, FuncId>
}


impl Namespace {
    pub fn new() -> Self {
        Namespace::with_ty_and_fn_cap(0, 0)
    }


    pub fn with_fn_cap(fn_cap: usize) -> Self {
        Self::with_ty_and_fn_cap(0, fn_cap)
    }


    pub fn with_ty_cap(ty_cap: usize) -> Self {
        Self::with_ty_and_fn_cap(ty_cap, 0)
    }


    pub fn with_ty_and_fn_cap(ty_cap: usize, fn_cap: usize) -> Self {
        Namespace {
            types: HashMap::with_cap(ty_cap),
            funcs: HashMap::with_cap(fn_cap),
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
    type_to_ns: HashMap<Type, NamespaceId>,
}


impl NamespaceMap {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
            type_to_ns: HashMap::new(),
        }
    }


    #[inline(always)]
    pub fn get_type(&mut self, id: Type) -> &Namespace {
        let id = self.type_to_ns.get(&id).unwrap();
        &self.map[*id]
    }


    #[inline(always)]
    pub fn get_type_mut(&mut self, id: Type) -> &mut Namespace {
        let id = self.type_to_ns.kget_or_insert_with(id, || {
            self.map.push(Namespace::new())
        });

        &mut self.map[*id]
    }


    #[inline(always)]
    pub fn get(&self, id: NamespaceId) -> &Namespace {
        &self.map[id]
    }


    #[inline(always)]
    pub fn get_mut(&mut self, id: NamespaceId) -> &mut Namespace {
        &mut self.map[id]
    }


    #[inline(always)]
    pub fn put(&mut self, ns: Namespace) -> NamespaceId {
        self.map.push(ns)
    }


    #[inline(always)]
    pub fn map_type(&mut self, ty: Type, ns: NamespaceId) {
        self.type_to_ns.insert(ty, ns);
    }
}
