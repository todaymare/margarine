use std::{collections::HashMap, hash::Hash};

use common::{string_map::{StringIndex, StringMap}, Swap};
use errors::ErrorId;
use llvm_api::{builder::{Builder, FPCmp, IntCmp, Local, Loop}, ctx::{Context, ContextRef}, module::Module, tys::{func::FunctionType, integer::IntegerTy, strct::StructTy, union::UnionTy, Type as LLVMType, TypeKind}, values::{bool::Bool, func::{FunctionPtr, Linkage}, int::Integer, ptr::Ptr, strct::Struct, Value}};
use parser::nodes::{decl::{DeclGeneric, Decl}, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::StmtId, NodeId, Pattern, PatternKind, AST};
use sti::{hash::fxhash::{FxHasher32, FxHasher64}, static_assert};

use crate::{namespace::NamespaceMap, syms::{self, containers::ContainerKind, sym_map::{BoundedGeneric, GenListId, SymbolId, SymbolMap}, ty::{Type, TypeHash}, SymbolKind}, AnalysisResult, TyChecker, TyInfo};

pub struct Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ty_info: &'me TyInfo<'out>,
    ty_mappings: HashMap<TypeHash, TypeMapping<'ctx>>,

    externs: HashMap<StringIndex, (FunctionType<'ctx>, FunctionPtr<'ctx>)>,
    funcs: HashMap<TypeHash, Function<'ctx>>,
    const_strs: Vec<StringIndex>,

    func_counter: u32,

    i32: IntegerTy<'ctx>,
    i64: IntegerTy<'ctx>,

    /// fn(): !
    abort_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(err_kind: i32, err_file: i32, err_index: i32): !
    err_fn  : (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(size: i32): ptr
    alloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr, size: i64): void
    dealloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(total_size: i64): ptr
    rc_alloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr): ptr
    rc_clone_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr, total_size: i64): void
    rc_drop_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr): void
    assert_not_null_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),


    // ptr1 is a function ptr
    // ptr2 is the environment ptr
    func_ref: StructTy<'ctx>,

    // tag is a u64
    // ptr is a ptr
    any_ref: StructTy<'ctx>,

    list_ty: StructTy<'ctx>,

    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,

}


#[derive(Clone, Copy, Debug)]
struct FuncIndex(u32);
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct BlockIndex(u32);


#[derive(Debug, Clone, Copy)]
struct TypeMapping<'ctx> {
    /// for primitives: the native representation
    /// for structs: a pointer
    /// for enums: a (tag: i32, union repr of variants)
    repr: LLVMType<'ctx>,


    /// this is either a native representation for stuff like primitives
    /// or the struct type for user types
    strct: LLVMType<'ctx>


}


#[derive(Debug)]
struct Function<'ctx> {
    sym: Type,

    name: StringIndex,

    kind: FunctionKind,
    error: Option<ErrorId>,

    func_ty: FunctionType<'ctx>,
    func_ptr: FunctionPtr<'ctx>,
}


#[derive(Debug)]
enum FunctionKind {
    Code,
    Extern(StringIndex),
}


struct Env<'a, 'ctx> {
    vars: Vec<(StringIndex, Local, Type, bool)>,
    loop_id: Option<Loop>,
    gens: &'a [(BoundedGeneric<'a>, Type)],
    info: HashMap<ExprId, Value<'ctx>>,
    ret_llvm_ty: Option<TypeMapping<'ctx>>,
}


struct Block<'a> {
    index: BlockIndex,
    bytecode: Builder<'a>,
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


pub fn run<'a>(
    string_map: &mut StringMap, syms: &mut SymbolMap<'a>, nss: &mut NamespaceMap,
    ast: &mut AST<'a>, ty_info: &mut TyInfo<'a>, errors: [Vec<Vec<String>>; 3], file_count: u32, startups: &[SymbolId], tests: &[SymbolId],
) {
    //println!("running llvm");

    let ctx = Context::new(ast.arena);
    let mut module = ctx.module("margarine");
    // generate the code 
    {
        let void = ctx.void();

        let abort_fn_ty = void.fn_ty(ctx.arena, &[], false);
        let abort_fn = module.function("margarineAbort", abort_fn_ty);
        abort_fn.set_linkage(Linkage::External);
        abort_fn.set_noreturn(ctx.as_ctx_ref());

        let i32_ty = ctx.integer(32);
        let err_fn_ty = void.fn_ty(ctx.arena, &[*i32_ty, *i32_ty, *i32_ty], false);
        let err_fn = module.function("margarineError", err_fn_ty);
        err_fn.set_linkage(Linkage::External);
        err_fn.set_noreturn(ctx.as_ctx_ref());

        let ptr = ctx.ptr();
        let i32_ty = ctx.integer(32);
        let alloc_fn_ty = ptr.fn_ty(ctx.arena, &[*ctx.integer(64)], false);
        let alloc_fn = module.function("margarineAlloc", alloc_fn_ty);
        alloc_fn.set_linkage(Linkage::External);

        let dealloc_fn_ty = void.fn_ty(ctx.arena, &[*ctx.ptr(), *ctx.integer(64)], false);
        let dealloc_fn = module.function("margarineDealloc", dealloc_fn_ty);
        dealloc_fn.set_linkage(Linkage::External);

        let rc_alloc_fn_ty = ptr.fn_ty(ctx.arena, &[*ctx.integer(64)], false);
        let rc_alloc_fn = module.function("margarineRcAlloc", rc_alloc_fn_ty);
        rc_alloc_fn.set_linkage(Linkage::External);

        let rc_clone_fn_ty = ptr.fn_ty(ctx.arena, &[*ctx.ptr()], false);
        let rc_clone_fn = module.function("margarineRcClone", rc_clone_fn_ty);
        rc_clone_fn.set_linkage(Linkage::External);

        let rc_drop_fn_ty = void.fn_ty(ctx.arena, &[*ctx.ptr(), *ctx.integer(64)], false);
        let rc_drop_fn = module.function("margarineRcDrop", rc_drop_fn_ty);
        rc_drop_fn.set_linkage(Linkage::External);

        let assert_not_null_fn_ty = void.fn_ty(ctx.arena, &[*ctx.ptr()], false);
        let assert_not_null_fn = module.function("margarineAssertNotNull", assert_not_null_fn_ty);
        assert_not_null_fn.set_linkage(Linkage::External);

        let func_ref = ctx.structure("funcRef");
        func_ref.set_fields(&[*ctx.ptr(), *ctx.ptr()], false);


        let any_ref = ctx.structure("anyType");
        any_ref.set_fields(&[*ctx.ptr(), *i32_ty], false);


        let list_ty = ctx.structure("listType");
        list_ty.set_fields(&[*ctx.integer(64), *i32_ty, *i32_ty, *ctx.ptr()], false);

        let mut conv = Conversion {
            string_map,
            syms,
            ns: nss,
            ast,
            ty_info,
            funcs: HashMap::new(),
            externs: HashMap::new(),
            ty_mappings: HashMap::new(),
            const_strs: Vec::new(),
            func_counter: 0,
            abort_fn: (abort_fn, abort_fn_ty),
            err_fn: (err_fn, err_fn_ty),
            alloc_fn: (alloc_fn, alloc_fn_ty),
            dealloc_fn: (dealloc_fn, dealloc_fn_ty),
            rc_alloc_fn: (rc_alloc_fn, rc_alloc_fn_ty),
            rc_clone_fn: (rc_clone_fn, rc_clone_fn_ty),
            rc_drop_fn: (rc_drop_fn, rc_drop_fn_ty),
            assert_not_null_fn: (assert_not_null_fn, assert_not_null_fn_ty),
            i32: i32_ty,
            i64: ctx.integer(64),
            ctx: ctx.as_ctx_ref(),
            func_ref,
            module,
            list_ty,
            any_ref,
        };

        conv.externs.insert(conv.string_map.insert("margarineAlloc"), (alloc_fn_ty, alloc_fn));
        conv.externs.insert(conv.string_map.insert("margarineAbort"), (abort_fn_ty, abort_fn));


        // register primitives
        {
            macro_rules! register {
                ($enum: ident, $call: expr) => {{
                    let val = $call;
                    conv.ty_mappings.insert(Type::$enum.hash(conv.syms), TypeMapping { repr: *val, strct: *val });
                }};
            }


            register!(I64, ctx.integer(64));
            register!(F64, ctx.f64());
            register!(UNIT, ctx.unit());
        }


        // create IR
        for sym in startups.iter() {
            let _ = conv.get_func(Type::Ty(*sym, GenListId::EMPTY));
        }

        for sym in tests.iter() {
            let _ = conv.get_func(Type::Ty(*sym, GenListId::EMPTY));
        }

        module = conv.module;
    }


    // transfer errors into the binary
    {
        let u32_ty = ctx.integer(32);
        let ptr_ty = ctx.ptr();
        
        let lex_error_files = &errors[0];
        let parse_error_files = &errors[1];
        let sema_errors = &errors[2][0];

        // file count
        {
            let file_count_global = module.add_global(*u32_ty, "fileCount");
            let file_count_value = ctx.const_int(u32_ty, file_count as i64, false);
            file_count_global.set_initialiser(*file_count_value);
        }


        // lexer errors
        {
            let strct = ctx.structure("lexer_err_ty");
            strct.set_fields(&[*u32_ty, *ptr_ty], false);

            let errs_ty = ctx.array(*strct, parse_error_files.len());
            let mut err_arr_values = Vec::with_capacity(parse_error_files.len());

            for file in lex_error_files {
                let file_err_array_ty = ctx.array(*ptr_ty, file.len());
                let mut file_arr_values = Vec::with_capacity(file.len());

                for s in file {
                    let global = ctx.const_str(&*s);
                    let ptr = module.add_global(*global.ty(), "");
                    ptr.set_initialiser(*global);
                    file_arr_values.push(*ptr);
                }

                // value arr
                let ptr = module.add_global(*file_err_array_ty, "");
                let arr = ctx.const_array(*ptr_ty, &file_arr_values);
                ptr.set_initialiser(*arr);

                let len = ctx.const_int(u32_ty, file.len() as i64, false);

                let strct = ctx.const_struct(strct, &[*len, *ptr]);
                err_arr_values.push(*strct);
            }

            // value arr
            let ptr = module.add_global(*errs_ty, "lexerErrors");
            let arr = ctx.const_array(*strct, &err_arr_values);
            ptr.set_initialiser(*arr);
        }


        // parser errors
        {
            let strct = ctx.structure("parser_err_ty");
            strct.set_fields(&[*u32_ty, *ptr_ty], false);

            let errs_ty = ctx.array(*strct, parse_error_files.len());
            let mut err_arr_values = Vec::with_capacity(parse_error_files.len());

            for file in parse_error_files {
                let file_err_array_ty = ctx.array(*ptr_ty, file.len());
                let mut file_arr_values = Vec::with_capacity(file.len());

                for s in file {
                    let global = ctx.const_str(&*s);
                    let ptr = module.add_global(*global.ty(), "");
                    ptr.set_initialiser(*global);
                    file_arr_values.push(*ptr);
                }

                // value arr
                let ptr = module.add_global(*file_err_array_ty, "");
                let arr = ctx.const_array(*ptr_ty, &file_arr_values);
                ptr.set_initialiser(*arr);

                let len = ctx.const_int(u32_ty, file.len() as i64, false);

                let strct = ctx.const_struct(strct, &[*len, *ptr]);
                err_arr_values.push(*strct);
            }

            // value arr
            let ptr = module.add_global(*errs_ty, "parserErrors");
            let arr = ctx.const_array(*strct, &err_arr_values);
            ptr.set_initialiser(*arr);
        }


        // sema errors
        let sema_err_array_ty = ctx.array(*ptr_ty, sema_errors.len());
        let mut arr_values = Vec::with_capacity(sema_errors.len());

        for e in sema_errors {
            let global = ctx.const_str(&*e);
            let ptr = module.add_global(*global.ty(), "");
            ptr.set_initialiser(*global);
            arr_values.push(*ptr);
        }

        let ptr = module.add_global(*sema_err_array_ty, "semaErrors");
        let array = ctx.const_array(*ptr.ty(), &arr_values);
        ptr.set_initialiser(*array);


        let ptr = module.add_global(*u32_ty, "semaErrorsLen");
        let num = ctx.const_int(u32_ty, sema_errors.len().try_into().unwrap(), false);
        ptr.set_initialiser(*num);

    }


    let dump = module.dump_to_str();

    std::fs::write("out.ll", dump.as_str().as_bytes()).unwrap();
}



