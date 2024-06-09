use common::{string_map::StringIndex, ImmutableData};
use parser::nodes::decl::DeclId;

use super::sym_map::{Generic, SymbolId};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionTy<'me> {
    args: &'me [FunctionArgument<'me>],
    ret : Generic<'me>,

    kind: FunctionKind,
}


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionArgument<'me> {
    name  : StringIndex,
    symbol: Generic<'me>,
    inout : bool,
}


#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    Extern(StringIndex),
    
    UserDefined {
        decl: DeclId,
    },

    Enum {
        sym: SymbolId,
        index: usize,
    }
}


impl<'me> FunctionTy<'me> {
    pub fn new(args: &'me [FunctionArgument<'me>], ret: Generic<'me>, kind: FunctionKind) -> Self { Self { args, ret, kind } }
}


impl<'me> FunctionArgument<'me> {
    pub fn new(name: StringIndex, symbol: Generic<'me>, inout: bool) -> Self { Self { name, symbol, inout } }
}
