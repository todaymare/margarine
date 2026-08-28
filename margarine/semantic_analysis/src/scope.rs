use common::{source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use sti::{define_key, vec::KVec};

use crate::{namespace::{NamespaceId, NamespaceMap, SymbolGetResult}, syms::{sym_map::{BoundedGeneric, ClosureId, Generic, SymbolId, SymbolMap}, ty::Type}};

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
    QualifiedNamespace(NamespaceId),
    QualifiedTypeNamespace(Type, Option<NamespaceId>),
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

    pub fn find_qualified_type(self, scope_map: &ScopeMap) -> Option<Type> {
        self.over(scope_map, |scope| {
            if let ScopeKind::QualifiedTypeNamespace(ty, _) = scope.kind {
                return Some(ty);
            }
            None
        })
    }


    pub fn find_sym(
        self, name: StringIndex, scope_map: &ScopeMap, 
        symbols: &mut SymbolMap, namespaces: &NamespaceMap
    ) -> SymbolGetResult {

        let Some(requester) = 
        self.over(
            scope_map, 
            |scope| match scope.kind {
                ScopeKind::ImplicitNamespace(ns) => Some(ns),
                _ => None,
            }
        )
        else { return SymbolGetResult::Undefined };

        self.find_sym_from(name, scope_map, symbols, namespaces, requester)
    }

    pub fn find_sym_from(
        self, name: StringIndex, scope_map: &ScopeMap, 
        symbols: &mut SymbolMap, namespaces: &NamespaceMap, 
        requester: NamespaceId) -> SymbolGetResult {
        let mut fence = false;
        let r = self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                fence = true;
            }

            match scope.kind {
                ScopeKind::ImplicitNamespace(ns) | ScopeKind::QualifiedNamespace(ns) => {
                    let result = namespaces.get_sym(ns, requester, name);

                    if result != SymbolGetResult::Undefined {
                        return Some(result)
                    }
                },
                ScopeKind::QualifiedTypeNamespace(ty, qualified_requester) => {
                    let Ok(sym) = ty.sym(symbols)
                    else { return None };
                    let ns = symbols.sym_ns(sym);
                    let requester = qualified_requester.unwrap_or(ns);
                    let result = namespaces.get_sym(ns, requester, name);

                    if result != SymbolGetResult::Undefined {
                        return Some(result)
                    }
                },
                _ => (),
            }


            if let ScopeKind::AliasDecl(ident, ty) = scope.kind {
                if name == ident 
                && let Some(sym) = ty.sym() {
                    return Some(SymbolGetResult::Symbol(sym))
                }
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0.name() == name) {
                    return Some(SymbolGetResult::Symbol(ty.1.sym(symbols).expect("please work")))
                }
            }

            None
        });

        if fence { SymbolGetResult::Undefined }
        else { r.unwrap_or(SymbolGetResult::Undefined) }
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


    /// Collects every enclosing generic (innermost first, first occurrence
    /// wins), stopping at a namespace fence like `find_gen` does.
    pub fn collect_generics(
        self,
        scope_map: &ScopeMap<'me>,
        out: &mut std::vec::Vec<(BoundedGeneric<'me>, Type)>,
    ) {
        self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                return Some(())
            }

            if let ScopeKind::Generics(generics_scope) = scope.kind {
                for (name, ty) in generics_scope.generics.iter() {
                    if !out.iter().any(|(n, _)| n.name() == name.name()) {
                        out.push((*name, *ty));
                    }
                }
            }

            None
        });
    }


    pub fn find_var(
        self,
        name: StringIndex,
        scope_map: &ScopeMap,
        namespaces: &NamespaceMap,
        symbols: &mut SymbolMap
    ) -> Result<(VariableScope, bool), SymbolGetResult> {

        let requester = 
        self.over(scope_map, 
        |scope| match scope.kind {
            ScopeKind::ImplicitNamespace(ns) => Some(ns),
            _ => None,
        });

        let mut fence = false;
        let mut captured = false;

        let result =
        self.over(scope_map, |scope| {
            if let ScopeKind::NamespaceFence = scope.kind {
                fence = true;
            }

            if matches!(scope.kind, ScopeKind::Closure(_)) {
                captured = true;
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

                return Some(Ok((v, captured)))
            }


            if let ScopeKind::Generics(generics_scope) = scope.kind {
                if let Some(ty) = generics_scope.generics.iter().find(|x| x.0.name() == name) {
                    return Some(Err(SymbolGetResult::Symbol(ty.1.sym(symbols).expect("please work"))))
                }
            }


            if fence && scope.parent().is_some() { return None }

            match scope.kind {
                ScopeKind::ImplicitNamespace(ns) | ScopeKind::QualifiedNamespace(ns) => {
                    let requester = requester.unwrap_or(ns);
                    let result = namespaces.get_sym(ns, requester, name);

                    if result != SymbolGetResult::Undefined {
                        return Some(Err(result));
                    }
                },
                ScopeKind::QualifiedTypeNamespace(ty, qualified_requester) => {
                    let Ok(sym) = ty.sym(symbols)
                    else { return None };
                    let ns = symbols.sym_ns(sym);
                    let requester = qualified_requester.unwrap_or(ns);
                    let result = namespaces.get_sym(ns, requester, name);

                    if result != SymbolGetResult::Undefined {
                        return Some(Err(result));
                    }
                },
                _ => (),
            }


            None
        });

        match result {
            Some(result) => result,
            None => Err(SymbolGetResult::Undefined),
        }
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
    name   : StringIndex,
    ty     : Type,
    mutable: bool,
}

impl VariableScope {
    pub fn new(name: StringIndex, ty: Type, mutable: bool) -> Self { Self { name, ty, mutable } }

    #[inline(always)]
    pub fn ty(&self) -> Type { self.ty }

    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }

    #[inline(always)]
    pub fn is_mutable(&self) -> bool { self.mutable }
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
