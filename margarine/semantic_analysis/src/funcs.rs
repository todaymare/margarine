use std::collections::HashMap;

use common::{copy_slice_in, string_map::{StringIndex, StringMap}};
use llvm_api::{builder::Local, Context, Function};
use parser::{nodes::Node, Block};
use sti::{arena::Arena, define_key, keyed::{KVec, Key}, traits::FromIn};

use crate::{scope::{GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, types::{hash_ty_list, Generic, SymbolMap, Type, TypeSymbol, TypeSymbolId}, Analyzer};


define_key!(u32, pub FunctionSymbolId);
define_key!(u32, pub FunctionId);


#[derive(Debug, Clone, Copy)]
pub struct FunctionSymbol<'me, 'ast> {
    name    : StringIndex,
    args    : &'me [FunctionArgument<'me>],
    ret     : Generic<'me>,
    generics: &'me [StringIndex],
    body    : Block<'ast>, 
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionArgument<'me> {
    pub name  : StringIndex,
    pub symbol: Generic<'me>,
    pub inout : bool,
}


#[derive(Debug, Clone, Copy)]
pub struct FunctionValue<'me> {
    symbol  : FunctionSymbolId,
    llvm_val: llvm_api::Function,

    // assumes the generics are in the same
    // order as the symbol's generics
    generics: &'me [Type],
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
    pub fn new(name: StringIndex, ret: Generic<'me>, args: &'me [FunctionArgument<'me>], generics: &'me [StringIndex], body: Block<'ast>) -> Self { Self { name, args, generics, ret, body } }

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


impl<'me, 'out, 'ast, 'str> Analyzer<'me, 'out, 'ast, 'str> {
    pub fn get_func(&mut self, discard_errors: bool, ctx: &mut Context,
                    scope: ScopeId, sym: FunctionSymbolId,
                    gens: &[Type]) -> FunctionId {
        if let Some(val) = self.funcs.get_from_sym(&self.types, sym, gens) { return val }
        let entry = &self.funcs.symbols[sym];

        assert_eq!(entry.symbol.generics.len(), gens.len());

        let pool = Arena::tls_get_rec();

        // generate the name for the func
        let name = 'l: {
            if gens.is_empty() { break 'l self.string_map.get(entry.symbol.name) }
            let mut str = sti::string::String::new_in(&*pool);
            str.push(self.string_map.get(entry.symbol.name));
            str.push("<");
            
            for (i, t) in gens.iter().enumerate() {
                if i != 0 { str.push(", "); }

                let ty = self.types.get_ty_val(*t);
                let sym = self.types.get_sym(ty.symbol);
                let name = sym.name;
                let name = self.string_map.get(name);
                str.push(name);
            }

            str.push_char('>');
            str.leak()
        };
        let name_id = self.string_map.insert(name);
        
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
        let mut llvm_args = sti::vec::Vec::with_cap_in(&*pool, entry.symbol.args().len());
        for (i, arg) in entry.symbol.args().iter().enumerate() {
            let ty = self.gen_to_ty(ctx, arg.symbol, generics).unwrap_or(Type::ERROR);
            let local = Local::from_usize(i).unwrap();
            let vs = VariableScope::new(arg.name, ty, arg.inout, local);
            dbg!(arg.symbol, ty);

            scope = self.scopes.push(Scope::new(scope.some(), ScopeKind::VariableScope(vs)));
            let ty = self.types.get_ty_val(ty);
            llvm_args.push((self.string_map.get(arg.name), ty.llvm_ty(), arg.inout));
        }


        let entry = &self.funcs.symbols[sym];
        // fake the return type
        let ret = self.gen_to_ty(ctx, entry.symbol.ret(), generics).unwrap_or(Type::ERROR);

       
        // move `gens` into the arena
        let gens = copy_slice_in(self.funcs.arena, gens);

        let llvm_ret = self.types.get_ty_val(ret).llvm_ty();

        let mut builder = Function::new(ctx, self.module,
                                     &*name, llvm_ret, &*llvm_args);


        // do the body
        let body = self.funcs.symbols[sym].symbol.body;
        let err_len = self.errors.len();

        let anal = self.block(&mut builder, name_id, scope, &body);

        builder.ret(anal.value);

        dbg!(err_len, self.errors.len(), discard_errors);
        if discard_errors { self.errors.truncate(err_len); }

        // finalise
        let func = builder.build();
        let func = FunctionValue {
            symbol: sym,
            llvm_val: func,
            generics: gens,
        };

        let func = self.funcs.funcs.push(func);
        self.funcs.insert_to_sym(&self.types, sym, gens, func);
        func
    }
}
