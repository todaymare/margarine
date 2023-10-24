use common::string_map::StringIndex;
use sti::{packed_option::PackedOption, define_key};

use crate::{namespace::NamespaceId, types::Type};

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
