use common::{string_map::{StringIndex}, ImmutableData};

use super::sym_map::Generic;

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct Container<'me> {
    fields: &'me [(StringIndex, Generic<'me>)],
    kind  : ContainerKind,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContainerKind {
    /// Assumptions
    /// * All fields are named
    Struct,
    /// Assumptions
    /// * All fields are named
    Enum,
    Tuple,
}


impl<'me> Container<'me> {
    pub fn new(fields: &'me [(StringIndex, Generic<'me>)], kind: ContainerKind) -> Self {
        Self {
            fields,
            kind,
        }
    }
}
