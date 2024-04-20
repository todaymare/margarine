use std::{collections::HashMap, thread::scope};

use common::{source::SourceRange, string_map::StringIndex};
use llvm_api::builder::{Local, Loop};
use sti::{define_key, keyed::KVec, packed_option::PackedOption};

use crate::{funcs::FunctionSymbolId, namespace::{Namespace, NamespaceId, NamespaceMap}, types::{SymbolMap, Type, TypeSymbolId}};

define_key!(u32, pub ScopeId);


#[derive(Debug, Clone, Copy)]
pub struct Scope<'me> {
    parent: PackedOption<ScopeId>,
    kind  : ScopeKind<'me>,
}


#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ScopeKind<'me> {
    ImplicitNamespace(NamespaceId),
    VariableScope(VariableScope),
    Generics(GenericsScope<'me>),
    Loop(Loop),
    Function(FunctionScope),
    Root,
}


#[derive(Debug)]
pub struct ScopeMap<'me> {
    map: KVec<ScopeId, Scope<'me>>
}


impl<'me> ScopeMap<'me> {
    pub fn new() -> Self { Self { map: KVec::new() } }

    #[inline(always)]
    pub fn push(&mut self, scope: Scope<'me>) -> ScopeId {
        self.map.push(scope)
    }

    #[inline(always)]
    pub fn get(&self, id: ScopeId) -> Scope<'me> {
        self.map[id]
    }
}


impl<'me> Scope<'me> {
    pub fn new(parent: impl Into<PackedOption<ScopeId>>, kind: ScopeKind<'me>) -> Self { Self { parent: parent.into(), kind } }

    pub fn find_func(self, name: StringIndex, scope_map: &ScopeMap, namespaces: &NamespaceMap) -> Option<FunctionSymbolId> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_func(name) {
                    return Some(ty)
                }
            }

            None
        })
    }


    pub fn find_ty(self, name: StringIndex, scope_map: &ScopeMap, symbols: &SymbolMap, namespaces: &NamespaceMap) -> Option<TypeSymbolId> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_ty_sym(name) {
                    return Some(ty)
                }
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.get(&name) {
                    return Some(symbols.get_ty_val(*ty).symbol())
                }
            }

            None
        })
    }


    pub fn find_ns(self, name: StringIndex, scope_map: &ScopeMap, namespaces: &NamespaceMap) -> Option<NamespaceId> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ns) = ns.get_ns(name) {
                    return Some(ns)
                }
            }

            None
        })
    }


    pub fn find_var(self, name: StringIndex, scope_map: &ScopeMap) -> Option<VariableScope> {
        self.over(scope_map, |scope| {
            if let ScopeKind::VariableScope(v) = scope.kind {
                if v.name() == name { return Some(v) }
            }

            None
        })
    }


    pub fn find_loop(self, scope_map: &ScopeMap) -> Option<Loop> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Loop(l) = scope.kind {
                return Some(l)
            }

            None
        })
    }


    pub fn find_curr_func(self, scope_map: &ScopeMap) -> Option<FunctionScope> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Function(l) = scope.kind {
                return Some(l)
            }

            None
        })
    }

    fn over<T>(self, scope_map: &ScopeMap, mut func: impl FnMut(Scope) -> Option<T>) -> Option<T> {
        let mut this = Some(self);
        while let Some(scope) = this {
            if let Some(val) = func(scope) { return Some(val) }

            this = scope.parent.to_option()
                .map(|x| scope_map.get(x))
        }
        None
    }
}


#[derive(Debug, Clone, Copy)]
pub struct VariableScope {
    name  : StringIndex,
    ty    : Type,
    is_mut: bool, 
    local : Local,
}

impl VariableScope {
    pub fn new(name: StringIndex, ty: Type, is_mut: bool, local: Local) -> Self { Self { name, ty, is_mut, local } }

    #[inline(always)]
    pub fn is_mut(&self) -> bool { self.is_mut }

    #[inline(always)]
    pub fn ty(&self) -> Type { self.ty }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }

    #[inline(always)]
    pub fn local(&self) -> Local { self.local }
}


#[derive(Debug, Clone, Copy)]
pub struct GenericsScope<'me> {
    generics: &'me HashMap<StringIndex, Type>,
}


impl<'me> GenericsScope<'me> {
    pub fn new(generics: &'me HashMap<StringIndex, Type>) -> Self { Self { generics } }
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionScope {
    pub ret: Type,
    pub ret_source: SourceRange,
        
}


impl FunctionScope {
    pub fn new(ret: Type, ret_source: SourceRange) -> Self { Self { ret, ret_source } }

}