impl<'me, 'out, 'ast, 'str, 'ctx> Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    fn get_func(&mut self, ty: Type) -> Result<&Function<'ctx>, ErrorId> {
        assert!(ty.is_resolved(&mut self.syms));

        let sym_id = ty.sym(self.syms).unwrap();
        let gens_id = ty.gens(&self.syms);

        let hash = ty.hash(&self.syms);

        if let Some(func) = self.funcs.get(&hash) { 
            assert!(func.sym.eq(self.syms, ty));
            return Ok(self.funcs.get(&hash).unwrap())
        }

        // create
        let sym = self.syms.sym(sym_id);
        if let Some(err) = sym.err() { return Err(err) }

        let SymbolKind::Function(sym_func) = sym.kind()
        else { unreachable!() };

        let gens = self.syms.gens()[gens_id];

        assert_eq!(gens.len(), sym.generics().len());
        for ((g0, _), n1) in gens.iter().zip(sym.generics()) {
            assert_eq!(g0.name, n1.name);
        }

        let ret = sym_func.ret().to_ty(gens, self.syms).unwrap();
        let is_never = ret.is_never(self.syms) || ret.is_err(self.syms);

        let llvm_ret = self.to_llvm_ty(ret);

        let args = sym_func
            .args().iter()
            .map(|x| 
                x.symbol()
                .to_ty(gens, self.syms).unwrap()
            ).collect::<Vec<_>>();

        let llvm_args = {
            let mut vec = sti::vec::Vec::with_cap_in(&*self.ctx.arena, sym_func.args().len());
            for i in &args {
                let ty = self.to_llvm_ty(*i);
                vec.push(ty.repr);
            }

            vec.push(*self.ctx.ptr());

            vec.leak()
        };

        let name = ty.display(self.string_map, self.syms);
        let name_idx = self.string_map.insert(name);

        match sym_func.kind() {
            syms::func::FunctionKind::Extern(path) => {
                let (func_ty, func_ptr) =
                if let Some(vals) = self.externs.get(&path) { *vals }
                else {
                    let func_ty = llvm_ret.repr.fn_ty(
                        self.ctx.arena, 
                        llvm_args.as_slice(),
                        false,
                    );


                    let func_ptr = self.module.function(self.string_map.get(path), func_ty);
                    func_ptr.set_linkage(Linkage::External);

                    if is_never {
                        func_ptr.set_noreturn(self.ctx);
                    }

                    self.externs.insert(path, (func_ty, func_ptr));
                    self.externs[&path]
                };

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(self.string_map.get(path)),
                    kind: FunctionKind::Extern(path),
                    error: self.ty_info.decl(sym_func.decl().unwrap()),

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());
                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::UserDefined => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    llvm_args.as_slice(),
                    false,
                );


                let func_ptr = self.module.function(name, func_ty);

                if is_never {
                    func_ptr.set_noreturn(self.ctx);
                }


                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: self.ty_info.decl(sym_func.decl().unwrap()),

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let mut env = Env {
                    vars: Vec::new(),
                    loop_id: None,
                    gens: self.syms.get_gens(gens_id),
                    info: HashMap::new(),
                    ret_llvm_ty: Some(llvm_ret),
                };

                for (i, arg) in sym_func.args().iter().enumerate() {
                    let arg_ty = arg.symbol().to_ty(gens, self.syms).unwrap();
                    let arg_ty = arg_ty.resolve(&[], self.syms);
                    env.alloc_var(arg.name(), builder.arg(i).unwrap(), arg_ty, true);
                }

                let Decl::Function { body, .. } = self.ast.decl(sym_func.decl().unwrap())
                else { unreachable!() };


                let result = self.block(&mut env, &mut builder, &*body);
                
