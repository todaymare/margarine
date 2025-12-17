pub mod containers;
pub mod ty;
pub mod func;
pub mod sym_map;

use common::{string_map::StringIndex, ImmutableData};
use crate::namespace::NamespaceId;

use self::{containers::Container, func::FunctionTy};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Symbol<'me> {
    name    : StringIndex,
    generics: &'me [StringIndex],
    kind    : SymbolKind<'me>,
}


#[derive(Debug, Clone, Copy)]
pub enum SymbolKind<'me> {
    Function(FunctionTy<'me>),
    Container(Container<'me>),
    Trait(Trait<'me>),
    Opaque,
    Namespace,
}


#[derive(Debug, Clone, Copy)]
pub struct Trait<'me> {
    pub funcs: &'me [(StringIndex, FunctionTy<'me>)],
}


impl<'me> Symbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex], kind: SymbolKind<'me>) -> Self {
        Self { name, generics, kind }
    }

    pub fn new_ns(name: StringIndex) -> Self {
        Self::new(name, &[], SymbolKind::Namespace)
    }
}

