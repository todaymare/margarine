use common::{string_map::StringIndex, ImmutableData};
use parser::nodes::decl::DeclId;

use super::sym_map::{Generic, SymbolId};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionTy<'me> {
    args: &'me [FunctionArgument<'me>],
    ret : Generic<'me>,

    kind: FunctionKind,
    decl: Option<DeclId>,
}


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionArgument<'me> {
    name  : StringIndex,
    symbol: Generic<'me>,
}


#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Extern(StringIndex),
    
    UserDefined,

    TypeId,

    Enum {
        sym: SymbolId,
        index: usize,
    }
}


impl<'me> FunctionTy<'me> {
    pub fn new(args: &'me [FunctionArgument<'me>], ret: Generic<'me>, kind: FunctionKind, decl: Option<DeclId>) -> Self { Self { args, ret, kind, decl } }
}


impl<'me> FunctionArgument<'me> {
    pub fn new(name: StringIndex, symbol: Generic<'me>) -> Self { Self { name, symbol } }
}
