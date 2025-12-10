use std::{collections::HashMap, hash::Hash};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use llvm_api::tys::func::FunctionType;
use parser::nodes::{decl::Decl, expr::{BinaryOperator, ExprId, UnaryOperator}, stmt::StmtId, NodeId, Pattern, PatternKind, AST};
use runtime::opcode::{self, runtime::builder::Builder, HEADER};
use sti::hash::fxhash::{FxHasher32, FxHasher64};

use crate::{namespace::NamespaceMap, syms::{self, containers::ContainerKind, sym_map::{GenListId, SymbolId, SymbolMap}, ty::{Sym, TypeHash}, SymbolKind}, TyChecker, TyInfo};

pub struct Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ty_info: &'me TyInfo,
    ty_mappings: HashMap<TypeHash, Type<'ctx>>,

    funcs: HashMap<TypeHash, Function<'me>>,
    const_strs: Vec<StringIndex>,

    func_counter: u32,

    abort_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    err_fn  : (FunctionPtr<'ctx>, FunctionType<'ctx>),

}


#[derive(Clone, Copy, Debug)]
struct FuncIndex(u32);
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct BlockIndex(u32);


#[derive(Debug)]
struct Function<'a, 'ctx> {
    sym: Sym,

    name: StringIndex,
    index: FuncIndex,
    args: Vec<u32>,
    ret: u32,

    kind: FunctionKind<'a>,
    error: Option<ErrorId>,

    func_ty: FunctionType<'ctx>,
    ptr_ty: FunctionType<'ctx>,

    hits: u32,

    cached: bool,
}


#[derive(Debug)]
enum FunctionKind<'a> {
    Code {
        local_count: u8,
        entry: BlockIndex,
        blocks: Vec<Block<'a>>,
    },


    Extern(StringIndex),
}


struct Env<'a> {
    vars: Vec<(StringIndex, u32)>,
    var_counter: u32,
    block_counter: u32,
    blocks: Vec<Block<'a>>,
    loop_cont: Option<BlockIndex>,
    loop_brek: Option<BlockIndex>,
    gens: GenListId,
}


#[derive(Debug)]
struct Block<'a> {
    index: BlockIndex,
    bytecode: Builder,
    terminator: BlockTerminator<'a>,
}


#[derive(Debug, Clone, Copy)]
enum BlockTerminator<'a> {
    Goto(BlockIndex),
    SwitchBool { op1: BlockIndex, op2: BlockIndex },
    Switch(&'a [BlockIndex]),
    Err(ErrorId),
    Ret,
}


pub fn run(
    string_map: &mut StringMap, syms: &mut SymbolMap, nss: &mut NamespaceMap,
    ast: &mut AST, ty_info: &mut TyInfo, errors: [Vec<Vec<String>>; 3], startups: &[SymbolId],
) -> Vec<u8> {
    let ctx = Context::new();
    let module = ctx.module("margarine");

    let void = ctx.void();

    let abort_fn_ty = void.fn_ty(&[], false);
    let abort_fn = module.function("abort", abort_fn_ty);
    abort_fn.set_linkage(Linkage::External);
    abort_fn.set_noreturn(ctx.as_ctx_ref());

    let i32_ty = ctx.integer(32);
    let err_fn_ty = void.fn_ty(&[*i32_ty, *i32_ty, *i32_ty], false);
    let err_fn = module.function("margarineError", abort_fn_ty);
    err_fn.set_linkage(Linkage::External);
    err_fn.set_noreturn(ctx.as_ctx_ref());

    let mut conv = Conversion {
        string_map,
        syms,
        ns: nss,
        ast,
        ty_info,
        funcs: HashMap::new(),
        ty_mappings: HashMap::new(),
        const_strs: Vec::new(),
        func_counter: 0,
        abort_fn,
        err_fn,
    };


    // create IR
    for sym in startups.iter() {
        let _ = conv.get_func(Sym::Ty(*sym, GenListId::EMPTY));
    }

    todo!()
}

