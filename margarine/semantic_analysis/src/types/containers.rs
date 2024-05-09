use common::string_map::OptStringIndex;

use super::{Generic, Type};

#[derive(Debug, Clone, Copy)]
pub struct Container<'me> {
    pub fields: &'me [(OptStringIndex, Generic<'me>)],
    pub kind  : ContainerKind,
}


#[derive(Debug, Clone, Copy)]
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
    pub fn new(fields: &'me [(OptStringIndex, Generic<'me>)], kind: ContainerKind) -> Self {
        Self {
            fields,
            kind,
        }
    }
}
