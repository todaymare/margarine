use std::collections::HashMap;

use common::{source::SourceRange, string_map::StringIndex};
use parser::nodes::decl::Visibility;
use sti::{define_key, vec::KVec};

use crate::{errors::Error, syms::sym_map::SymbolId};

define_key!(pub NamespaceId(u32));

#[derive(Debug, Clone)]
pub struct SymbolEntry {
    result: Result<SymbolId, Error>,
    visibility: Visibility,
}

impl SymbolEntry {
    pub fn result(&self) -> Result<SymbolId, Error> { self.result.clone() }
    pub fn visibility(&self) -> Visibility { self.visibility }
}

#[derive(Debug)]
pub struct Namespace {
    symbols: HashMap<StringIndex, SymbolEntry>,
    pub path: StringIndex,
    parent: Option<NamespaceId>,
}

#[derive(Debug)]
pub struct NamespaceMap {
    map: KVec<NamespaceId, Namespace>,
}

impl NamespaceMap {
    pub fn new() -> Self { Self { map: KVec::new() } }
    pub fn push(&mut self, mut ns: Namespace, parent: Option<NamespaceId>) -> NamespaceId {
        ns.parent = parent;
        self.map.push(ns)
    }
    pub fn get_ns(&self, ns: NamespaceId) -> &Namespace { &self.map[ns] }

    pub fn get_double(&mut self, ns1: NamespaceId, ns2: NamespaceId) -> (&mut Namespace, &mut Namespace) {
        assert_ne!(ns1, ns2);
        let arr = self.map.as_mut_slice().get_disjoint_mut([ns1.usize(), ns2.usize()]).unwrap();
        let ptr = arr.as_ptr();
        unsafe { (ptr.read(), ptr.add(1).read()) }
    }

    pub fn get_ns_mut(&mut self, ns: NamespaceId) -> &mut Namespace { &mut self.map[ns] }

    pub fn get_sym(&self, owner: NamespaceId, requester: NamespaceId, name: StringIndex) -> Option<Result<SymbolId, Error>> {
        let entry = self.get_ns(owner).entry(name)?;
        if entry.visibility() == Visibility::Public || self.can_access(owner, requester) {
            Some(entry.result())
        } else {
            Some(Err(Error::PrivateSymbol { source: SourceRange::ZERO, name }))
        }
    }

    pub fn can_access(&self, owner: NamespaceId, mut requester: NamespaceId) -> bool {
        loop {
            if owner == requester { return true; }
            let Some(parent) = self.get_ns(requester).parent else { return false; };
            requester = parent;
        }
    }
}

impl Namespace {
    pub fn new(path: StringIndex) -> Self { Self { symbols: HashMap::new(), path, parent: None } }

    pub fn set_err_sym(&mut self, name: StringIndex, err: Error) {
        let visibility = self.symbols.get(&name).map(|entry| entry.visibility()).unwrap_or(Visibility::Private);
        self.symbols.insert(name, SymbolEntry { result: Err(err), visibility });
    }

    pub fn add_sym(&mut self, source: SourceRange, name: StringIndex, sym: SymbolId, visibility: Visibility) -> Result<(), Error> {
        let old_sym = self.symbols.insert(name, SymbolEntry { result: Ok(sym), visibility });
        if old_sym.is_some() {
            let id = Error::NameIsAlreadyDefined { source, name };
            self.symbols.insert(name, SymbolEntry { result: Err(id.clone()), visibility });
            return Err(id)
        }
        Ok(())
    }

    pub fn get_sym(&self, name: StringIndex) -> Option<Result<SymbolId, Error>> { self.entry(name).map(SymbolEntry::result) }
    pub fn entry(&self, name: StringIndex) -> Option<&SymbolEntry> { self.symbols.get(&name) }
    pub fn syms(&self) -> &HashMap<StringIndex, SymbolEntry> { &self.symbols }
}

impl NamespaceId { pub fn usize(self) -> usize { self.0 as usize } }
