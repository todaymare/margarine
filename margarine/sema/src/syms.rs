pub mod containers;
pub mod ty;
pub mod func;
pub mod sym_map;

use common::{string_map::StringIndex, ImmutableData};
use errors::ErrorId;
use crate::syms::sym_map::{BoundedGeneric, Generic};

use self::{containers::Container, func::FunctionTy};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Symbol<'me> {
    name    : StringIndex,
    generics: &'me [BoundedGeneric<'me>],
    kind    : SymbolKind<'me>,
}


#[derive(Debug, Clone, Copy)]
pub enum SymbolKind<'me> {
    Function(FunctionTy<'me>),
    Container(Container<'me>),
    Trait(Trait<'me>),
    Alias(Generic<'me>),
    Opaque,
    Namespace,
    /// A symbol that failed to resolve; carries the originating error so
    /// consumers can cite it. Every failed resolution registers its own
    /// entry (see `SymbolMap::error_sym`).
    Error(ErrorId),
}


#[derive(Debug, Clone, Copy)]
pub struct Trait<'me> {
    pub funcs: &'me [(StringIndex, FunctionTy<'me>)],
    pub synthesis: TraitSynthesis,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraitSynthesis {
    None,
    UniversalNoop,
}


impl<'me> Symbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [BoundedGeneric<'me>], kind: SymbolKind<'me>) -> Self {
        Self { name, generics, kind }
    }

    pub fn new_ns(name: StringIndex) -> Self {
        Self::new(name, &[], SymbolKind::Namespace)
    }

}