                if let Some(e) = self.ty_info.decl(sym_func.decl().unwrap()) {
                    self.error(&mut builder, e);
                } else {
                    match result {
                        Ok(v) => {
                            if !is_never {
                                self.drop_all_locals(&env, &mut builder);
                                builder.ret(v);
                            } else {
                                builder.unreachable();
                            }
                        },


                        Err(e) => {
                            println!("error in function {}: {:?}", self.string_map.get(name_idx), e);
                            self.error(&mut builder, e);
                        },
                    }
                }


                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::TypeId => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &[],
                    false,
                );

                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);
                
                let id = gens[0].1.sym(self.syms).unwrap();
                let num = builder.const_int(self.i64, id.0 as i64, false);
                builder.ret(*num);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::SizeOf => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &[],
                    false,
                );

                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);
                
                let sym = gens[0].1;
                let ty = self.to_llvm_ty(sym);
                let size = ty.repr.size_of(self.module).unwrap();

                let num = builder.const_int(self.i64, size as i64, false);
                builder.ret(*num);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::Rc => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let elem_size = llvm_elem.repr.size_of(self.module).unwrap();
                let total_size = 8 + elem_size;

                let size_val = builder.const_int(self.i64, total_size as i64, false);
                let ptr = builder.call(self.rc_alloc_fn.0, self.rc_alloc_fn.1, &[*size_val]).as_ptr();

                let arg = builder.arg(0).unwrap();
                let arg = builder.local_get(arg);

                let offset = builder.const_int(self.i64, 1, false);
                let data_ptr = builder.gep(ptr, *self.i64, offset);
                builder.store(data_ptr, arg);

                builder.ret(*ptr);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::RcGet => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let arg = builder.arg(0).unwrap();
                let arg = builder.local_get(arg);

                let offset = builder.const_int(self.i64, 1, false);
                let data_ptr = builder.gep(arg.as_ptr(), *self.i64, offset);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let result = builder.load(data_ptr, llvm_elem.repr);

                builder.ret(result);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::RcSet => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let rc_arg = builder.arg(0).unwrap();
                let rc_arg = builder.local_get(rc_arg);

                let val_arg = builder.arg(1).unwrap();
                let val_arg = builder.local_get(val_arg);

                let offset = builder.const_int(self.i64, 1, false);
                let data_ptr = builder.gep(rc_arg.as_ptr(), *self.i64, offset);
                builder.store(data_ptr, val_arg);

                builder.ret(*builder.const_unit());

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrAlloc => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let elem_size = llvm_elem.repr.size_of(self.module).unwrap() as i64;
                let elem_size_val = builder.const_int(self.i64, elem_size, false);

                let count = builder.local_get(builder.arg(0).unwrap()).as_integer();
                let total_size = builder.mul_int(count, elem_size_val);

                let ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*total_size]);
                builder.ret(ptr);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrFree => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let elem_size = llvm_elem.repr.size_of(self.module).unwrap() as i64;
                let elem_size_val = builder.const_int(self.i64, elem_size, false);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let count = builder.local_get(builder.arg(1).unwrap()).as_integer();
                let total_size = builder.mul_int(count, elem_size_val);

                builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*ptr, *total_size]);
                builder.ret(*builder.const_unit());

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrRead => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                builder.call(self.assert_not_null_fn.0, self.assert_not_null_fn.1, &[*ptr]);
                let val = builder.load(ptr, llvm_elem.repr);
                let result = self.emit_copy(&mut builder, val, elem_ty);

                builder.ret(result);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrWrite => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                builder.call(self.assert_not_null_fn.0, self.assert_not_null_fn.1, &[*ptr]);
                let val = builder.local_get(builder.arg(1).unwrap());

                let old = builder.load(ptr, llvm_elem.repr);
                self.emit_drop(&mut builder, old, elem_ty);
                builder.store(ptr, val);
                builder.ret(*builder.const_unit());

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrNull => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &[],
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);
                let null = builder.ptr_null();
                builder.ret(*null);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrOffset => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let off = builder.local_get(builder.arg(1).unwrap()).as_integer();

                let gep = builder.gep(ptr, llvm_elem.repr, off);
                builder.ret(*gep);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrCast => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena,
                    &llvm_args,
                    false,
                );
                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);

                let to_ty = gens[1].1;
                let llvm_to = self.to_llvm_ty(to_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let cast = builder.ptr_bitcast(ptr, llvm_to.repr);
                builder.ret(*cast);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::Any => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &llvm_args,
                    false,
                );

                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);
                
                let sym = gens[0].1;
                let ty = self.to_llvm_ty(sym);

                let size = ty.repr.size_of(self.module).unwrap();
                let id = sym.sym(self.syms).unwrap();

                let size = builder.const_int(self.i64, size as i64, false);
                let id = builder.const_int(self.i32, id.0 as i64, false);

                let ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]).as_ptr();

                let arg = builder.arg(0).unwrap();
                let arg = builder.local_get(arg);

                builder.store(ptr, arg);

                let strct = builder.struct_instance(self.any_ref, [*ptr, *id]);
                builder.ret(*strct);

                return Ok(&self.funcs[&hash]);
            },



            syms::func::FunctionKind::DowncastAny => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &llvm_args,
                    false,
                );

                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut builder = func_ptr.builder(self.ctx, func_ty);
                
                let target_sym = gens[0].1;
                let target_id = target_sym.sym(self.syms).unwrap();

                let target_id = builder.const_int(self.i32, target_id.0 as i64, false);

                let curr = builder.arg(0).unwrap();
                let curr = builder.local_get(curr).as_struct();
                let curr_id = builder.field_load(curr, 1).as_integer();

                let cmp = builder.cmp_int(target_id, curr_id, IntCmp::Ne);
                let cmp = builder.int_cast(cmp.as_integer(), *self.i32, false);
                let payload_ptr = builder.field_load(curr, 0).as_ptr();

                let ret_gens = ret.gens(self.syms);
                let ret_gens = self.syms.get_gens(ret_gens);
                let payload_ty = ret_gens[0].1;
                let payload_llvm = self.to_llvm_ty(payload_ty);
                let payload_data = builder.load(payload_ptr, payload_llvm.repr);

                let buf = self.create_enum_from_llvm(&mut builder, cmp, payload_data, llvm_ret);
                builder.ret(buf);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::Enum { sym: sym_id, index } => {
                let sym = self.syms.sym(sym_id);
                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!() };

                let arg = cont.fields()[index];
                let arg_ty = arg.1.to_ty(gens, self.syms).unwrap();

                let is_unit = arg_ty.sym(self.syms).unwrap() == SymbolId::UNIT;

                let func_ty =

                if is_unit {
                    llvm_ret.repr.fn_ty(
                        self.ctx.arena, 
                        &[llvm_args[0]],
                        false,
                    )
                } else {
                    assert_eq!(args.len(), 1);
                    llvm_ret.repr.fn_ty(
                        self.ctx.arena, 
                        &llvm_args,
                        false,
                    )
                };



                let func_ptr = self.module.function(name, func_ty);

                let func = Function {
                    sym: ty,
                    name: name_idx,
                    kind: FunctionKind::Code,
                    error: sym_func.decl().map(|e| self.ty_info.decl(e)).flatten(),

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let mut builder = func_ptr.builder(self.ctx, func_ty);
                
                let en = 
                if is_unit {
                    let kind = builder.const_int(self.i32, index as _, false);
                    let unit = *builder.const_unit();
                    self.create_enum(&mut builder, ret, *kind, unit)
                } else {
                    let kind = builder.const_int(self.i32, index as _, false);
                    let value = builder.arg(0).unwrap();
                    let value = builder.local_get(value);
                    self.create_enum(&mut builder, ret, *kind, value)
                };

                builder.ret(en);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::Closure(_) => unreachable!(),
            syms::func::FunctionKind::Trait => unreachable!(),
        }
    }


    fn error(&self, builder: &mut Builder<'_>, e: errors::ErrorId) {
        let (err_ty, err_file, err_index) = match e {
            ErrorId::Lexer(v) => (0, v.0, v.1.0),
            ErrorId::Parser(v) => (1, v.0, v.1.0),
            ErrorId::Sema(v) => (2, 0, v.0),
            ErrorId::Bypass => {
                builder.unreachable();
                return;
            },
        };

        let i32_ty = self.i32;

        let err_ty    = builder.const_int(i32_ty, err_ty, false);
        let err_file  = builder.const_int(i32_ty, err_file as i64, false);
        let err_index = builder.const_int(i32_ty, err_index as i64, false);

        builder.call(self.err_fn.0, self.err_fn.1, &[*err_ty, *err_file, *err_index]);
        builder.unreachable();
    }


    fn to_llvm_ty(&mut self, ty: Type) -> TypeMapping<'ctx> {
        assert!(ty.is_resolved(self.syms));

        let hash = ty.hash(self.syms);
        
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let sym_id = ty.sym(self.syms).unwrap();

        if sym_id == SymbolId::ANY {
            self.ty_mappings.insert(hash, TypeMapping { repr: *self.any_ref, strct: *self.any_ref });
            return self.ty_mappings[&hash]
        }


        if sym_id == SymbolId::LIST {
            self.ty_mappings.insert(hash, TypeMapping { repr: *self.ctx.ptr(), strct: *self.list_ty });
            return self.ty_mappings[&hash]
        }

        let sym = self.syms.sym(sym_id);

        let gens = ty.gens(self.syms);
        let gens = self.syms.get_gens(gens);

        let name = ty.display(self.string_map, self.syms);

        match sym.kind() {
            SymbolKind::Function(function_ty) => {
                let mapping = TypeMapping { repr: *self.func_ref, strct: *self.ctx.void() };
                self.ty_mappings.insert(hash, mapping);

                let ret = function_ty.ret().to_ty(gens, self.syms).unwrap();
                //dbg!(ret.display(self.string_map, self.syms));

                let ret = self.to_llvm_ty(ret).repr;

                let llvm_args = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*self.ctx.arena, function_ty.args().len());
                    for i in function_ty.args().iter() {
                        let arg = i.symbol().to_ty(gens, self.syms).unwrap();
                        let ty = self.to_llvm_ty(arg);
                        vec.push(ty.repr);
                    }

                    vec.push(*self.ctx.ptr());

                    vec.leak()
                };


                let strct = ret.fn_ty(self.ctx.arena, llvm_args, false);
                let mapping = TypeMapping { repr: *self.func_ref, strct: *strct };
                self.ty_mappings.insert(hash, mapping);
            },


            SymbolKind::Container(cont) => {
                match cont.kind() {
                      ContainerKind::Tuple => {
                        let mut fields = sti::vec::Vec::with_cap_in(&*self.ctx.arena, cont.fields().len());

                        for i in cont.fields() {
                            let ty = i.1.to_ty(gens, self.syms).unwrap();
                            fields.push(self.to_llvm_ty(ty).repr);
                        }

                        let strct = self.ctx.literal_struct(fields.as_slice(), false);
                        let mapping = TypeMapping { repr: *strct, strct: *strct };

                        self.ty_mappings.insert(hash, mapping);
                    },


                    ContainerKind::Struct => {
                        let strct = self.ctx.structure(name);
                        let mapping = TypeMapping { repr: *strct, strct: *strct };

                        self.ty_mappings.insert(hash, mapping);

                        let fields = {
                            let mut vec = sti::vec::Vec::with_cap_in(&*self.ctx.arena, cont.fields().len());

                            for i in cont.fields() {
                                let ty = i.1.to_ty(gens, self.syms).unwrap();
                                vec.push(self.to_llvm_ty(ty).repr);
                            }

                            vec.leak()
                        };

                        strct.set_fields(fields.as_slice(), false);
                    },


                    ContainerKind::Enum => {
                        let strct = self.ctx.structure(name);
                        let mapping = TypeMapping { repr: *strct, strct: *strct };
                        self.ty_mappings.insert(hash, mapping);

                        let mut payload_tys = sti::vec::Vec::with_cap_in(&*self.ctx.arena, cont.fields().len());
                        for field in cont.fields() {
                            let ft = field.1.to_ty(gens, self.syms).unwrap();
                            let llvm = self.to_llvm_ty(ft);
                            payload_tys.push(llvm.repr);
                        }

                        if payload_tys.is_empty() {
                            strct.set_fields(&[*self.i32, *self.ctx.unit()], false);
                        } else {
                            let mut union_name = sti::string::String::with_cap_in(&*self.ctx.arena, name.len() + 7);
                            union_name.push(name);
                            union_name.push(".union");
                            let union_ty = self.ctx.union(union_name.leak());
                            union_ty.set_fields(self.ctx, self.module, payload_tys.as_slice());
                            strct.set_fields(&[*self.i32, *union_ty], false);
                        }
                    },



                    ContainerKind::Generic => unreachable!(),
                }
            },


            SymbolKind::Opaque => {
                self.ty_mappings.insert(hash, TypeMapping { repr: *self.ctx.ptr(), strct: *self.ctx.ptr() });
            },


            SymbolKind::Namespace => unreachable!(),
            SymbolKind::Trait(_) => unreachable!(),
        };


        self.ty_mappings[&hash]
    }



    fn block(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, block: &[NodeId]
    ) -> Result<Value<'ctx>, ErrorId> {
        let mut has_ret = None;
        let len = env.vars.len();


        for (_, &n) in block.iter().enumerate() {
            has_ret = None;
            match n {
                NodeId::Decl(_) => (),


                NodeId::Stmt(stmt_id) => self.stmt(env, builder, stmt_id)?,


                NodeId::Expr(expr_id) => {
                    let result = self.expr(env, builder, expr_id)?;
                    has_ret = Some(result);
                },


                NodeId::Err(error_id) => {
                    env.vars.truncate(len);
                    return Err(error_id);
                },
            }
        }

        self.drop_locals(env, builder, len);
        env.vars.truncate(len);
        match has_ret {
            Some(v) => Ok(v),
            None => Ok(*builder.const_unit()),
        }
    }



    fn stmt(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, stmt: StmtId
    ) -> Result<(), ErrorId> {
        macro_rules! out_if_err {
            () => {{
                match self.ty_info.stmt(stmt) {
                    Some(e) => {
                        return Err(e);
                    },


                    None => (),
               }
            }};
        }


        let val = self.ast.stmt(stmt);

        match val {
            parser::nodes::stmt::Stmt::Variable { pat, rhs, .. } => {
                let value = self.expr(env, builder, rhs)?;
                out_if_err!();

                let margarine_ty = self.ty_info.expr(rhs).unwrap();
                let margarine_ty = margarine_ty.resolve(&[env.gens], self.syms);

                let ty = self.to_llvm_ty(margarine_ty);

                Self::resolve_pattern(self, env, builder, margarine_ty, ty, value, pat);

                Ok(())
            },


            parser::nodes::stmt::Stmt::UpdateValue { lhs, rhs } => {
                if let Err(e) = self.ty_info.expr(lhs) { return Err(e) }
                let rhs = self.expr(env, builder, rhs)?;

                out_if_err!();

                self.assign(env, builder, lhs, rhs);
                Ok(())
            },


            parser::nodes::stmt::Stmt::ForLoop { binding, expr, body } => {
                let iter_expr = self.expr(env, builder, expr)?;
                out_if_err!();

                let (iter_fn_ret_ty, func_ptr, func_ty) = {
                    let iter_sym = self.ty_info.expr(expr).unwrap();
                    let iter_sym = iter_sym.resolve(&[env.gens], self.syms);

                    let sym = iter_sym.sym(self.syms).unwrap();
                    let ns = self.syms.sym_ns(sym);
                    let ns = self.ns.get_ns(ns);

                    let Ok(sym) = ns.get_sym(StringMap::ITER_NEXT_FUNC).unwrap()
                    else { unreachable!() };

                    let func = Type::Ty(sym, iter_sym.gens(&self.syms));
                    let func = func.resolve(&[], self.syms);

                    let ret_ty = self.syms.sym(sym);
                    let SymbolKind::Function(ret_ty) = ret_ty.kind()
                    else { unreachable!() };

                    let gens = iter_sym.gens(self.syms);
                    let gens = self.syms.get_gens(gens);
                    let ret_ty = ret_ty.ret().to_ty(gens, self.syms).unwrap();

                    let func = self.get_func(func).unwrap();
                    (ret_ty, func.func_ptr, func.func_ty)
                };

                let iter_fn_binding_value_ty = iter_fn_ret_ty.gens(&self.syms);
                let iter_fn_binding_value_ty = self.syms.get_gens(iter_fn_binding_value_ty)[0].1;
                let iter_fn_binding_value_ty_sym = iter_fn_binding_value_ty.resolve(&[env.gens], self.syms);
                let iter_fn_binding_value_ty_llvm = self.to_llvm_ty(iter_fn_binding_value_ty_sym);

                builder.loop_indefinitely(|builder, l| {
                    let null = builder.ptr_null();
                    let call_ret_value = builder.call(func_ptr, func_ty, &[iter_expr, *null]).as_struct();

                    let lo = env.loop_id.swap(Some(l));

                    let tag = builder.field_load(call_ret_value, 0).as_integer();
                    let none_case = builder.const_int(tag.as_integer().ty(), 1, false);
                    let cond = builder.cmp_int(tag, none_case, IntCmp::Eq);

                    builder.ite(&mut (), cond, 
                    |builder, _| {
                        builder.loop_break(l);
                    }, |_, _| {});

                    let ret_alloca = builder.alloca_store(*call_ret_value);
                    let data_ptr = builder.field_ptr(ret_alloca, call_ret_value.ty(), 1);
                    let value = builder.load(data_ptr, iter_fn_binding_value_ty_llvm.repr);

                    Self::resolve_pattern(self, env, builder, iter_fn_binding_value_ty_sym, iter_fn_binding_value_ty_llvm, value, binding);

                    let result = self.block(env, builder, &*body);
                    if let Err(e) = result {
                        self.error(builder, e);
                    };

                    env.loop_id = lo;
                    if let Some((_, local, ty, _)) = env.vars.last().copied() {
                        let value = builder.local_get(local);
                        self.emit_drop(builder, value, ty);
                    }
                    env.vars.pop();
                });

                Ok(())
            },
        }

    }


    fn resolve_lvalue_ptr(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>,
        expr: ExprId,
    ) -> Ptr<'ctx> {
        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Identifier(name, _) => {
                let local = env.find_var(name).unwrap();
                builder.local_ptr(local)
            }


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                let parent_ptr = self.resolve_lvalue_ptr(env, builder, val);

                let ty = self.ty_info.expr(val).unwrap();
                if ty.is_err(self.syms) { unreachable!() }

                let ty = ty.resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(ty);

                let sym = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!() };

                let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                    let name = f.0;
                    field_name == name
                }).unwrap();


                builder.field_ptr(parent_ptr, llvm_ty.strct.as_struct(), i)
            }


            parser::nodes::expr::Expr::IndexList { list, index } => {
                let list_val = match self.ast.expr(list) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        builder.local_get(local).as_ptr()
                    },
                    _ => self.expr(env, builder, list).unwrap().as_ptr(),
                };
                let index_val = self.expr(env, builder, index).unwrap().as_integer();

                let elem_ty = self.ty_info.expr(list).unwrap();
                let elem_ty = elem_ty.gens(self.syms);
                let elem_ty = self.syms.get_gens(elem_ty)[0].1;
                let elem_ty = elem_ty.resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(elem_ty);

                let buf_ptr = builder.field_ptr(list_val, self.list_ty, 3);
                let buf_ptr = builder.load(buf_ptr, *self.ctx.ptr()).as_ptr();
                builder.gep(buf_ptr, llvm_ty.repr, index_val)
            }


            _ => unreachable!("invalid lvalue"),
        }
    }


    fn assign(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, 
        expr: ExprId, value: Value<'ctx>
    ) {
        self.ty_info.expr(expr).unwrap();

        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Identifier(name, _) => {
                let local = env.find_var(name).unwrap();
                if let Some(old_ty) = env.find_var_ty(name) {
                    let old_value = builder.local_get(local);
                    self.emit_drop(builder, old_value, old_ty);
                }
                builder.local_set(local, value)
            }


            parser::nodes::expr::Expr::AccessField { .. } => {
                let ptr = self.resolve_lvalue_ptr(env, builder, expr);
                builder.store(ptr, value);
            }


            parser::nodes::expr::Expr::Unwrap(expr) => {
                let ty = self.ty_info.expr(expr).unwrap();
                if ty.is_err(self.syms) { unreachable!() }
                let ty = ty.resolve(&[env.gens], self.syms);
                let llvm_enum_ty = self.to_llvm_ty(ty);

                match self.ast.expr(expr) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        let enum_struct = builder.local_get(local).as_struct();

                        // unwrap
                        let some = builder.const_int(self.i32, 0, false);
                        let tag = builder.field_load(enum_struct, 0);

                        let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                        builder.ite(&mut (), comp,
                        |_, _| {}, 


                        |builder, _| {
                            builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                        }, 
                        );


                        let value = self.create_enum_from_llvm(builder, tag, value, llvm_enum_ty);
                        builder.local_set(local, value);
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        let ty = self.ty_info.expr(val).unwrap();
                        if ty.is_err(self.syms) { unreachable!() }

                        let sym = ty.sym(self.syms).unwrap();
                        let sym = self.syms.sym(sym);

                        let SymbolKind::Container(cont) = sym.kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                            let name = f.0;
                            field_name == name
                        }).unwrap();

                        let gens = ty.gens(self.syms);
                        let gens = self.syms.get_gens(gens);
                        let field_ty = cont.fields()[i].1.to_ty(gens, self.syms).unwrap();
                        let field_ty = field_ty.resolve(&[env.gens], self.syms);
                        let field_llvm_ty = self.to_llvm_ty(field_ty);


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                let field = self.resolve_lvalue_ptr(env, builder, expr);


                                let enum_struct = builder
                                    .load(field, field_llvm_ty.repr).as_struct();

                                // unwrap
                                let some = builder.const_int(self.i32, 0, false);
                                let tag = builder.field_load(enum_struct, 0);

                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                                }, 
                                );


                                let value = self.create_enum_from_llvm(builder, tag, value, field_llvm_ty);
                                builder.store(field, value);
                            },


                            ContainerKind::Enum => {
                                let val_ty = ty.resolve(&[env.gens], self.syms);
                                let val_llvm_ty = self.to_llvm_ty(val_ty);
                                let enum_ptr = self.resolve_lvalue_ptr(env, builder, val);
                                let strct = builder.load(enum_ptr, val_llvm_ty.repr).as_struct();
                                let tag = builder.field_load(strct, 0);

                                // unwrap
                                let some = builder.const_int(self.i32, i as i64, false);
                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                                }, 
                                );


                                let data_ptr = builder.field_ptr(enum_ptr, val_llvm_ty.strct.as_struct(), 1);
                                builder.store(data_ptr, value);
                            },


                            ContainerKind::Generic => unreachable!(),
                        }


                    }
                    _ => (),
                }
            }


            parser::nodes::expr::Expr::OrReturn(expr) => {
                let ty = self.ty_info.expr(expr).unwrap();
                if ty.is_err(self.syms) { unreachable!() }
                let ty = ty.resolve(&[env.gens], self.syms);
                let llvm_enum_ty = self.to_llvm_ty(ty);
                let ret_llvm_ty = env.ret_llvm_ty;

                match self.ast.expr(expr) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        let enum_struct = builder.local_get(local).as_struct();

                        // unwrap
                        let some = builder.const_int(self.i32, 0, false);
                        let tag = builder.field_load(enum_struct, 0);

                        let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                        builder.ite(&mut (), comp,
                        |_, _| {}, 


                        |builder, _| {
                            self.drop_all_locals(env, builder);
                            if let Some(ret_ty) = ret_llvm_ty {
                                let none_tag = builder.const_int(self.i32, 1, false);
                                let none_value = *builder.const_unit();
                                let ret_val = self.create_enum_from_llvm(builder, *none_tag, none_value, ret_ty);
                                builder.ret(ret_val);
                            } else {
                                builder.ret(*enum_struct);
                            }
                        }, 
                        );


                        let value = self.create_enum_from_llvm(builder, tag, value, llvm_enum_ty);
                        builder.local_set(local, value);
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        let ty = self.ty_info.expr(val).unwrap();
                        if ty.is_err(self.syms) { unreachable!() }

                        let sym = ty.sym(self.syms).unwrap();
                        let sym = self.syms.sym(sym);

                        let SymbolKind::Container(cont) = sym.kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                            let name = f.0;
                            field_name == name
                        }).unwrap();

                        let gens = ty.gens(self.syms);
                        let gens = self.syms.get_gens(gens);
                        let field_ty = cont.fields()[i].1.to_ty(gens, self.syms).unwrap();
                        let field_ty = field_ty.resolve(&[env.gens], self.syms);
                        let field_llvm_ty = self.to_llvm_ty(field_ty);


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                let field = self.resolve_lvalue_ptr(env, builder, expr);


                                let enum_struct = builder
                                    .load(field, field_llvm_ty.repr).as_struct();

                                // unwrap
                                let some = builder.const_int(self.i32, 0, false);
                                let tag = builder.field_load(enum_struct, 0);

                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    self.drop_all_locals(env, builder);
                                    if let Some(ret_ty) = ret_llvm_ty {
                                        let none_tag = builder.const_int(self.i32, 1, false);
                                        let none_value = *builder.const_unit();
                                        let ret_val = self.create_enum_from_llvm(builder, *none_tag, none_value, ret_ty);
                                        builder.ret(ret_val);
                                    } else {
                                        builder.ret(*enum_struct);
                                    }
                                }, 
                                );


                                let value = self.create_enum_from_llvm(builder, tag, value, field_llvm_ty);
                                builder.store(field, value);
                            },


                            ContainerKind::Enum => {
                                let val_ty = ty.resolve(&[env.gens], self.syms);
                                let val_llvm_ty = self.to_llvm_ty(val_ty);
                                let enum_ptr = self.resolve_lvalue_ptr(env, builder, val);
                                let strct = builder.load(enum_ptr, val_llvm_ty.repr).as_struct();
                                let tag = builder.field_load(strct, 0);

                                // unwrap
                                let some = builder.const_int(self.i32, i as i64, false);
                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    self.drop_all_locals(env, builder);
                                    let none_tag = builder.const_int(self.i32, 1, false);
                                    let none_value = *builder.const_unit();
                                    let ret_ty = ret_llvm_ty.unwrap_or(val_llvm_ty);
                                    let ret_enum = self.create_enum_from_llvm(builder, *none_tag, none_value, ret_ty);
                                    builder.ret(ret_enum);
                                }, 
                                );


                                let data_ptr = builder.field_ptr(enum_ptr, val_llvm_ty.strct.as_struct(), 1);
                                builder.store(data_ptr, value);
                            },


                            ContainerKind::Generic => unreachable!(),
                        }


                    }
                    _ => (),
                }
            }


            Expr::IndexList { list, index } => {
                let list_val = match self.ast.expr(list) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        builder.local_get(local).as_ptr()
                    },
                    _ => self.expr(env, builder, list).unwrap().as_ptr(),
                };
                let index_val = self.expr(env, builder, index).unwrap().as_integer();

                let elem_ty = self.ty_info.expr(list).unwrap();
                let elem_ty = elem_ty.gens(self.syms);
                let elem_ty = self.syms.get_gens(elem_ty)[0].1;
                let elem_ty = elem_ty.resolve(&[env.gens], self.syms);

                let llvm_ty = self.to_llvm_ty(elem_ty);

                let list_ptr_slot = builder.alloca(*self.ctx.ptr());
                builder.store(list_ptr_slot, *list_val);

                let header = builder.load(list_val, *self.list_ty).as_struct();
                let refcount = builder.field_load(header, 0).as_integer();
                let one_i64 = builder.const_int(self.i64, 1, false);
                let needs_cow = builder.cmp_int(refcount, one_i64, IntCmp::SignedGt);

                builder.ite(&mut (), needs_cow,
                |builder, _| {
                    let new_rc = builder.sub_int(refcount, one_i64);
                    let rc_ptr = builder.field_ptr(list_val, self.list_ty, 0);
                    builder.store(rc_ptr, *new_rc);

                    let len = builder.field_load(header, 1).as_integer();
                    let cap = builder.field_load(header, 2).as_integer();
                    let old_data = builder.field_load(header, 3).as_ptr();

                    let elem_size = llvm_ty.repr.size_of(self.module).unwrap() as i64;
                    let elem_size_val = builder.const_int(self.i64, elem_size, false);
                    let cap_i64 = builder.int_cast(cap, *self.i64, false).as_integer();
                    let buf_size = builder.mul_int(cap_i64, elem_size_val);
                    let new_buf = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*buf_size]).as_ptr();

                    let zero_i64 = builder.const_int(self.i64, 0, false);
                    let counter = builder.alloca(*self.i64);
                    builder.store(counter, *zero_i64);

                    let len_i64 = builder.int_cast(len, *self.i64, false).as_integer();

                    builder.loop_indefinitely(|builder, l| {
                        let i = builder.load(counter, *self.i64).as_integer();

                        let done = builder.cmp_int(i, len_i64, IntCmp::SignedGe);
                        builder.ite(&mut () as &mut (), done,
                            |builder, _| { builder.loop_break(l); },
                            |builder, _| {
                                let i_i32 = builder.int_cast(i, *self.i32, false).as_integer();
                                let old_ptr = builder.gep(old_data, llvm_ty.repr, i_i32);
                                let old_elem = builder.load(old_ptr, llvm_ty.repr);
                                let new_elem = self.emit_copy(builder, old_elem, elem_ty);

                                let new_ptr = builder.gep(new_buf, llvm_ty.repr, i_i32);
                                builder.store(new_ptr, new_elem);

                                let next_i = builder.add_int(i, one_i64);
                                builder.store(counter, *next_i);
                            },
                        );
                    });

                    let header_size = self.list_ty.size_of(self.module).unwrap() as i64;
                    let header_size_val = builder.const_int(self.i64, header_size, false);
                    let new_header_ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*header_size_val]).as_ptr();

                    let new_header_strct = builder.struct_instance(self.list_ty, [*one_i64, *len, *cap, *new_buf]);
                    builder.store(new_header_ptr, *new_header_strct);

                    match self.ast.expr(list) {
                        parser::nodes::expr::Expr::Identifier(name, _) => {
                            let local = env.find_var(name).unwrap();
                            builder.local_set(local, *new_header_ptr);
                        },
                        parser::nodes::expr::Expr::AccessField { .. } => {
                            let ptr = self.resolve_lvalue_ptr(env, builder, list);
                            builder.store(ptr, *new_header_ptr);
                        },
                        _ => {},
                    }

                    builder.store(list_ptr_slot, *new_header_ptr);
                }, |_, _| {});

                let current_list = builder.load(list_ptr_slot, *self.ctx.ptr()).as_ptr();

                let buf_ptr = builder.field_ptr(current_list, self.list_ty, 3);
                let buf_ptr = builder.load(buf_ptr, *self.ctx.ptr()).as_ptr();

                let elem_ptr = builder.gep(buf_ptr, llvm_ty.repr, index_val);
                let old_elem = builder.load(elem_ptr, llvm_ty.repr);
                self.emit_drop(builder, old_elem, elem_ty);
                builder.store(elem_ptr, value);
            }


            _ => unreachable!("{:?}", self.ast.expr(expr)),
        }
    }


    fn expr(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, expr: ExprId
    ) -> Result<Value<'ctx>, ErrorId> {
        self.expr_ex(env, builder, expr).map(|x| x.0)
    }


    fn expr_ex(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, expr: ExprId
    ) -> Result<(Value<'ctx>, Type), ErrorId> {
        macro_rules! out_if_err {
            () => {{
                match self.ty_info.expr(expr) {
                    Ok(e) => e,
                    Err(e) => {
                        println!("Error in expr {:?}: {:?}", self.ast.expr(expr), e);
                        return Err(e);
                    },
               }
            }};
        }


        let val = self.ast.expr(expr);
        let ty = self.ty_info.exprs[expr];


        Ok((match val {
            parser::nodes::expr::Expr::Unit => *builder.const_unit(),
            parser::nodes::expr::Expr::Literal(literal) => {
                match literal {
                    lexer::Literal::Integer(v) => *builder.const_int(self.i64, v, true),
                    lexer::Literal::Float(f) => *builder.const_f64(f.inner()),


                    lexer::Literal::String(string_index) => {
                        let string = self.string_map.get(string_index);
                        let array_ty = self.ctx.array(
                            *self.ctx.integer(8),
                            string.len()
                        );

                        let strct_ty = self.ctx.structure("str");
                        strct_ty.set_fields(&[*self.i32, *array_ty], true);

                        let len = self.ctx.const_int(self.i32, string.len() as _, false);
                        let arr = *self.ctx.const_str(string);
                        let val = self.ctx.const_struct(strct_ty, &[*len, arr]);

                        let ptr = self.module.add_global(*strct_ty, "str");
                        ptr.set_initialiser(*val);

                        *ptr
                    },


                    lexer::Literal::Bool(v) => {
                        let kind = builder.const_bool(v);
                        let value = *builder.const_unit();
                        self.create_enum(builder, Type::BOOL, *kind, value)
                    },
                }
            },


            parser::nodes::expr::Expr::Paren(expr_id) => self.expr(env, builder, expr_id)?,


            parser::nodes::expr::Expr::Identifier(name, _) => {
                let ty = out_if_err!();

                let ty = ty.resolve(&[env.gens], self.syms);

                let func = 
                // its a trait func
                if let Some(tr) = self.ty_info.trait_funcs.get(&expr) {
                    let sym = self.ty_info.idents.get(&expr).unwrap().unwrap();
                    let sym = Type::Ty(sym, GenListId::EMPTY);
                    let sym = sym.resolve(&[env.gens], self.syms);

                    let sym = sym.sym(self.syms).unwrap();

                    let (ns, _, _) = self.syms.traits(sym).get(tr).unwrap();
                    self.ns.get_ns(*ns).get_sym(name).unwrap().ok()
                    
                } else if let Some(Some(sym)) = self.ty_info.idents.get(&expr) {
                    Some(*sym)

                } else {
                    None
                };

                // it's a function
                if let Some(func) = func {
                    let func_gens = ty.gens(self.syms);
                    let sym = Type::Ty(func, func_gens);

                    let sym = sym.resolve(&[env.gens], self.syms);

                    let func = self.get_func(sym).unwrap();
                    //dbg!(func.func_ty);

                    // create func ref
                    // we want a null ptr as the environment pointer
                    // since named-funcs have no captures we don't 
                    // need to allocate anything
                    let null = builder.ptr_null();
                    let ptr = func.func_ptr;
                    let func_ty = func.func_ty;
                    let ty = self.func_ref;
                    let func_ref = builder.struct_instance(
                        ty,
                        [*ptr, *null],
                    );

                    return Ok((*func_ref, sym))
                }


                let value = builder.local_get(env.find_var(name).unwrap());
                if env.is_var_param(name) {
                    value
                } else {
                    self.emit_copy(builder, value, ty)
                }
            },


            parser::nodes::expr::Expr::Range { lhs, rhs } => {
                let lhs = self.expr(env, builder, lhs)?;
                let rhs = self.expr(env, builder, rhs)?;

                out_if_err!();

                let fields = sti::vec_in![
                    self.ctx.arena; 
                    (StringMap::MIN, lhs),
                    (StringMap::MAX, rhs),
                ];

                *self.create_struct(builder, Type::RANGE, &fields)
            },


            parser::nodes::expr::Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs_val = self.expr(env, builder, lhs)?;
                let rhs_val = self.expr(env, builder, rhs)?;
                out_if_err!();

                let sym = self.ty_info.expr(lhs).unwrap();
                let sym = sym.resolve(&[env.gens], self.syms);

                let result = match operator {
                    BinaryOperator::Eq => {
                        let local = builder.local(*self.ctx.bool());
                        let f = builder.const_bool(true);
                        builder.local_set(local, *f);

                        self.eq(builder, sym, local, lhs_val, rhs_val);

                        builder.local_get(local)
                    },


                    BinaryOperator::Ne => {
                        let local = builder.local(*self.ctx.bool());
                        let f = builder.const_bool(true);
                        builder.local_set(local, *f);

                        self.eq(builder, sym, local, lhs_val, rhs_val);

                        let local = builder.local_get(local);
                        *builder.bool_not(local.as_bool())
                    },

                    _ => {
                        let sym = sym.sym(self.syms).unwrap();
                        if sym.is_int() {
                            let l = lhs_val.as_integer();
                            let r = rhs_val.as_integer();
                            let signed = sym.is_sint();

                            match operator {
                              BinaryOperator::Add => *builder.add_int(l, r),
                              BinaryOperator::Sub => *builder.sub_int(l, r),
                              BinaryOperator::Mul => *builder.mul_int(l, r),
                              BinaryOperator::Div => *builder.div_int(l, r, signed),
                              BinaryOperator::Rem => *builder.rem_int(l, r, signed),
                              BinaryOperator::BitshiftLeft => *builder.shl(l, r),
                              BinaryOperator::BitshiftRight => *builder.shr(l, r, signed),
                              BinaryOperator::BitwiseAnd => *builder.and(l, r),
                              BinaryOperator::BitwiseOr => *builder.or(l, r),
                              BinaryOperator::BitwiseXor => *builder.xor(l, r),
                              BinaryOperator::Gt => *builder.cmp_int(l, r, IntCmp::SignedGt),
                              BinaryOperator::Ge => *builder.cmp_int(l, r, IntCmp::SignedGe),
                              BinaryOperator::Lt => *builder.cmp_int(l, r, IntCmp::SignedLt),
                              BinaryOperator::Le => *builder.cmp_int(l, r, IntCmp::SignedLe), 

                              _ => unreachable!(),
                            }

                        } else if sym.is_num() {
                            let l = lhs_val.as_fp();
                            let r = rhs_val.as_fp();

                            match operator {
                              BinaryOperator::Add => *builder.add_fp(l, r),
                              BinaryOperator::Sub => *builder.sub_fp(l, r),
                              BinaryOperator::Mul => *builder.mul_fp(l, r),
                              BinaryOperator::Div => *builder.div_fp(l, r),
                              BinaryOperator::Rem => *builder.rem_fp(l, r),
                              BinaryOperator::Gt => *builder.cmp_fp(l, r, FPCmp::Gt),
                              BinaryOperator::Ge => *builder.cmp_fp(l, r, FPCmp::Ge),
                              BinaryOperator::Lt => *builder.cmp_fp(l, r, FPCmp::Lt),
                              BinaryOperator::Le => *builder.cmp_fp(l, r, FPCmp::Le), 

                              _ => unreachable!(),
                            }

                        } else { unreachable!() }
                    },
                };


                if operator.is_ocomp() || operator.is_ecomp() {
                    assert_eq!(result.ty().kind(), TypeKind::Integer);

                    let value = *builder.const_unit();

                    self.create_enum(
                        builder,
                        Type::BOOL,
                        result,
                        value,
                    )
                } else {
                    result 
                }
            },

            parser::nodes::expr::Expr::UnaryOp { operator, rhs } => {
                let rhs = self.expr(env, builder, rhs)?;
                out_if_err!();
                
                match operator {
                    UnaryOperator::Not => {
                        let buf = builder.alloca_store(rhs);
                        let tag_ptr = builder.field_ptr(buf, rhs.ty().as_struct(), 0);

                        let value = builder.load(tag_ptr, *self.i32).as_integer();
                        let value = builder.int_cast(value, *self.ctx.bool(), false);
                        let value = builder.bool_not(value.as_bool()).as_integer();
                        let value = builder.int_cast(value, *self.i32, false);

                        builder.store(tag_ptr, value);
                        builder.load(buf, rhs.ty())
                    },

                    UnaryOperator::Neg => {
                        if rhs.ty().kind() == TypeKind::Integer {
                            let c = builder.const_int(rhs.ty().as_integer(), -1, true);
                            *builder.mul_int(rhs.as_integer(), c)
                        } else {
                            let c = builder.const_f64(-1.0);
                            *builder.mul_fp(rhs.as_fp(), c)
                        }
                    },
                }
            },


            parser::nodes::expr::Expr::If { condition, body, else_block } => {
                let cond = self.expr(env, builder, condition)?;

                let ty = out_if_err!().resolve(&[env.gens], self.syms);

                let local = {
                    let ty = self.to_llvm_ty(ty);
                    Some(builder.local(ty.repr))
                };

                let tag = builder.field_load(cond.as_struct(), 0).as_integer();
                let tag = builder.int_cast(tag, *self.ctx.bool(), false).as_bool();

                builder.ite(&mut (self, env), tag,
                |builder, (this, env)| {
                    let Ok(value) = this.expr(env, builder, body)
                    else { return; };

                    if let Some(local) = local {
                        builder.local_set(local, value);
                    }
                },


                |builder, (slf, env)| {
                    let Some(body) = else_block
                    else { return; };

                    let Ok(value) = slf.expr(env, builder, body)
                    else { return; };

                    if let Some(local) = local {
                        builder.local_set(local, value);
                    }
                },
                );

                if let Some(local) = local {
                    builder.local_get(local)
                } else {
                    *builder.const_unit()
                }
            },


            parser::nodes::expr::Expr::Match { value, mappings } => {
                let val = self.expr(env, builder, value)?;
                let ty = out_if_err!().resolve(&[env.gens], self.syms);

                let sym = self.ty_info.expr(value).unwrap();
                let sym = sym.resolve(&[env.gens], self.syms);

                let gens = sym.gens(self.syms);
                let sym_id = sym.sym(self.syms).unwrap();
                let sym_data = self.syms.sym(sym_id);

                let SymbolKind::Container(cont) = sym_data.kind()
                else { unreachable!() };

                let value_ty = val.as_struct();
                let value_tag = builder.field_load(value_ty, 0).as_integer();

                let enum_llvm_ty = self.to_llvm_ty(sym);
                let val_alloca = builder.alloca_store(val);
                let val_data_ptr = builder.field_ptr(val_alloca, enum_llvm_ty.strct.as_struct(), 1);

                let iter = cont.fields().iter().map(|sf| {
                    let name = sf.0;
                    (sf, mappings.iter().find(|x| x.variant() == name).unwrap())
                });

                let ret_ty = self.to_llvm_ty(ty);
                let ret_local = builder.local(ret_ty.repr);

                builder.switch(value_tag, iter,
                |builder, (field, mapping)| {
                    // initialize the binding
                    let gens = self.syms.get_gens(gens);
                    let field_ty = field.1.to_ty(gens, self.syms).unwrap();
                    let field_ty = field_ty.resolve(&[env.gens], self.syms);
                    let field_ty_llvm = self.to_llvm_ty(field_ty);

                    let local = builder.local(field_ty_llvm.repr);
                    let value = if field_ty.sym(self.syms).unwrap() == SymbolId::UNIT {
                        *builder.const_unit()
                    } else {
                        builder.load(val_data_ptr, field_ty_llvm.repr)
                    };
                    builder.local_set(local, value);

                    env.vars.push((mapping.binding(), local, field_ty, false));

                    // run the body
                    let ret_val = self.expr(env, builder, mapping.expr());
                    debug_assert_eq!(env.vars.pop().unwrap(), (mapping.binding(), local, field_ty, false));

                    let Ok(ret_val) = ret_val
                    else { return };

                    builder.local_set(ret_local, ret_val);
                });

                if ty.is_never(self.syms) { *builder.const_unit() }
                else { builder.local_get(ret_local) }
            },


            parser::nodes::expr::Expr::IndexList { list, index } => {
                let list_value = match self.ast.expr(list) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        builder.local_get(local)
                    },
                    _ => self.expr(env, builder, list)?,
                };
                let index = self.expr(env, builder, index)?.as_integer();
                out_if_err!();

                let elem_ty = {
                    let ty = self.ty_info.expr(list).unwrap();
                    let ty = ty.gens(self.syms);
                    let ty = self.syms.get_gens(ty);
                    let ty = ty[0].1.resolve(&[env.gens], self.syms);
                    self.to_llvm_ty(ty)
                };

                let strct = builder.load(list_value.as_ptr(), *self.list_ty).as_struct();
                let len = builder.field_load(strct, 1).as_integer();
                let len = builder.int_cast(len, *self.i64, true).as_integer();

                let buf = builder.field_load(strct, 3).as_ptr();

                let is_lt_len = builder.cmp_int(index, len, IntCmp::SignedLt);
                let zero = builder.const_int(self.i64, 0, false);
                let is_ge_zero = builder.cmp_int(index, zero, IntCmp::SignedGe);

                let is_in_bounds = builder.bool_and(is_lt_len, is_ge_zero);

                builder.ite(
                &mut (),
                is_in_bounds,
                |_, _| {},

                |builder, _| {
                    // @todo: put a proper err msg
                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                });

                let ptr = builder.gep(buf, elem_ty.repr, index);
                builder.load(ptr, elem_ty.repr)
            },


            parser::nodes::expr::Expr::Block { block } => {
                self.block(env, builder, &*block)?
            },


            parser::nodes::expr::Expr::CreateStruct { fields, .. } => {
                let mut values = sti::vec::Vec::with_cap_in(self.ctx.arena, fields.len());

                let ty = out_if_err!();
                println!("CreateStruct: {:?}", ty.display(self.string_map, self.syms));
                let ty = ty.resolve(&[env.gens], self.syms);

                for (name, _, e) in fields {
                    let value = self.expr(env, builder, *e)?;
                    values.push((*name, value));
                }

                *self.create_struct(builder, ty, &*values)
            },


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                let value = self.expr(env, builder, val)?;
                env.info.insert(expr, value);

                let slf = out_if_err!();

                let val = self.ty_info.expr(val).unwrap();
                let val = val.resolve(&[env.gens], self.syms);
                let ty = val.sym(self.syms).unwrap();

                if let SymbolKind::Container(cont) = self.syms.sym(ty).kind()
                && let Some((i, _)) = cont.fields().iter().enumerate().find(|(_, f)| {
                    let name = f.0;
                    field_name == name
                }) {
                    match cont.kind() {
                          ContainerKind::Tuple
                        | ContainerKind::Struct => {
                            return Ok((builder.field_load(value.as_struct(), i as _), slf))
                        },

                        ContainerKind::Enum => {
                            let val_ty = val.resolve(&[env.gens], self.syms);
                            let val_llvm_ty = self.to_llvm_ty(val_ty);
                            let result_ty = slf.resolve(&[env.gens], self.syms);
                            let result_llvm_ty = self.to_llvm_ty(result_ty);

                            let enum_strct = value.as_struct();
                            let tag = builder.field_load(enum_strct, 0).as_integer();
                            let index = builder.const_int(self.i32, i as _, false);
                            let cond = builder.cmp_int(tag, index, IntCmp::Eq);

                            let gens_list = self.syms.get_gens(val.gens(self.syms));
                            let field_gen = cont.fields()[i].1;
                            let field_ty = field_gen.to_ty(gens_list, self.syms).unwrap();
                            let field_llvm = self.to_llvm_ty(field_ty);

                            let src_buf = builder.alloca_store(value);
                            let src_data_ptr = builder.field_ptr(src_buf, val_llvm_ty.strct.as_struct(), 1);

                            let result_buf = builder.alloca(result_llvm_ty.repr);
                            let result_strct_ty = result_llvm_ty.strct.as_struct();

                            builder.ite(
                                &mut (),
                                cond,
                                |builder, _| {
                                    let some_tag = builder.const_int(self.i32, 0, false);
                                    let tag_ptr = builder.field_ptr(result_buf, result_strct_ty, 0);
                                    builder.store(tag_ptr, *some_tag);
                                    let payload = builder.load(src_data_ptr, field_llvm.repr);
                                    let dst_data = builder.field_ptr(result_buf, result_strct_ty, 1);
                                    builder.store(dst_data, payload);
                                },
                                |builder, _| {
                                    let none_tag = builder.const_int(self.i32, 1, false);
                                    let tag_ptr = builder.field_ptr(result_buf, result_strct_ty, 0);
                                    builder.store(tag_ptr, *none_tag);
                                },
                            );

                            return Ok((builder.load(result_buf, result_llvm_ty.repr), slf))
                        },


                        ContainerKind::Generic => unreachable!(),
                    }

                }


                let sym_gens = self.syms.get_gens(val.gens(self.syms));

                let ns = 
                if let Some(tr) = self.ty_info.trait_funcs.get(&expr) {
                    self.syms.traits(ty)[tr].0
                } else {
                    self.syms.sym_ns(ty)
                };


                let ns = self.ns.get_ns(ns);


                if let Some(sym) = ns.get_sym(field_name) {
                    let sym = sym.unwrap();
                    let gens = slf.gens(self.syms);
                    //dbg!(self.syms.get_gens(gens));

                    let sym = Type::Ty(sym, gens)
                        .resolve(&[env.gens, sym_gens], self.syms);

                    assert!(sym.is_resolved(self.syms));

                    let func = self.get_func(sym)?;

                    // create func ref
                    // we want a null ptr as the environment pointer
                    // since named-funcs have no captures we don't 
                    // need to allocate anything
                    let null = builder.ptr_null();
                    let ptr = func.func_ptr;
                    let ty = self.func_ref;
                    let func_ref = builder.struct_instance(
                        ty,
                        [*ptr, *null],
                    );

                    return Ok((*func_ref, sym))
                }

                unreachable!()

            },


            parser::nodes::expr::Expr::CallFunction { lhs, args } => {
                let mut llvm_args = sti::vec::Vec::with_cap_in(self.ctx.arena, args.len());
                for arg in args {
                    let arg = self.expr(env, builder, *arg)?;
                    llvm_args.push(arg);
                }

                out_if_err!();
                let (func, func_ty) = self.expr_ex(env, builder, lhs)?;

                if let Expr::AccessField { .. } = self.ast.expr(lhs) {
                    llvm_args.insert(0, env.info[&lhs]);
                }


                let func_ty = func_ty.resolve(&[env.gens], self.syms);
                assert!(func_ty.is_resolved(self.syms));

                let func_ty = self.to_llvm_ty(func_ty);

                let func_ptr = builder.field_load(func.as_struct(), 0);
                let capture_ptr = builder.field_load(func.as_struct(), 1);

                llvm_args.push(capture_ptr);
                //dbg!(&llvm_args);
                builder.call(func_ptr.as_func(), func_ty.strct.as_func(), &llvm_args)
            },


            parser::nodes::expr::Expr::Closure { args, body } => {
                let ty = out_if_err!();

                let closure = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(closure);
                let SymbolKind::Function(func_ty) = sym.kind()
                else { unreachable!() };

                let syms::func::FunctionKind::Closure(closure) = func_ty.kind()
                else { unreachable!() };

                let outer_gens = env.gens;
                let ty = ty.resolve(&[outer_gens], self.syms);
                let closure_gens = self.syms.get_gens(ty.gens(&self.syms));
                let llvm_ty = self.to_llvm_ty(ty);

                let mut combined_gens = sti::vec::Vec::with_cap_in(self.syms.arena(), closure_gens.len() + outer_gens.len());
                combined_gens.extend_from_slice(closure_gens);
                combined_gens.extend_from_slice(outer_gens);
                let combined_gens: &[_] = combined_gens.leak();

                let captured = self.syms.closure(closure).captured_variables.clone();

                let mut hash = FxHasher64::new();
                for capture in &captured {
                    let ty = capture.1.resolve(&[combined_gens], self.syms);
                    ty.hash(self.syms).hash(&mut hash);
                }

                let hash = ty.hash_fn(self.syms, |h| {
                    expr.hash(h);
                    hash.hash.hash(h);
                    self.funcs.len().hash(h);
                });


                let mut tys: Vec<LLVMType<'ctx>> = Vec::with_capacity(captured.len());
                let mut vals: Vec<Value<'ctx>> = Vec::with_capacity(captured.len());
                let mut drop_tys: Vec<Type> = Vec::with_capacity(captured.len());
                for name in &captured {
                    let index = env.find_var(name.0).unwrap();
                    let value = builder.local_get(index);
                    let capture_ty = name.1.resolve(&[env.gens], self.syms);
                    let value = self.emit_copy(builder, value, capture_ty);
                    tys.push(value.ty());
                    vals.push(value);
                    drop_tys.push(capture_ty);
                }
                
                let mut strct_fields: Vec<LLVMType<'ctx>> = Vec::with_capacity(captured.len() + 2);
                strct_fields.push(*self.i64);
                strct_fields.push(*self.ctx.ptr());
                strct_fields.extend_from_slice(&tys);
                let strct_ty = self.ctx.structure("captures");
                strct_ty.set_fields(&strct_fields, false);

                let one = builder.const_int(self.i64, 1, false);
                let zero = builder.const_int(self.i64, 0, false);

                let void = self.ctx.void();
                let drop_fn_ty = void.fn_ty(self.ctx.arena, &[*self.ctx.ptr()], false);
                let drop_fn = self.module.function("__closure_drop", drop_fn_ty);

                {
                    let mut drop_builder = drop_fn.builder(self.ctx, drop_fn_ty);
                    let arg = drop_builder.arg(0).unwrap();
                    let drop_ptr = drop_builder.local_get(arg).as_ptr();

                    let drop_header = drop_builder.load(drop_ptr, *strct_ty).as_struct();
                    let rc = drop_builder.field_load(drop_header, 0).as_integer();
                    let new_rc = drop_builder.sub_int(rc, one);

                    let rc_ptr = drop_builder.field_ptr(drop_ptr, strct_ty, 0);
                    drop_builder.store(rc_ptr, *new_rc);

                    let is_zero = drop_builder.cmp_int(new_rc, zero, IntCmp::Eq);

                    drop_builder.ite(&mut (), is_zero,
                    |builder, _| {
                        for i in (0..drop_tys.len()).rev() {
                            let capture_ty = drop_tys[i];
                            let value = builder.field_load(drop_header, i + 2);
                            self.emit_drop(builder, value, capture_ty);
                        }
                        let size = strct_ty.size_of(self.module).unwrap() as i64;
                        let size_val = builder.const_int(self.i64, size, false);
                        builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*drop_ptr, *size_val]);
                    },
                    |_, _| {},
                    );

                    drop_builder.ret_void();
                }

                let mut all_vals: Vec<Value<'ctx>> = Vec::with_capacity(captured.len() + 2);
                all_vals.push(*one);
                all_vals.push(*drop_fn);
                all_vals.extend_from_slice(&vals);

                let captures = builder.struct_instance(strct_ty, all_vals);

                let size = strct_ty.size_of(self.module);
                let size = builder.const_int(self.i64, size.unwrap() as _, false);
                let buf = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]).as_ptr();
                builder.store(buf, *captures);


                let func = {
                    let llvm_func_ty = llvm_ty.strct.as_func();
                    let func_ptr = self.module.function("<closure>", llvm_func_ty);

                    let func = Function {
                        sym: ty,
                        name: StringMap::CLOSURE,
                        kind: FunctionKind::Code,
                        error: None,
                        func_ty: llvm_func_ty,
                        func_ptr,
                    };


                    assert!(self.funcs.insert(hash, func).is_none());

                    let mut builder = func_ptr.builder(self.ctx, llvm_func_ty);

                    let mut env = Env {
                        vars: Vec::new(),
                        loop_id: None,
                        gens: combined_gens,
                        info: HashMap::new(),
                        ret_llvm_ty: None,
                    };


                    let captured_ptr = builder.arg(func_ty.args().len() as _).unwrap();
                    let captured_ptr = builder.local_get(captured_ptr).as_ptr();
                    let captured_strct = builder.load(captured_ptr, *strct_ty).as_struct();

                    for (i, capture) in captured.iter().enumerate() {
                        let value = builder.field_load(captured_strct, i + 2);
                        let capture_ty = capture.1.resolve(&[combined_gens], self.syms);
                        let value = self.emit_copy(&mut builder, value, capture_ty);
                        let local = builder.local(value.ty());
                        builder.local_set(local, value);
                        env.alloc_var(capture.0, local, capture_ty, false);
                    }


                    for (i, arg) in func_ty.args().iter().enumerate() {
                        let arg_ty = arg.symbol().to_ty(env.gens, self.syms).unwrap();
                        let arg_ty = arg_ty.resolve(&[], self.syms);
                        env.alloc_var(arg.name(), builder.arg(i).unwrap(), arg_ty, true);
                    }

                    let result = self.expr(&mut env, &mut builder, body);

                    match result {
                        Ok(v) => {
                            self.drop_all_locals(&env, &mut builder);
                            builder.ret(v);
                        },
                        Err(e) => self.error(&mut builder, e),
                    };
 
                    &self.funcs[&hash]
                };


                let func_ref = builder.struct_instance(self.func_ref, [*func.func_ptr, *buf]);
                *func_ref
            },


              parser::nodes::expr::Expr::WithinNamespace { action, .. }
            | parser::nodes::expr::Expr::WithinTypeNamespace { action, .. } => {
                out_if_err!();
                return self.expr_ex(env, builder, action)
            },


            parser::nodes::expr::Expr::Loop { body } => {
                let lid = env.loop_id;
                let mut value = Ok(());

                builder.loop_indefinitely(
                |builder, l| {
                    env.loop_id = Some(l);
                    let result = self.block(env, builder, &body);

                    if let Err(e) = result {
                        self.error(builder, e);
                        value = Err(e) 
                    };
                });

                env.loop_id = lid;
                out_if_err!();

                *builder.const_unit()
            },


            parser::nodes::expr::Expr::Return(expr_id) => {
                let val = self.expr(env, builder, expr_id)?;
                out_if_err!();

                self.drop_all_locals(env, builder);
                builder.ret(val);
                *builder.const_unit()
            },



            parser::nodes::expr::Expr::Continue => {
                out_if_err!();

                builder.loop_continue(env.loop_id.unwrap());
                *builder.const_unit()
            },


            parser::nodes::expr::Expr::Break => {
                out_if_err!();

                builder.loop_break(env.loop_id.unwrap());
                *builder.const_unit()
            },


            parser::nodes::expr::Expr::Tuple(exprs) => {
                let llvm_exprs = {
                    let mut vec = Vec::with_capacity(exprs.len());
                    for (i, &e) in exprs.iter().enumerate() {
                        vec.push((self.string_map.num(i), self.expr(env, builder, e)?));
                    }

                    vec
                };

                let ty = self.ty_info.expr(expr).unwrap();

                let ty = ty.resolve(&[env.gens], self.syms);
                *self.create_struct(builder, ty, &llvm_exprs)
            },


            parser::nodes::expr::Expr::AsCast { lhs, .. } => {
                let lsym = self.ty_info.expr(lhs).unwrap().sym(self.syms).unwrap();
                let lhs = self.expr(env, builder, lhs)?;
                out_if_err!();


                let ty = out_if_err!();
                let dest = self.to_llvm_ty(ty);


                if lsym.is_int() && ty.is_float(self.syms) {
                    builder.si_to_fp(lhs.as_integer(), dest.repr)
                } else if lsym.is_float() && ty.is_int(self.syms) {
                    builder.fp_to_si(lhs.as_fp(), dest.repr.as_integer())
                } else if lsym == SymbolId::BOOL && ty.is_int(self.syms) {
                    let tag = builder.field_load(lhs.as_struct(), 0);
                    builder.int_cast(tag.as_integer(), dest.repr, false)
                } else if lsym == ty.sym(self.syms).unwrap() {
                    lhs
                } else {
                    unreachable!()
                }
            },


            parser::nodes::expr::Expr::CreateList { exprs } => {
                let llvm_exprs = {
                    let mut vec = Vec::with_capacity(exprs.len());
                    for &e in exprs.iter() {
                        vec.push(self.expr(env, builder, e)?);
                    }

                    vec
                };

                println!("list");
                let list_ty = out_if_err!();
                println!("list_ty: {:?}", list_ty);
                let list_ty = list_ty.resolve(&[env.gens], self.syms);
                let list_ty = self.to_llvm_ty(list_ty);

                let buf = {
                    let buf_size = list_ty.repr.size_of(self.module).unwrap() * exprs.len();
                    let buf_size = builder.const_int(self.i64, buf_size as i64, false);
                    let ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*buf_size]);
                    ptr.as_ptr()
                };


                for (i, &value) in llvm_exprs.iter().enumerate() {
                    let index = builder.const_int(self.i32, i as i64, false);
                    let ptr = builder.gep(buf, list_ty.repr, index);
                    builder.store(ptr, value);
                }


                let len = builder.const_int(self.i32, exprs.len() as i64, false);

                let size = self.list_ty.size_of(self.module).unwrap();
                let size = builder.const_int(self.i64, size as i64, false);

                let one = builder.const_int(self.i64, 1, false);
                let strct = builder.struct_instance(self.list_ty, [*one, *len, *len, *buf]);
                let buf = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]);

                builder.store(buf.as_ptr(), *strct);

                buf
            },

            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                let value = self.expr(env, builder, expr_id)?;
                out_if_err!();


                let some = builder.const_int(self.i32, 0, false);
                let tag = builder.field_load(value.as_struct(), 0);

                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                builder.ite(&mut (), comp,
                |_, _| {}, 


                |builder, _| {
                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                }, 
                );


                let buf = builder.alloca_store(value);
                let field_ty = self.ty_info.expr(expr_id).unwrap();
                let gens = field_ty.gens(self.syms);
                let payload_ty = self.syms.get_gens(gens)[0].1;
                let payload_ty = payload_ty.resolve(&[env.gens], self.syms);
                let payload_llvm = self.to_llvm_ty(payload_ty);

                let data_ptr = builder.field_ptr(buf, value.ty().as_struct(), 1);
                builder.load(data_ptr, payload_llvm.repr)
            },


            parser::nodes::expr::Expr::OrReturn(expr_id) => {
                let value = self.expr(env, builder, expr_id)?;
                out_if_err!();

                let some = builder.const_int(self.i32, 0, false);

                let tag = builder.field_load(value.as_struct(), 0);

                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                builder.ite(&mut (), comp,
                |_, _| {}, 


                |builder, _| {
                    self.drop_all_locals(env, builder);
                    builder.ret(value);
                }, 
                );

                let ty = self.ty_info.expr(expr_id).unwrap();
                let gens = ty.resolve(&[env.gens], self.syms).gens(self.syms);
                let gens = self.syms.get_gens(gens);

                let value_ty = gens[0].1;
                let value_llvm = self.to_llvm_ty(value_ty);

                let buf = builder.alloca_store(value);
                let data_ptr = builder.field_ptr(buf, value.ty().as_struct(), 1);
                let payload = builder.load(data_ptr, value_llvm.repr);

                payload
            },
        }, match ty.unwrap_or(crate::ExprInfo::Errored(ErrorId::Bypass)) {
            crate::ExprInfo::Result { ty } => ty,
            crate::ExprInfo::Errored(e) => return Err(e),
        }))
    }


    /// expects the top of the stack to be the value
    fn resolve_pattern(
        &mut self,
        env: &mut Env, builder: &mut Builder<'ctx>,
        ty: Type, _sym: TypeMapping<'ctx>, value: Value<'ctx>, pattern: Pattern,
    ) {
        match pattern.kind() {
            PatternKind::Variable(name) => {
                let local = builder.local(value.ty());

                env.alloc_var(name, local, ty, false);

                builder.local_set(local, value);
            },


            PatternKind::Tuple(items) => {
                let value = value.as_struct();

                let sym_id = ty.sym(self.syms).unwrap();
                let sym_data = self.syms.sym(sym_id);
                let SymbolKind::Container(cont) = sym_data.kind()
                else { unreachable!() };
                let gens = self.syms.get_gens(ty.gens(self.syms));

                for (i, &item) in items.iter().enumerate() {
                    let field = builder.field_load(value, i);
                    let local = builder.local(field.ty());

                    let field_ty = cont.fields()[i].1.to_ty(gens, self.syms).unwrap();
                    let field_ty = field_ty.resolve(&[], self.syms);

                    env.alloc_var(item, local, field_ty, false);
                    builder.local_set(local, field);
                }
            },
        }
    }


    fn create_struct(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Type,
        values: &[(StringIndex, Value<'ctx>)]
    ) -> Struct<'ctx> {
        let sym_id = ty.sym(self.syms).unwrap();
        let sym = self.syms.sym(sym_id);
        let SymbolKind::Container(cont) = sym.kind()
        else { unreachable!("type is not a container") };

        assert_eq!(values.len(), cont.fields().len());
        assert!(matches!(cont.kind(), ContainerKind::Struct | ContainerKind::Tuple));

        let ty = self.to_llvm_ty(ty);

        builder.struct_instance(
            ty.strct.as_struct(), 
            cont.fields().iter().map(|(field_name, _)| {
                let value = values.iter().find(|x| x.0 == *field_name).unwrap();
                value.1
            })
        )
    }


    fn create_enum(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Type,
        kind: Value<'ctx>,
        value: Value<'ctx>,
    ) -> Value<'ctx> {
        assert_eq!(kind.ty().kind(), TypeKind::Integer);

        let tag_val = builder.int_cast(kind.as_integer(), *self.i32, false);
        let llvm_ty = self.to_llvm_ty(ty);

        let buf = builder.alloca(llvm_ty.repr);
        let strct_ty = llvm_ty.strct.as_struct();

        let tag_ptr = builder.field_ptr(buf, strct_ty, 0);
        if value.ty().size_of(self.module).unwrap_or(1) == 0 {
            let zero = builder.const_zero(llvm_ty.repr);
            builder.store(buf, zero);
            builder.store(tag_ptr, tag_val);
        } else {
            builder.store(tag_ptr, tag_val);
            let data_ptr = builder.field_ptr(buf, strct_ty, 1);
            builder.store(data_ptr, value);
        }

        builder.load(buf, llvm_ty.repr)
    }


    fn create_enum_from_llvm(
        &self,
        builder: &mut Builder<'ctx>,
        tag: Value<'ctx>,
        value: Value<'ctx>,
        llvm_ty: TypeMapping<'ctx>,
    ) -> Value<'ctx> {
        let tag_val = builder.int_cast(tag.as_integer(), *self.i32, false);

        let strct_ty = llvm_ty.strct.as_struct();
        let buf = builder.alloca(llvm_ty.repr);

        let tag_ptr = builder.field_ptr(buf, strct_ty, 0);
        if value.ty().size_of(self.module).unwrap_or(1) == 0 {
            let zero = builder.const_zero(llvm_ty.repr);
            builder.store(buf, zero);
            builder.store(tag_ptr, tag_val);
        } else {
            builder.store(tag_ptr, tag_val);
            let data_ptr = builder.field_ptr(buf, strct_ty, 1);
            builder.store(data_ptr, value);
        }

        builder.load(buf, llvm_ty.repr)
    }


    fn emit_copy(&mut self, builder: &mut Builder<'ctx>, value: Value<'ctx>, ty: Type) -> Value<'ctx> {
        let Ok(sym_id) = ty.sym(&self.syms) else {
            return value;
        };

        if sym_id == SymbolId::RC {
            return builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[value]);
        }

        if sym_id == SymbolId::LIST {
            return builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[value]);
        }

        if sym_id == SymbolId::UNIT {
            return value;
        }

        let sym_data = self.syms.sym(sym_id);
        let SymbolKind::Container(cont) = sym_data.kind()
        else {
            if matches!(sym_data.kind(), SymbolKind::Function(_)) {
                let func_ref = value.as_struct();
                let capture_ptr = builder.field_load(func_ref, 1).as_ptr();
                let capture_int = builder.bitcast(*capture_ptr, *self.i64).as_integer();
                let null_int = builder.const_int(self.i64, 0, false);
                let is_closure = builder.cmp_int(capture_int, null_int, IntCmp::Ne);
                builder.ite(&mut (), is_closure,
                    |builder, _| {
                        builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[*capture_ptr]);
                    },
                    |_, _| {},
                );
                return value;
            }
            return value;
        };

        if cont.fields().is_empty() {
            return value;
        }

        let cont_gens = self.syms.get_gens(ty.gens(&self.syms));
        let llvm_ty = self.to_llvm_ty(ty);

        match cont.kind() {
            ContainerKind::Struct | ContainerKind::Tuple => {
                let struct_val = value.as_struct();
                let mut fields = Vec::with_capacity(cont.fields().len());
                for (i, (_, field_gen)) in cont.fields().iter().enumerate() {
                    let Ok(field_ty) = field_gen.to_ty(cont_gens, self.syms) else { continue };
                    let field_ty = field_ty.resolve(&[], self.syms);
                    let field_val = builder.field_load(struct_val, i);
                    let copied = self.emit_copy(builder, field_val, field_ty);
                    fields.push(copied);
                }
                *builder.struct_instance(llvm_ty.strct.as_struct(), fields)
            },

            ContainerKind::Enum => {
                let tag = builder.field_load(value.as_struct(), 0);
                let buf = builder.alloca(llvm_ty.repr);
                let src_buf = builder.alloca_store(value);
                let src_data_ptr = builder.field_ptr(src_buf, llvm_ty.strct.as_struct(), 1);
                let dst_data_ptr = builder.field_ptr(buf, llvm_ty.strct.as_struct(), 1);

                for (i, (_, field_gen)) in cont.fields().iter().enumerate() {
                    let Ok(field_ty) = field_gen.to_ty(cont_gens, self.syms) else { continue };
                    let field_ty = field_ty.resolve(&[], self.syms);
                    if field_ty.sym(&self.syms) == Ok(SymbolId::UNIT) {
                        continue;
                    }
                    let field_llvm = self.to_llvm_ty(field_ty);

                    let index = builder.const_int(self.i32, i as _, false);
                    let cond = builder.cmp_int(tag.as_integer(), index, IntCmp::Eq);

                    builder.ite(
                        &mut (),
                        cond,
                        |builder, _| {
                            let payload = builder.load(src_data_ptr, field_llvm.repr);
                            let copied = self.emit_copy(builder, payload, field_ty);
                            builder.store(dst_data_ptr, copied);
                        },
                        |_, _| {},
                    );
                }

                let tag_ptr = builder.field_ptr(buf, llvm_ty.strct.as_struct(), 0);
                builder.store(tag_ptr, tag);
                builder.load(buf, llvm_ty.repr)
            },

            ContainerKind::Generic => unreachable!(),
        }
    }


    fn emit_drop(&mut self, builder: &mut Builder<'ctx>, value: Value<'ctx>, ty: Type) {
        let Ok(sym_id) = ty.sym(&self.syms) else {
            return;
        };

        if sym_id == SymbolId::RC {
            let gens_id = ty.gens(&self.syms);
            let gens = self.syms.get_gens(gens_id);
            let elem_ty = gens[0].1;
            let llvm_elem = self.to_llvm_ty(elem_ty);
            let elem_size = llvm_elem.repr.size_of(self.module).unwrap();
            let total_size = 8 + elem_size;
            let size_val = builder.const_int(self.i64, total_size as i64, false);

            let ptr = value.as_ptr();

            let rc_ptr = builder.gep(ptr, *self.i64, builder.const_int(self.i64, 0, false));
            let rc = builder.load(rc_ptr, *self.i64).as_integer();
            let one = builder.const_int(self.i64, 1, false);
            let new_rc = builder.sub_int(rc, one);
            builder.store(rc_ptr, *new_rc);

            let zero = builder.const_int(self.i64, 0, false);
            let is_zero = builder.cmp_int(new_rc, zero, IntCmp::Eq);

            builder.ite(&mut () as &mut (), is_zero,
            |builder, _| {
                let offset = builder.const_int(self.i64, 1, false);
                let data_ptr = builder.gep(ptr, *self.i64, offset);
                let data = builder.load(data_ptr, llvm_elem.repr);

                let _ = self.call_trait_method(
                    builder, elem_ty,
                    SymbolId::DESTROY_TRAIT, StringMap::DESTROY_FUNC,
                    &[data],
                );

                self.emit_drop(builder, data, elem_ty);
                builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*ptr, *size_val]);
            },
            |_, _| {});

            return;
        }

        if sym_id == SymbolId::LIST {
            let gens_id = ty.gens(&self.syms);
            let gens = self.syms.get_gens(gens_id);
            let elem_ty = gens[0].1;
            let llvm_elem = self.to_llvm_ty(elem_ty);

            let header = builder.load(value.as_ptr(), *self.list_ty).as_struct();

            let refcount = builder.field_load(header, 0).as_integer();
            let one = builder.const_int(self.i64, 1, false);
            let new_rc = builder.sub_int(refcount, one);

            let rc_ptr = builder.field_ptr(value.as_ptr(), self.list_ty, 0);
            builder.store(rc_ptr, *new_rc);

            let zero = builder.const_int(self.i64, 0, false);
            let is_zero = builder.cmp_int(new_rc, zero, IntCmp::Eq);

            builder.ite(&mut (), is_zero, 
            |builder, _| {
                let len = builder.field_load(header, 1).as_integer();
                let cap = builder.field_load(header, 2).as_integer();
                let data = builder.field_load(header, 3).as_ptr();

                let counter = builder.alloca(*self.i64);
                builder.store(counter, *len);

                let one = builder.const_int(self.i64, 1, false);
                let zero_val = builder.const_int(self.i64, 0, false);

                builder.loop_indefinitely(|builder, l| {
                    let i = builder.load(counter, *self.i64).as_integer();
                    let i = builder.sub_int(i, one);
                    builder.store(counter, *i);

                    let done = builder.cmp_int(i, zero_val, IntCmp::SignedLt);
                    builder.ite(&mut () as &mut (), done,
                        |builder, _| { builder.loop_break(l); },
                        |builder, _| {
                            let i_casted = builder.int_cast(i, *self.i32, false).as_integer();
                            let ptr = builder.gep(data, llvm_elem.repr, i_casted);
                            let elem = builder.load(ptr, llvm_elem.repr);
                            self.emit_drop(builder, elem, elem_ty);
                        },
                    );
                });

                let elem_size = llvm_elem.repr.size_of(self.module).unwrap() as i64;
                let elem_size_val = builder.const_int(self.i64, elem_size, false);
                let cap_i64 = builder.int_cast(cap, *self.i64, false).as_integer();
                let buf_size = builder.mul_int(cap_i64, elem_size_val);
                builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*data, *buf_size]);

                let header_size = self.list_ty.size_of(self.module).unwrap() as i64;
                let header_size_val = builder.const_int(self.i64, header_size, false);
                builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*value.as_ptr(), *header_size_val]);
            }, |_, _| {});

            return;
        }

        if sym_id == SymbolId::UNIT {
            return;
        }

        let sym_data = self.syms.sym(sym_id);
        let SymbolKind::Container(cont) = sym_data.kind()
        else {
            if matches!(sym_data.kind(), SymbolKind::Function(_)) {
                let func_ref = value.as_struct();
                let capture_ptr = builder.field_load(func_ref, 1).as_ptr();
                let capture_int = builder.bitcast(*capture_ptr, *self.i64).as_integer();
                let null_int = builder.const_int(self.i64, 0, false);
                let is_closure = builder.cmp_int(capture_int, null_int, IntCmp::Ne);
                builder.ite(&mut (), is_closure,
                    |builder, _| {
                        let gep = builder.gep(capture_ptr, *self.i64, builder.const_int(self.i64, 1, false));
                        let drop_fn_val = builder.load(gep, *self.ctx.ptr());
                        let drop_fn_ptr = drop_fn_val.as_func();
                        let void = self.ctx.void();
                        let drop_fn_ty = void.fn_ty(self.ctx.arena, &[*self.ctx.ptr()], false);
                        builder.call(drop_fn_ptr, drop_fn_ty, &[*capture_ptr]);
                    },
                    |_, _| {},
                );
                return;
            }
            return;
        };

        if cont.fields().is_empty() {
            return;
        }

        let cont_gens = self.syms.get_gens(ty.gens(&self.syms));
        let llvm_ty = self.to_llvm_ty(ty);

        match cont.kind() {
            ContainerKind::Struct | ContainerKind::Tuple => {
                let struct_val = value.as_struct();
                for i in (0..cont.fields().len()).rev() {
                    let (_, field_gen) = cont.fields()[i];
                    let Ok(field_ty) = field_gen.to_ty(cont_gens, self.syms) else { continue };
                    let field_ty = field_ty.resolve(&[], self.syms);
                    let field_val = builder.field_load(struct_val, i);
                    self.emit_drop(builder, field_val, field_ty);
                }
            },

            ContainerKind::Enum => {
                let tag = builder.field_load(value.as_struct(), 0);
                let src_buf = builder.alloca_store(value);
                let src_data_ptr = builder.field_ptr(src_buf, llvm_ty.strct.as_struct(), 1);

                for (i, (_, field_gen)) in cont.fields().iter().enumerate() {
                    let Ok(field_ty) = field_gen.to_ty(cont_gens, self.syms) else { continue };
                    let field_ty = field_ty.resolve(&[], self.syms);
                    if field_ty.sym(&self.syms) == Ok(SymbolId::UNIT) {
                        continue;
                    }
                    let field_llvm = self.to_llvm_ty(field_ty);

                    let index = builder.const_int(self.i32, i as _, false);
                    let cond = builder.cmp_int(tag.as_integer(), index, IntCmp::Eq);

                    builder.ite(
                        &mut (),
                        cond,
                        |builder, _| {
                            let payload = builder.load(src_data_ptr, field_llvm.repr);
                            self.emit_drop(builder, payload, field_ty);
                        },
                        |_, _| {},
                    );
                }
            },

            ContainerKind::Generic => unreachable!(),
        }
    }

    fn call_trait_method(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Type,
        trait_id: SymbolId,
        func_name: StringIndex,
        args: &[Value<'ctx>],
    ) -> Option<Value<'ctx>> {
        let sym = ty.sym(self.syms).ok()?;
        let ns = {
            let traits = self.syms.traits(sym);
            let (ns, _, _) = traits.get(&trait_id)?;
            *ns
        };

        let func_sym = self.ns.get_ns(ns).get_sym(func_name).unwrap().ok()?;
        let func = self.get_func(Type::Ty(func_sym, ty.gens(self.syms))).ok()?;

        let null = builder.ptr_null();
        let mut call_args = Vec::with_capacity(args.len() + 1);
        call_args.extend_from_slice(args);
        call_args.push(*null);

        Some(builder.call(func.func_ptr, func.func_ty, &call_args))
    }


    fn drop_locals(&mut self, env: &Env<'_, 'ctx>, builder: &mut Builder<'ctx>, start: usize) {
        for i in (start..env.vars.len()).rev() {
            let (_, local, ty, _) = env.vars[i];
            let value = builder.local_get(local);
            self.emit_drop(builder, value, ty);
        }
    }


    fn drop_all_locals(&mut self, env: &Env<'_, 'ctx>, builder: &mut Builder<'ctx>) {
        self.drop_locals(env, builder, 0);
    }


    fn eq(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Type,
        accum: Local,
        lhs: Value<'ctx>,
        rhs: Value<'ctx>,
    ) {

        assert!(ty.is_resolved(self.syms));

        let sym = ty.sym(self.syms).unwrap();
        let gens = ty.gens(self.syms);
        let gens = self.syms.get_gens(gens);

        match sym {
            SymbolId::I64 => {

                let a = builder.local_get(accum).as_bool();
                let b = builder.cmp_int(
                    lhs.as_integer(),
                    rhs.as_integer(), 
                    IntCmp::Eq
                );


                let result = builder.bool_and(a, b);
                builder.local_set(accum, *result);

            },


            SymbolId::F64 => {

                let a = builder.local_get(accum).as_bool();
                let b = builder.cmp_fp(
                    lhs.as_fp(),
                    rhs.as_fp(), 
                    FPCmp::Eq
                );


                let result = builder.bool_and(a, b);
                builder.local_set(accum, *result);


            },


            SymbolId::UNIT => {
                let val = builder.const_bool(true);
                builder.local_set(accum, *val);
            },


            _ => {
                let b = self.call_trait_method(
                    builder, ty, SymbolId::EQ_TRAIT, StringMap::EQ_FUNC, &[lhs, rhs],
                ).unwrap();

                let b = b.as_struct();
                let b = builder.field_load(b, 0).as_integer();
                let b = builder.int_cast(b, *self.ctx.bool(), false).as_bool();

                let a = builder.local_get(accum).as_bool();

                let result = builder.bool_and(a, b);
                builder.local_set(accum, *result);

            }
        }

    }


}





impl Env<'_, '_> {
    pub fn alloc_var(&mut self, name: StringIndex, local: Local, ty: Type, is_param: bool) {
        self.vars.push((name, local, ty, is_param));
    }


    pub fn find_var(&self, name: StringIndex) -> Option<Local> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.1)
    }


    pub fn find_var_ty(&self, name: StringIndex) -> Option<Type> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.2)
    }


    pub fn is_var_param(&self, name: StringIndex) -> bool {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.3).unwrap_or(false)
    }
}
