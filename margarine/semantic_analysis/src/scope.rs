use std::thread::current;

use common::{string_map::{StringIndex, StringMap}, source::SourceRange, Swap};
use sti::{define_key, keyed::KVec, packed_option::PackedOption};
use wasm::{FunctionId, LocalId, LoopId};

use crate::{funcs::FuncId, namespace::{NamespaceId, NamespaceMap}, types::{ty::Type, ty_map::TypeMap}};

define_key!(u32, pub ScopeId);

#[derive(Debug, Clone, Copy, PartialEq)]
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
    ) -> Option<Type> {
        self.over(scopes, |current| {
            if let ScopeKind::ImportType((ty, id)) = current.kind() {
                if ty == name { return Some(id) }
            }

            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let Some(ns) = namespaces.get(ns)
                else { return Some(Type::Error) };
                if let Some(val) = ns.get_type(name) { return Some(Type::Custom(val)) }
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
            if let ScopeKind::ImportFunction((func, id)) = current.kind() {
                if func == name { return Some(id) }
            }

            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let Some(ns) = namespaces.get(ns)
                else { return None };
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
                if let Some(ns) = namespaces.get(ns)?.get_mod(name) {
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
        types: &TypeMap,
    ) -> Option<NamespaceId> {
        let s = self.over(scopes, |current| {
            if let ScopeKind::ExplicitNamespace(var) = current.kind() {
                if var.name == name {
                    return Some(var.namespace)
                }
            }

            if let ScopeKind::ImplicitNamespace(ns) = current.kind() {
                let ns = namespaces.get(ns);
                if let Some(val) = ns?.get_type(name) {
                    return Some(namespaces.get_type(Type::Custom(val), types))
                }

                if let Some(val) = ns?.get_mod(name) {
                    return Some(val)
                }
            }

            if let ScopeKind::ImportType(ty) = current.kind() {
                if ty.0 == name { return Some(namespaces.get_type(ty.1, types)) }
            }
            
            if let ScopeKind::Root = current.kind() {
                let ty = match name {
                    StringMap::INT => namespaces.get_type(Type::I64, types),
                    StringMap::FLOAT => namespaces.get_type(Type::F64, types),
                    StringMap::BOOL => namespaces.get_type(Type::BOOL, types),
                    _ => return None,
                };

                return Some(ty);
            }

            None
        });

        if let Some(s) = s { return Some(s) }

        let ty = match name {
            StringMap::INT => namespaces.get_type(Type::I64, types),
            StringMap::FLOAT => namespaces.get_type(Type::F64, types),
            StringMap::BOOL => namespaces.get_type(Type::BOOL, types),
            _ => return None,
        };

        Some(ty)
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


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScopeKind {
    ExplicitNamespace(ExplicitNamespace),
    ImplicitNamespace(NamespaceId),
    FunctionDefinition(FunctionDefinitionScope),
    ImportType((StringIndex, Type)),
    ImportFunction((StringIndex, FuncId)),
    Variable(VariableScope),
    Loop(LoopScope),
    Root,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariableScope {
    pub name: StringIndex,
    pub is_mutable: bool,
    pub ty: Type,
    pub local_id: LocalId,
}

impl VariableScope {
    pub fn new(name: StringIndex, is_mutable: bool, ty: Type, local_id: LocalId) -> Self { Self { name, is_mutable, ty, local_id } }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExplicitNamespace {
    pub name: StringIndex,
    pub namespace: NamespaceId,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FunctionDefinitionScope {
    pub return_type: Type,
    pub return_source: SourceRange,
}

impl FunctionDefinitionScope {
    pub fn new(return_type: Type, return_source: SourceRange) -> Self { Self { return_type, return_source } }
}


#[derive(Debug, Clone, Copy, PartialEq)]
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
    pub const ROOT : ScopeId = ScopeId(0);

    pub fn new() -> Self { 
        let mut slf = Self { map: KVec::new() };
        slf.push(Scope::new(ScopeKind::Root, None.into()));
        slf
    }

    #[inline(always)]
    pub fn push(&mut self, scope: Scope) -> ScopeId {
        self.map.push(scope)
    }

    #[inline(always)]
    pub fn get(&self, scope_id: ScopeId) -> Scope {
        *self.map.get(scope_id).unwrap()
    }

    #[inline(always)]
    pub fn swap(&mut self, scope_id: ScopeId, scope: Scope) -> Scope {
        self.map.get_mut(scope_id).unwrap().swap(scope)
    }
}
