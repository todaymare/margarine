use common::{string_map::StringIndex, ImmutableData};
use parser::{dt::FunctionAbi, nodes::decl::DeclId};

use crate::syms::sym_map::ClosureId;

use super::sym_map::{BoundedGeneric, Generic, SymbolId};

#[derive(Debug, Clone, Copy, ImmutableData, PartialEq)]
pub struct FunctionTy<'me> {
    args: &'me [FunctionArgument<'me>],
    ret : Generic<'me>,
    declared_generics: &'me [BoundedGeneric<'me>],
    abi: FunctionAbi,

    kind: FunctionKind,
    decl: Option<DeclId>,

    pub cached: bool,
}


#[derive(Debug, Clone, Copy, ImmutableData, PartialEq)]
pub struct FunctionArgument<'me> {
    name  : StringIndex,
    symbol: Generic<'me>,
    is_inout: bool,
}


#[derive(PartialEq, Debug, Clone, Copy)]
pub enum FunctionKind {
    Extern(StringIndex),

    Closure(ClosureId),
    
    UserDefined,

    TypeId,
    SizeOf,
    Trait,

    Rc,
    RcGet,
    RcSet,

    PtrAlloc,
    PtrFree,
    PtrRead,
    PtrWrite,
    PtrWriteUninit,
    PtrDrop,
    PtrNull,
    PtrOffset,
    PtrCast,

    ListConcat,
    ListSlice,
    ListLen,
    ListIter,
    ListIterNext,
    FloatSqrt,

    Enum {
        sym: SymbolId,
        index: usize,
    }
}


impl<'me> FunctionTy<'me> {
    pub fn new(
        args: &'me [FunctionArgument<'me>],
        ret: Generic<'me>,
        kind: FunctionKind,
        decl: Option<DeclId>,
        declared_generics: &'me [BoundedGeneric<'me>],
    ) -> Self {
        Self::new_with_abi(args, ret, FunctionAbi::Margarine, kind, decl, declared_generics)
    }

    pub fn new_with_abi(
        args: &'me [FunctionArgument<'me>],
        ret: Generic<'me>,
        abi: FunctionAbi,
        kind: FunctionKind,
        decl: Option<DeclId>,
        declared_generics: &'me [BoundedGeneric<'me>],
    ) -> Self {
        Self { args, ret, declared_generics, abi, kind, decl, cached: false }
    }
}


impl<'me> FunctionArgument<'me> {
    pub fn new(name: StringIndex, symbol: Generic<'me>) -> Self { Self { name, symbol, is_inout: false } }
    pub fn new_inout(name: StringIndex, symbol: Generic<'me>, is_inout: bool) -> Self { Self { name, symbol, is_inout } }
}
