use std::collections::HashMap;

use common::string_map::StringIndex;
use sti::{define_key, keyed::{KVec, Key}};

use crate::{errors::Error, syms::sym_map::SymbolId};

define_key!(u32, pub NamespaceId);


#[derive(Debug)]
pub struct Namespace {
    symbols  : HashMap<StringIndex, Option<SymbolId>>,
    imported_symbols: HashMap<StringIndex, Option<SymbolId>>,
    namespaces  : HashMap<StringIndex, NamespaceId>,
    imported_namespaces: HashMap<StringIndex, NamespaceId>,
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


    pub fn get_double(&mut self, ns1: NamespaceId, ns2: NamespaceId) -> (&mut Namespace, &mut Namespace) {
        assert_ne!(ns1, ns2);
        let arr = self.map.inner_mut().get_many_mut([ns1.usize(), ns2.usize()]).unwrap();
        let ptr = arr.as_ptr();
        unsafe { (ptr.read(), ptr.add(1).read()) }
    }


    pub fn get_ns_mut(&mut self, ns: NamespaceId) -> &mut Namespace {
        &mut self.map[ns]
    }
}


impl Namespace {
    pub fn new(path: StringIndex) -> Self {
        Self {
            symbols: HashMap::new(),
            imported_symbols: HashMap::new(),
            namespaces: HashMap::new(),
            imported_namespaces: HashMap::new(),
            path,
        }
    }


    pub fn add_sym(&mut self, name: StringIndex, sym: SymbolId) {
        let old_sym = self.symbols.insert(name, Some(sym));
        if old_sym.is_some() {
            self.symbols.insert(name, None);
        }
    }


    pub fn add_err_sym(&mut self, name: StringIndex) {
        self.symbols.insert(name, None);
    }


    pub fn add_import_sym(&mut self, name: StringIndex, sym: SymbolId) {
        let old_sym = self.imported_symbols.insert(name, Some(sym));
        if old_sym.is_some() {
            self.symbols.insert(name, None);
        }
    }


    pub fn add_ns(&mut self, name: StringIndex, ns: NamespaceId) {
        let old_sym = self.namespaces.insert(name, ns);
        assert!(old_sym.is_none());
    }


    pub fn add_import_ns(&mut self, name: StringIndex, ns: NamespaceId) {
        let old_sym = self.imported_namespaces.insert(name, ns);
        assert!(old_sym.is_none());
    }


    pub fn get_sym(&self, name: StringIndex) -> Option<Result<SymbolId, Error>> {
        if let Some(v) = self.symbols.get(&name).copied() {
            if let Some(v) = v { return Some(Ok(v)) }
            return Some(Err(Error::Bypass))
        }


        if let Some(v) = self.imported_symbols.get(&name).copied() {
            if let Some(v) = v { return Some(Ok(v)) }
            return Some(Err(Error::Bypass))
        }

        None
    }


    pub fn get_ns(&self, name: StringIndex) -> Option<NamespaceId> {
        self.namespaces.get(&name).copied()
    }

    pub fn syms(&self) -> &HashMap<StringIndex, Option<SymbolId>> {
        &self.symbols
    }

    pub fn nss(&self) -> &HashMap<StringIndex, NamespaceId> {
        &self.namespaces
    }
}
