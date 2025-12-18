pub mod containers;
pub mod ty;
pub mod func;
pub mod sym_map;

use common::{string_map::StringIndex, ImmutableData};
use errors::ErrorId;
use parser::nodes::decl::DeclGeneric;
use crate::{namespace::NamespaceId, syms::sym_map::BoundedGeneric};

use self::{containers::Container, func::FunctionTy};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Symbol<'me> {
    name    : StringIndex,
    generics: &'me [BoundedGeneric<'me>],
    kind    : SymbolKind<'me>,
    err     : Option<ErrorId>,
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
    pub fn new(name: StringIndex, generics: &'me [BoundedGeneric<'me>], kind: SymbolKind<'me>) -> Self {
        Self { name, generics, kind, err: None }
    }

    pub fn new_ns(name: StringIndex) -> Self {
        Self::new(name, &[], SymbolKind::Namespace)
    }
}

