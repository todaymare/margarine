use common::{string_map::{StringIndex, StringMap}, source::SourceRange};
use sti::{packed_option::PackedOption, define_key, keyed::KVec};
use wasm::{LocalId, LoopId};

use crate::{namespace::{NamespaceId, NamespaceMap}, types::{ty::Type, ty_map::TypeId}, funcs::FuncId};

define_key!(u32, pub ScopeId);

#[derive(Debug, Clone, Copy)]
pub struct Scope {
    parent: PackedOption<ScopeId>,
    kind: ScopeKind,
}


impl Scope {
    #[inline(always)]
    pub fn new(kind: ScopeKind, parent: PackedOption<ScopeId>) -> Self {
        Self {
            parent,
            kind,
        }
    }

    #[inline(always)]
    pub fn parent(self) -> PackedOption<ScopeId> { self.parent }

    #[inline(always)]
    pub fn kind(self) -> ScopeKind { self.kind }

    pub fn get_type(
        self,
        name: StringIndex,
        scopes: &ScopeMap,
        namespaces: &NamespaceMap,
    ) -> Option<TypeId> {
        self.over(scopes, |current| {
            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let ns = namespaces.get(ns);
                if let Some(val) = ns.get_type(name) { return Some(val) }
            }

            None
        })
    }


    pub fn get_func(
        self,
        name: StringIndex,
        scopes: &ScopeMap,
        namespaces: &NamespaceMap,
    ) -> Option<FuncId> {
        self.over(scopes, |current| {
            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let ns = namespaces.get(ns);
                if let Some(val) = ns.get_func(name) { return Some(val) }
            }

            None
        })
    }


    pub fn get_var(
        self,
        name: StringIndex,
        scopes: &ScopeMap,
    ) -> Option<VariableScope> {
        self.over(scopes, |current| {
            if let ScopeKind::Variable(var) = current.kind() {
                if var.name == name {
                    return Some(var)
                }
            }

            None
        })
    }


    pub fn get_mod(
        self,
        name: StringIndex,
        scopes: &ScopeMap,
        namespaces: &NamespaceMap,
    ) -> Option<NamespaceId> {
        self.over(scopes, |current| {
            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                if let Some(ns) = namespaces.get(ns).get_mod(name) {
                    return Some(ns);
                }
            }

            None
        })
    }

    pub fn get_ns(
        self,
        name: StringIndex,
        scopes: &ScopeMap,
        namespaces: &mut NamespaceMap,
    ) -> Option<NamespaceId> {
        self.over(scopes, |current| {
            if let ScopeKind::ExplicitNamespace(var) = current.kind() {
                if var.name == name {
                    return Some(var.namespace)
                }
            }

            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let ns = namespaces.get(ns);
                if let Some(val) = ns.get_type(name) {
                    return Some(namespaces.get_type(Type::Custom(val)))
                }

                if let Some(val) = ns.get_mod(name) {
                    return Some(val)
                }
            }

            
            if let ScopeKind::Root = current.kind() {
                let ty = match name {
                    StringMap::INT => namespaces.get_type(Type::I64),
                    StringMap::FLOAT => namespaces.get_type(Type::F64),
                    StringMap::BOOL => namespaces.get_type(Type::BOOL),
                    StringMap::ANY => namespaces.get_type(Type::Any),
                    _ => return None,
                };

                return Some(ty);
            }

            None
        })
    }


    pub fn get_func_def(
        self,
        scopes: &ScopeMap,
    ) -> Option<FunctionDefinitionScope> {
        self.over(scopes, |current| {
            if let ScopeKind::FunctionDefinition(funcdef) = current.kind() {
                return Some(funcdef)
            }

            None
        })
    }


    pub fn get_loop(
        self,
        scopes: &ScopeMap,
    ) -> Option<LoopScope> {
        self.over(scopes, |current| {
            if let ScopeKind::Loop(funcdef) = current.kind() {
                return Some(funcdef)
            }

            None
        })
    }


    ///
    /// Iterates over the current scope and all of its
    /// parents, calling `func` on each of them. If `func`
    /// returns `Some` it will return that value, short-circuiting.
    /// 
    /// If `func` does not return `Some` and there are no more parents
    /// left, this function will return `None`
    ///
    pub fn over<T>(
        self,
        scopes: &ScopeMap,
        mut func: impl FnMut(Self) -> Option<T>
    ) -> Option<T> {
        let mut current = self;
        loop {
            if let Some(val) = func(current) {
                return Some(val)
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
    Loop(LoopScope),
    Root,
}


#[derive(Debug, Clone, Copy)]
pub struct VariableScope {
    pub name: StringIndex,
    pub is_mutable: bool,
    pub ty: Type,
    pub local_id: LocalId,
}

impl VariableScope {
    pub fn new(name: StringIndex, is_mutable: bool, ty: Type, local_id: LocalId) -> Self { Self { name, is_mutable, ty, local_id } }

}


#[derive(Debug, Clone, Copy)]
pub struct ExplicitNamespace {
    name: StringIndex,
    pub namespace: NamespaceId,
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionDefinitionScope {
    pub return_type: Type,
    pub return_source: SourceRange,
}

impl FunctionDefinitionScope {
    pub fn new(return_type: Type, return_source: SourceRange) -> Self { Self { return_type, return_source } }
}


#[derive(Debug, Clone, Copy)]
pub struct LoopScope {
    pub loop_id: LoopId,
}

impl LoopScope {
    pub fn new(loop_id: LoopId) -> Self { Self { loop_id } }
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
