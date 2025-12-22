use common::{source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use parser::nodes::decl::DeclGeneric;
use sti::{define_key, key::Key, vec::KVec};

use crate::{errors::Error, namespace::{NamespaceId, NamespaceMap}, syms::{sym_map::{BoundedGeneric, ClosureId, Generic, SymbolId, SymbolMap}, ty::Type, SymbolKind}};

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
    ImplicitTrait(SymbolId),
    NamespaceFence,
    AliasDecl(StringIndex, Generic<'me>),
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
            if let ScopeKind::AliasDecl(sym_name, sym) = scope.kind {
                if sym_name == StringMap::SELF_TY { return Some(sym) }
            }

            None
        })
    }


    pub fn find_super(self, scope_map: &ScopeMap<'me>) -> Option<SymbolId> {
        self.over(scope_map, |scope| {
            if let ScopeKind::ImplicitTrait(sym) = scope.kind {
                return Some(sym)
            }

            None
        })
    }


    pub fn find_sym(self, name: StringIndex, scope_map: &ScopeMap, symbols: &mut SymbolMap, namespaces: &NamespaceMap) -> Option<Result<SymbolId, Error>> {
        let mut fence = false;
        let r = self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                fence = true;
            }


            if !fence 
            && let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_sym(name) {
                    return Some(ty)
                }
            }


            if let ScopeKind::AliasDecl(ident, ty) = scope.kind {
                if name == ident 
                && let Some(sym) = ty.sym() {
                    return Some(Ok(sym))
                }
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0.name() == name) {
                    return Some(Ok(ty.1.sym(symbols).expect("please work")))
                }
            }

            None
        });

        if fence { None }
        else { r }
    }


    pub fn find_gen(self, name: StringIndex, scope_map: &ScopeMap) -> Option<Type> {
        let mut fence = false;
        let r = self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                fence = true;
                return Some(Type::I64);
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0.name() == name) {
                    return Some(ty.1)
                }
            }

            None
        });

        if fence { None }
        else { r }
    }


    pub fn find_var(
        self,
        name: StringIndex,
        scope_map: &ScopeMap,
        namespaces: &NamespaceMap,
        symbols: &mut SymbolMap
    ) -> Option<Result<VariableScope, Result<SymbolId, Error>>> {

        let mut fence = false;
        self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                fence = true;
            }


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


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0.name() == name) {
                    return Some(Err(Ok(ty.1.sym(symbols).expect("please work"))))
                }
            }


            if fence && scope.parent().is_some() { return None }

            if let ScopeKind::ImplicitNamespace(ns) = scope.kind {
                let ns = namespaces.get_ns(ns);
                if let Some(ty) = ns.get_sym(name) {
                    return Some(Err(ty))
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


    pub fn over<T>(self, scope_map: &ScopeMap<'me>, mut func: impl FnMut(Scope<'me>) -> Option<T>) -> Option<T> {
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
    ty    : Type,
}

impl VariableScope {
    pub fn new(name: StringIndex, ty: Type) -> Self { Self { name, ty } }

    #[inline(always)]
    pub fn ty(&self) -> Type { self.ty }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
}


#[derive(Debug, ImmutableData, Clone, Copy)]
pub struct GenericsScope<'me> {
    generics: &'me [(BoundedGeneric<'me>, Type)],
}


impl<'me> GenericsScope<'me> {
    pub fn new(generics: &'me [(BoundedGeneric<'me>, Type)]) -> Self { Self { generics } }
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionScope {
    pub ret: Type,
    pub ret_source: SourceRange,
        
}


impl FunctionScope {
    pub fn new(ret: Type, ret_source: SourceRange) -> Self { Self { ret, ret_source } }

}
