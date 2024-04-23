use std::collections::HashMap;

use common::{copy_slice_in, source::SourceRange, string_map::StringIndex};
use parser::nodes::expr::Block;
use sti::{arena::Arena, define_key, keyed::KVec};

use crate::{errors::Error, scope::{GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, types::{Generic, SymbolMap, Type}, TyChecker};


define_key!(u32, pub FunctionSymbolId);


#[derive(Debug, Clone, Copy)]
pub struct FunctionSymbol<'me, 'ast> {
    pub name    : StringIndex,
    args    : &'me [FunctionArgument<'me>],
    ret     : Generic<'me>,
    generics: &'me [StringIndex],
    body    : Block<'ast>, 

    header  : SourceRange,
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionArgument<'me> {
    pub name  : StringIndex,
    pub symbol: Generic<'me>,
    pub inout : bool,
}


#[derive(Debug)]
pub struct FunctionMap<'me, 'ast> {
    symbols: KVec<FunctionSymbolId, FunctionSymbol<'me, 'ast>>,
    arena  : &'me Arena,
}


pub struct Function<'me>(FunctionSymbolId, &'me [Type]);


impl<'me, 'ast> FunctionMap<'me, 'ast> {
    pub fn new(arena: &'me Arena) -> Self {
        Self { symbols: KVec::new(), arena } }


    #[inline(always)]
    pub fn get_sym(&self, sym: FunctionSymbolId) -> FunctionSymbol<'me, 'ast> {
        self.symbols[sym]
    }


    pub fn push(&mut self, sym: FunctionSymbol<'me, 'ast>) -> FunctionSymbolId {
        self.symbols.push(sym)
    }
}


impl<'me, 'ast> FunctionSymbol<'me, 'ast> {
    pub fn new(name: StringIndex, ret: Generic<'me>, args: &'me [FunctionArgument<'me>], generics: &'me [StringIndex], body: Block<'ast>, header: SourceRange) -> Self { Self { name, args, generics, ret, body, header } }

    #[inline(always)]
    pub fn generics(&self) -> &'me [StringIndex] { self.generics }

    #[inline(always)]
    pub fn args(&self) -> &'me [FunctionArgument<'me>] { self.args }

    #[inline(always)]
    pub fn ret(&self) -> Generic<'me> { self.ret }
}


impl<'me> FunctionArgument<'me> {
    pub fn new(name: StringIndex, symbol: Generic<'me>, inout: bool) -> Self { Self { name, symbol, inout } }
}


impl<'me, 'out, 'ast, 'str> TyChecker<'me, 'out, 'ast, 'str> {
    pub fn get_func(&mut self, sym: FunctionSymbolId, gens: &[Type]) -> Function<'out> {
        let gens = copy_slice_in(self.output, gens);
        Function(sym, gens)
    }
}
