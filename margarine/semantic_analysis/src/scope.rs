use common::{source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use sti::{define_key, vec::KVec};

use crate::{errors::Error, namespace::{NamespaceId, NamespaceMap}, syms::{sym_map::{ClosureId, Generic, SymbolId, SymbolMap}, ty::Sym}};

define_key!(pub ScopeId(u32));


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Scope<'me> {
    parent: Option<ScopeId>,
    kind  : ScopeKind<'me>,
}


#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub enum ScopeKind<'me> {
    ImplicitNamespace(NamespaceId),
    Alias(StringIndex, Generic<'me>),
    VariableScope(VariableScope),
    Generics(GenericsScope<'me>),
    Loop,
    Function(FunctionScope),
    Closure(ClosureId),
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
    pub fn new(parent: impl Into<Option<ScopeId>>, kind: ScopeKind<'me>) -> Self { Self { parent: parent.into(), kind } }


    pub fn find_self(self, scope_map: &ScopeMap<'me>) -> Option<Generic<'me>> {
        self.over(scope_map, |scope| {
            if let ScopeKind::Alias(sym_name, sym) = scope.kind {
                if sym_name == StringMap::SELF_TY { return Some(sym) }
            }

            None
        })
    }


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


    pub fn find_var(
        self,
        name: StringIndex,
        scope_map: &ScopeMap,
        namespaces: &NamespaceMap,
        symbols: &mut SymbolMap
    ) -> Option<Result<VariableScope, Result<SymbolId, Error>>> {

        self.over(scope_map, |scope| {
            if let ScopeKind::VariableScope(v) = scope.kind {
                if v.name() != name { return None }
                self.over(scope_map, |scope| {
                    if let ScopeKind::VariableScope(v) = scope.kind {
                        if v.name() == name { return Some(()) }
                    }
                    
                    if let ScopeKind::Closure(closure) = scope.kind() {
                        symbols.insert_closure_capture(closure, name, v.ty);
                    }

                    None
                });

                return Some(Ok(v))
            }

            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_sym(name) {
                    return Some(Err(ty))
                }
            }

            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0 == name) {
                    return Some(Err(Ok(ty.1.sym(symbols).expect("please work"))))
                }
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



    pub fn over_gens(
        self,
        scope_map: &ScopeMap<'me>,
        mut func: impl FnMut(GenericsScope) 
    ) {
        self.over(scope_map, |scope| {
            if let ScopeKind::Generics(generics_scope) = scope.kind {
                func(generics_scope);
            }

            Some(())
        });

        
    }


    fn over<T>(self, scope_map: &ScopeMap<'me>, mut func: impl FnMut(Scope<'me>) -> Option<T>) -> Option<T> {
        let mut this = Some(self);
        while let Some(scope) = this {
            if let Some(val) = func(scope) { return Some(val) }

            this = scope.parent
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


#[derive(Debug, ImmutableData, Clone, Copy)]
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
