use std::collections::HashMap;

use common::{source::SourceRange, string_map::StringIndex};
use errors::{ErrorId, SemaError};
use sti::{define_key, vec::{KVec}};

use crate::{errors::Error, syms::sym_map::SymbolId};

define_key!(pub NamespaceId(u32));


#[derive(Debug)]
pub struct Namespace {
    symbols: HashMap<StringIndex, Result<SymbolId, Error>>,
    namespaces: HashMap<StringIndex, NamespaceId>,
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
        let arr = self.map.as_mut_slice().get_disjoint_mut([ns1.usize(), ns2.usize()]).unwrap();
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
            namespaces: HashMap::new(),
            path,
        }
    }


    pub fn set_err_sym(&mut self, name: StringIndex, err: Error) {
        self.symbols.insert(name, Err(err));
    }


    pub fn add_sym(&mut self, source: SourceRange, name: StringIndex, sym: SymbolId) -> Result<(), Error>{
        let old_sym = self.symbols.insert(name, Ok(sym));
        if old_sym.is_some() {
            let id = Error::NameIsAlreadyDefined { source, name };
            self.symbols.insert(name, Err(id.clone()));
            return Err(id)
        }

        Ok(())
    }


    pub fn add_ns(&mut self, name: StringIndex, ns: NamespaceId) {
        let old_sym = self.namespaces.insert(name, ns);
        assert!(old_sym.is_none());
    }

    pub fn get_sym(&self, name: StringIndex) -> Option<Result<SymbolId, Error>> {
        if let Some(v) = self.symbols.get(&name).cloned() {
            return Some(v)
        }

        None
    }


    pub fn get_ns(&self, name: StringIndex) -> Option<NamespaceId> {
        self.namespaces.get(&name).copied()
    }

    pub fn syms(&self) -> &HashMap<StringIndex, Result<SymbolId, Error>> {
        &self.symbols
    }

    pub fn nss(&self) -> &HashMap<StringIndex, NamespaceId> {
        &self.namespaces
    }
}


impl NamespaceId {
    pub fn usize(self) -> usize { self.0 as usize }
}
