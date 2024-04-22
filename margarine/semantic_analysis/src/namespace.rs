use std::collections::HashMap;

use common::string_map::StringIndex;
use sti::{define_key, keyed::KVec};

use crate::{funcs::FunctionSymbolId, types::TypeSymbolId};

define_key!(u32, pub NamespaceId);


#[derive(Debug)]
pub struct Namespace {
    ty_symbols  : HashMap<StringIndex, TypeSymbolId>,
    func_symbols: HashMap<StringIndex, FunctionSymbolId>,
    namespaces  : HashMap<StringIndex, NamespaceId>,
    pub path: StringIndex,
}


#[derive(Debug)]
pub struct NamespaceMap {
    map: KVec<NamespaceId, Namespace>,
}


impl NamespaceMap {
    pub fn new() -> Self { Self { map: KVec::new() } }

    pub fn push(&mut self, ns: Namespace) -> NamespaceId {
        self.map.push(ns)
    }


    pub fn get_ns(&self, ns: NamespaceId) -> &Namespace {
        &self.map[ns]
    }


    pub fn get_ns_mut(&mut self, ns: NamespaceId) -> &mut Namespace {
        &mut self.map[ns]
    }
}


impl Namespace {
    pub fn new(path: StringIndex) -> Self {
        Self {
            ty_symbols: HashMap::new(),
            func_symbols: HashMap::new(),
            namespaces: HashMap::new(),
            path,
        }
    }
    pub fn add_sym(&mut self, name: StringIndex, sym: TypeSymbolId) {
        let old_sym = self.ty_symbols.insert(name, sym);
        assert!(old_sym.is_none());
    }


    pub fn add_func(&mut self, name: StringIndex, sym: FunctionSymbolId) {
        let old_sym = self.func_symbols.insert(name, sym);
        assert!(old_sym.is_none());
    }


    pub fn add_ns(&mut self, name: StringIndex, ns: NamespaceId) {
        let old_sym = self.namespaces.insert(name, ns);
        assert!(old_sym.is_none());
    }


    pub fn get_ty_sym(&self, name: StringIndex) -> Option<TypeSymbolId> {
        self.ty_symbols.get(&name).copied()
    }


    pub fn get_func(&self, name: StringIndex) -> Option<FunctionSymbolId> {
        self.func_symbols.get(&name).copied()
    }


    pub fn get_ns(&self, name: StringIndex) -> Option<NamespaceId> {
        self.namespaces.get(&name).copied()
    }
}
