use std::collections::HashMap;

use common::{copy_slice_in, source::SourceRange, string_map::StringIndex};
use parser::nodes::expr::Block;
use sti::{arena::Arena, define_key, keyed::KVec};

use crate::{errors::Error, scope::{GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, types::{hash_ty_list, Generic, SymbolMap, Type}, TyChecker};


define_key!(u32, pub FunctionSymbolId);
define_key!(u32, pub FunctionId);


#[derive(Debug, Clone, Copy)]
pub struct FunctionSymbol<'me, 'ast> {
    name    : StringIndex,
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


#[derive(Debug, Clone, Copy)]
pub struct FunctionValue<'me> {
    pub symbol  : FunctionSymbolId,

    // assumes the generics are in the same
    // order as the symbol's generics
    pub generics: &'me [Type],
}


#[derive(Debug)]
pub struct FunctionMap<'me, 'ast> {
    symbols: KVec<FunctionSymbolId, Symbol<'me, 'ast>>,
    funcs  : KVec<FunctionId      , FunctionValue<'me>>,
    arena  : &'me Arena,
}


#[derive(Debug)]
struct Symbol<'me, 'ast> {
    symbol: FunctionSymbol<'me, 'ast>,
    maps  : HashMap<u32, FunctionId>,
}

impl<'me, 'ast> FunctionMap<'me, 'ast> {
    pub fn new(arena: &'me Arena) -> Self {
        Self { symbols: KVec::new(), funcs: KVec::new(), arena } }

    #[inline(always)]
    pub fn get_func_val(&self, func: FunctionId) -> FunctionValue<'me> {
        self.funcs[func]
    }


    #[inline(always)]
    pub fn get_sym(&self, sym: FunctionSymbolId) -> FunctionSymbol<'me, 'ast> {
        self.symbols[sym].symbol
    }


    pub fn push(&mut self, sym: FunctionSymbol<'me, 'ast>) -> FunctionSymbolId {
        self.symbols.push(Symbol { symbol: sym, maps: HashMap::new() })
    }


    pub fn insert_to_sym(&mut self, map: &SymbolMap, sym: FunctionSymbolId, gens: &[Type], ty_id: FunctionId) {
        let hash = hash_ty_list(map, gens);

        assert!(self.symbols[sym].maps.insert(hash, ty_id).is_none());
    }


    pub fn get_from_sym(&mut self, map: &SymbolMap, sym: FunctionSymbolId, gens: &[Type]) -> Option<FunctionId> {
        let hash = hash_ty_list(map, gens);
        self.symbols[sym].maps.get(&hash).copied()
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
    pub fn get_func(&mut self, discard_errors: bool,
                    scope: ScopeId, sym: FunctionSymbolId,
                    gens: &[Type]) -> FunctionId {
        if let Some(val) = self.funcs.get_from_sym(&self.types, sym, gens) { return val }
        let entry = &self.funcs.symbols[sym];

        assert_eq!(entry.symbol.generics.len(), gens.len());

        let generics = {
            let mut hashmap = HashMap::with_capacity(gens.len());
            for (name, gen) in entry.symbol.generics.iter().zip(gens.iter()) {
                hashmap.insert(*name, *gen);
            }

            hashmap
        };

        let generics = sti::boks::Box::new_in(self.output, generics);
        let generics = generics.leak();
        let gen_scope = GenericsScope::new(generics);
        let mut scope = self.scopes.push(Scope::new(scope.some(),
                                ScopeKind::Generics(gen_scope)));
        
        
        // fake the arguments
        for arg in entry.symbol.args().iter() {
            let ty = self.gen_to_ty(arg.symbol, generics).unwrap_or(Type::ERROR);
            let vs = VariableScope::new(arg.name, ty, arg.inout);

            scope = self.scopes.push(Scope::new(scope.some(), ScopeKind::VariableScope(vs)));
        }


        let entry = &self.funcs.symbols[sym];
        let ret = entry.symbol.ret;
        let ret = self.gen_to_ty(ret, generics).unwrap_or(Type::ERROR);
       
        // move `gens` into the arena
        let gens = copy_slice_in(self.funcs.arena, gens);
        
        // do the body
        let body = self.funcs.symbols[sym].symbol.body;
        let err_len = self.errors.len();

        let entry = &self.funcs.symbols[sym];
        let anal = self.block(entry.symbol.name, scope, &body);

        let entry = &self.funcs.symbols[sym];
        if anal.ty.ne(&mut self.types, ret) {
            self.error(body[0], Error::FunctionBodyAndReturnMismatch {
                header: entry.symbol.header, item: body.range(),
                return_type: ret, body_type: anal.ty });
        }

        if discard_errors { self.errors.truncate(err_len); }

        // finalise
        let func = FunctionValue {
            symbol: sym,
            generics: gens,
        };

        let func = self.funcs.funcs.push(func);
        self.funcs.insert_to_sym(&self.types, sym, gens, func);
        func
    }
}
