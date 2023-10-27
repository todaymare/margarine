use common::string_map::StringIndex;
use sti::{packed_option::PackedOption, define_key, keyed::KVec};

use crate::{namespace::{NamespaceId, NamespaceMap}, types::{Type, TypeId, TypeMap}};

define_key!(u32, pub ScopeId);

#[derive(Debug, Clone, Copy)]
pub struct Scope {
    parent: PackedOption<ScopeId>,
    kind: ScopeKind,
}


impl Scope {
    #[inline(always)]
    pub fn parent(self) -> PackedOption<ScopeId> { self.parent }

    #[inline(always)]
    pub fn kind(self) -> ScopeKind { self.kind }

    pub fn get_type(
        self,
        name: StringIndex,
        scopes: ScopeMap,
        namespaces: NamespaceMap,
    ) -> Option<TypeId> {
        let mut current = self;
        loop {
            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let ns = namespaces.get(ns);
                if let Some(val) = ns.find_type(name) { return Some(val) }
            }

            let Some(parent) = current.parent().to_option()
            else { break };

            current = scopes.get(parent);
        }

        None
    }
}


#[derive(Debug, Clone, Copy)]
pub enum ScopeKind {
    ExplicitNamespace(ExplicitNamespace),
    ImplicitNamespace(NamespaceId),
    FunctionDefinition(FunctionDefinitionScope),
    Variable(VariableScope),
    None,
}


#[derive(Debug, Clone, Copy)]
pub struct VariableScope {
    name: StringIndex,
    is_mutable: bool,
}


#[derive(Debug, Clone, Copy)]
pub struct ExplicitNamespace {
    name: StringIndex,
    namespace: NamespaceId,
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionDefinitionScope {
    return_type: Type,
}


#[derive(Debug)]
pub struct ScopeMap {
    map: KVec<ScopeId, Scope>,
}

impl ScopeMap {
    pub fn new() -> Self { Self { map: KVec::new() } }

    #[inline(always)]
    pub fn push(&mut self, scope: Scope) -> ScopeId {
        self.map.push(scope)
    }

    #[inline(always)]
    pub fn get(&self, scope_id: ScopeId) -> Scope {
        *self.map.get(scope_id).unwrap()
    }
}
