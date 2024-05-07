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
    pub generics: &'me [StringIndex],

    header  : SourceRange,
    kind: FunctionType<'ast>,
}


#[derive(Debug, Clone, Copy)]
pub enum FunctionType<'ast> {
    Extern,
    UserDefined {
        body: Block<'ast>,
    }
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


impl<'me, 'ast> FunctionMap<'me, 'ast> {
    pub fn new(arena: &'me Arena) -> Self {
        Self { symbols: KVec::new(), arena } }


    #[inline(always)]
    pub fn get_sym(&self, sym: FunctionSymbolId) -> FunctionSymbol<'me, 'ast> {
        self.symbols[sym]
    }


    #[inline(always)]
    pub fn mut_sym(&mut self, sym: FunctionSymbolId) -> &mut FunctionSymbol<'me, 'ast> {
        &mut self.symbols[sym]
    }


    pub fn push(&mut self, sym: FunctionSymbol<'me, 'ast>) -> FunctionSymbolId {
        self.symbols.push(sym)
    }
}


impl<'me, 'ast> FunctionSymbol<'me, 'ast> {
    pub fn new(name: StringIndex, ret: Generic<'me>, args: &'me [FunctionArgument<'me>], generics: &'me [StringIndex], kind: FunctionType<'ast>, header: SourceRange) -> Self { Self { name, args, generics, ret, kind, header } }

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
}
