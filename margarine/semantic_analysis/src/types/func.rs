use common::string_map::StringIndex;
use parser::nodes::decl::DeclId;

use super::{Generic, SymbolId};

#[derive(Debug, Clone, Copy)]
pub struct FunctionTy<'me> {
    pub args: &'me [FunctionArgument<'me>],
    pub ret : Generic<'me>,

    pub kind: FunctionKind,
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionArgument<'me> {
    pub name  : StringIndex,
    pub symbol: Generic<'me>,
    pub inout : bool,
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
