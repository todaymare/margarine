use common::{string_map::StringIndex, ImmutableData};
use parser::nodes::decl::DeclId;

use crate::syms::sym_map::ClosureId;

use super::sym_map::{Generic, SymbolId};

#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionTy<'me> {
    args: &'me [FunctionArgument<'me>],
    ret : Generic<'me>,

    kind: FunctionKind,
    decl: Option<DeclId>,

    pub cached: bool,
}


#[derive(Debug, Clone, Copy, ImmutableData)]
pub struct FunctionArgument<'me> {
    name  : StringIndex,
    symbol: Generic<'me>,
}


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FunctionKind {
    Extern(StringIndex),

    Closure(ClosureId),
    
    UserDefined,

    TypeId,
    SizeOf,
    Any,
    DowncastAny,

    Enum {
        sym: SymbolId,
        index: usize,
    }
}


impl<'me> FunctionTy<'me> {
    pub fn new(args: &'me [FunctionArgument<'me>], ret: Generic<'me>, kind: FunctionKind, decl: Option<DeclId>) -> Self { Self { args, ret, kind, decl, cached: false } }
}


impl<'me> FunctionArgument<'me> {
    pub fn new(name: StringIndex, symbol: Generic<'me>) -> Self { Self { name, symbol } }
}
