use std::collections::HashMap;

use common::{source::SourceRange, string_map::StringIndex};
use llvm_api::builder::Loop;
use sti::{define_key, keyed::KVec, packed_option::PackedOption};

use crate::{funcs::FunctionSymbolId, namespace::{NamespaceId, NamespaceMap}, types::{SymbolMap, Type, SymbolId}};

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


    pub fn find_ty(self, name: StringIndex, scope_map: &ScopeMap, symbols: &mut SymbolMap, namespaces: &NamespaceMap) -> Option<SymbolId> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_ty_sym(name) {
                    return Some(ty)
                }
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0 == name) {
                    return Some(ty.1.sym(symbols).expect("please work"))
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


    pub fn find_curr_func(self, scope_map: &ScopeMap<'me>) -> Option<FunctionScope> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Function(l) = scope.kind {
                return Some(l)
            }

            None
        })
    }

    fn over<T>(self, scope_map: &ScopeMap<'me>, mut func: impl FnMut(Scope<'me>) -> Option<T>) -> Option<T> {
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
}

impl VariableScope {
    pub fn new(name: StringIndex, ty: Type, is_mut: bool) -> Self { Self { name, ty, is_mut } }

    #[inline(always)]
    pub fn is_mut(&self) -> bool { self.is_mut }

    #[inline(always)]
    pub fn ty(&self) -> Type { self.ty }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
}


#[derive(Debug, Clone, Copy)]
pub struct GenericsScope<'me> {
    generics: &'me [(StringIndex, Type)],
}


impl<'me> GenericsScope<'me> {
    pub fn new(generics: &'me [(StringIndex, Type)]) -> Self { Self { generics } }
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionScope {
    pub ret: Type,
    pub ret_source: SourceRange,
        
}


impl FunctionScope {
    pub fn new(ret: Type, ret_source: SourceRange) -> Self { Self { ret, ret_source } }

}
