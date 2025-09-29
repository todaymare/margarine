use common::{source::SourceRange, string_map::StringIndex, ImmutableData};
use sti::{define_key, keyed::KVec, packed_option::PackedOption};

use crate::{errors::Error, namespace::{NamespaceId, NamespaceMap}, syms::{ty::Sym, sym_map::{SymbolId, SymbolMap}}};

define_key!(u32, pub ScopeId);


#[derive(Debug, Clone, Copy, ImmutableData)]
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
    Loop,
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

    pub fn find_sym(self, name: StringIndex, scope_map: &ScopeMap, symbols: &mut SymbolMap, namespaces: &NamespaceMap) -> Option<Result<SymbolId, Error>> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_sym(name) {
                    return Some(ty)
                }
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0 == name) {
                    return Some(Ok(ty.1.sym(symbols).expect("please work")))
                }
            }

            None
        })
    }


    pub fn find_gen(self, name: StringIndex, scope_map: &ScopeMap) -> Option<Sym> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0 == name) {
                    return Some(ty.1)
                }
            }

            None
        })
    }


    pub fn find_ns(self, name: StringIndex, scope_map: &ScopeMap, namespaces: &NamespaceMap, symbols: &SymbolMap) -> Option<(NamespaceId, bool)> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ns) = ns.get_ns(name) {
                    return Some((ns, false))
                }

                if let Some(ty) = ns.get_sym(name) {
                    let ns = match ty {
                        Ok(v) => (symbols.sym_ns(v), false),
                        Err(_) => (symbols.sym_ns(SymbolId::ERR), true),
                    };

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


    pub fn find_loop(self, scope_map: &ScopeMap) -> Option<()> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Loop = scope.kind {
                return Some(())
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
    ty    : Sym,
}

impl VariableScope {
    pub fn new(name: StringIndex, ty: Sym) -> Self { Self { name, ty } }

    #[inline(always)]
    pub fn ty(&self) -> Sym { self.ty }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
}


#[derive(Debug, Clone, Copy)]
pub struct GenericsScope<'me> {
    generics: &'me [(StringIndex, Sym)],
}


impl<'me> GenericsScope<'me> {
    pub fn new(generics: &'me [(StringIndex, Sym)]) -> Self { Self { generics } }
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionScope {
    pub ret: Sym,
    pub ret_source: SourceRange,
        
}


impl FunctionScope {
    pub fn new(ret: Sym, ret_source: SourceRange) -> Self { Self { ret, ret_source } }

}
