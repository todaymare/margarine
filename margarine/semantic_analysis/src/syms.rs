pub mod containers;
pub mod ty;
pub mod func;
pub mod sym_map;

use common::{string_map::StringIndex, ImmutableData};
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
    Opaque,
}


impl<'me> Symbol<'me> {
    pub fn new(name: StringIndex, generics: &'me [StringIndex], kind: SymbolKind<'me>) -> Self {
        Self { name, generics, kind }
    }
}
