use std::{collections::HashMap, hash::Hash, os::unix::process::CommandExt, process::Command};

use common::{string_map::{StringIndex, StringMap}, Swap};
use errors::ErrorId;
use llvm_api::{builder::{Builder, FPCmp, IntCmp, Local, Loop}, ctx::{Context, ContextRef}, module::Module, tys::{func::FunctionType, integer::IntegerTy, strct::StructTy, Type, TypeKind}, values::{bool::Bool, func::{FunctionPtr, Linkage}, int::Integer, ptr::Ptr, strct::Struct, Value}};
use parser::nodes::{decl::Decl, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::StmtId, NodeId, Pattern, PatternKind, AST};
use sti::{hash::fxhash::{FxHasher32, FxHasher64}, static_assert};

use crate::{namespace::NamespaceMap, syms::{self, containers::ContainerKind, sym_map::{GenListId, SymbolId, SymbolMap}, ty::{Sym, TypeHash}, SymbolKind}, TyChecker, TyInfo};

pub struct Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,

    ty_info: &'me TyInfo,
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


    // ptr1 is a function ptr
    // ptr2 is the environment ptr
    func_ref: StructTy<'ctx>,

    // tag is a u32
    // ptr is a ptr
    enum_ref: StructTy<'ctx>,

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
    repr: Type<'ctx>,


    /// this is either a native representation for stuff like primitives
    /// or the struct type for user types
    strct: Type<'ctx>


}


struct Function<'ctx> {
    sym: Sym,

    name: StringIndex,

    kind: FunctionKind,
    error: Option<ErrorId>,

    func_ty: FunctionType<'ctx>,
    func_ptr: FunctionPtr<'ctx>,
}


enum FunctionKind {
    Code,
    Extern(StringIndex),
}


struct Env<'a, 'ctx> {
    vars: Vec<(StringIndex, Local)>,
    loop_id: Option<Loop>,
    gens: &'a [(StringIndex, Sym)],
    info: HashMap<ExprId, Value<'ctx>>,
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
    string_map: &mut StringMap, syms: &mut SymbolMap, nss: &mut NamespaceMap,
    ast: &mut AST<'a>, ty_info: &mut TyInfo, errors: [Vec<Vec<String>>; 3], file_count: u32, startups: &[SymbolId],
) {
    println!("running llvm");

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
        let err_fn = module.function("margarineError", abort_fn_ty);
        err_fn.set_linkage(Linkage::External);
        err_fn.set_noreturn(ctx.as_ctx_ref());

        let ptr = ctx.ptr();
        let i32_ty = ctx.integer(32);
        let alloc_fn_ty = ptr.fn_ty(ctx.arena, &[*ctx.integer(64)], false);
        let alloc_fn = module.function("margarineAlloc", alloc_fn_ty);
        alloc_fn.set_linkage(Linkage::External);

        let func_ref = ctx.structure("funcRef");
        func_ref.set_fields(&[*ctx.ptr(), *ctx.ptr()], false);

        let enum_ref = ctx.structure("enumRef");
        enum_ref.set_fields(&[*ctx.ptr(), *i32_ty], false);


        let any_ref = ctx.structure("anyType");
        any_ref.set_fields(&[*ctx.ptr(), *i32_ty], false);


        let list_ty = ctx.structure("listType");
        list_ty.set_fields(&[*i32_ty, *i32_ty, *ctx.ptr()], false);

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
            i32: i32_ty,
            i64: ctx.integer(64),
            ctx: ctx.as_ctx_ref(),
            func_ref,
            enum_ref,
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
                    conv.ty_mappings.insert(Sym::$enum.hash(conv.syms), TypeMapping { repr: *val, strct: *val });
                }};
            }


            register!(I64, ctx.integer(64));
            register!(F64, ctx.f64());
            register!(UNIT, ctx.unit());
        }


        // create IR
        for sym in startups.iter() {
            let _ = conv.get_func(Sym::Ty(*sym, GenListId::EMPTY));
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

    println!("{:?}",
        Command::new("llc")
            .arg("-filetype=obj")
            .arg("out.ll")
            .arg("-o=program.o")
            .output()
    );

    println!("{:?}",
        Command::new("clang")
            .arg("-O3")
            .arg("program.o")
            .arg("libmargarine.a")
            .arg("-lzstd")
            .arg("-lz")
            .arg("-lc++")
            .arg("-lc++abi")
            .arg("-o")
            .arg("program")
            .output()
    );

    println!("{}",
        std::str::from_utf8(&Command::new("./program")
            .output()
            .unwrap()
            .stdout
        ).unwrap()
    );


}



impl<'me, 'out, 'ast, 'str, 'ctx> Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    fn get_func(&mut self, ty: Sym) -> Result<&Function<'ctx>, ErrorId> {
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
        let SymbolKind::Function(sym_func) = sym.kind()
        else { unreachable!() };

        let gens = self.syms.gens()[gens_id];

        assert_eq!(gens.len(), sym.generics().len());
        for ((g0, _), n1) in gens.iter().zip(sym.generics()) {
            assert_eq!(*g0, *n1);
        }

        let ret = sym_func.ret().to_ty(gens, self.syms).unwrap();
        let is_never = ret.is_never(self.syms);

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


                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);

                if is_never {
                    func_ptr.set_noreturn(self.ctx);
                }


                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
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
                };

                for (i, arg) in sym_func.args().iter().enumerate() {
                    env.alloc_var(arg.name(), builder.arg(i).unwrap());
                }

                let Decl::Function { body, .. } = self.ast.decl(sym_func.decl().unwrap())
                else { unreachable!() };


                let result = self.block(&mut env, &mut builder, &*body);
                
                if let Some(e) = self.ty_info.decl(sym_func.decl().unwrap()) {
                    self.error(&mut builder, e);
                }


                match result {
                    Ok(v) => {
                        if !is_never {
                            builder.ret(v);
                        }
                    },


                    Err(e) => {
                        self.error(&mut builder, e);
                    },
                }

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::TypeId => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &[],
                    false,
                );

                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
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

                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
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


            syms::func::FunctionKind::Any => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &llvm_args,
                    false,
                );

                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
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

                let strct = builder.struct_instance(self.any_ref, &[*ptr, *id]);
                builder.ret(*strct);

                return Ok(&self.funcs[&hash]);
            },



            syms::func::FunctionKind::DowncastAny => {
                let func_ty = llvm_ret.repr.fn_ty(
                    self.ctx.arena, 
                    &llvm_args,
                    false,
                );

                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
                    kind: FunctionKind::Code,
                    error: None,

                    func_ty,
                    func_ptr,
                };

                assert!(self.funcs.insert(hash, func).is_none());

                let builder = func_ptr.builder(self.ctx, func_ty);
                
                let target_sym = gens[0].1;
                let target_id = target_sym.sym(self.syms).unwrap();

                let target_id = builder.const_int(self.i32, target_id.0 as i64, false);

                let curr = builder.arg(0).unwrap();
                let curr = builder.local_get(curr).as_struct();
                let curr_id = builder.field_load(curr, 1).as_integer();

                let cmp = builder.cmp_int(target_id, curr_id, IntCmp::Ne);
                let cmp = builder.int_cast(cmp.as_integer(), *self.i32, false);
                let payload = builder.field_load(curr, 0);

                let buf = builder.struct_instance(self.enum_ref, &[payload, cmp]);
                builder.ret(*buf);

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


                let func_ptr = self.module.function(self.string_map.get(sym.name()), func_ty);


                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(ty.display(self.string_map, self.syms)),
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
                    let nul = builder.ptr_null();
                    self.create_enum(&mut builder, ret, *kind, *nul)
                } else {
                    let kind = builder.const_int(self.i32, index as _, false);
                    let value = builder.arg(0).unwrap();
                    let value = builder.local_get(value);
                    self.create_enum(&mut builder, ret, *kind, value)
                };

                builder.ret(*en);

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


    fn to_llvm_ty(&mut self, ty: Sym) -> TypeMapping<'ctx> {
        assert!(ty.is_resolved(self.syms));

        let hash = ty.hash(self.syms);
        
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let sym_id = ty.sym(self.syms).unwrap();

        if sym_id == SymbolId::ANY {
            self.ty_mappings.insert(hash, TypeMapping { repr: *self.any_ref, strct: *self.any_ref });
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
                      ContainerKind::Tuple
                    | ContainerKind::Struct => {
                        let strct = self.ctx.structure(name);
                        let mapping = TypeMapping { repr: *self.ctx.ptr(), strct: *strct };

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
                        let mapping = TypeMapping { repr: *self.enum_ref, strct: *self.enum_ref };

                        self.ty_mappings.insert(hash, mapping);
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
                    if self.ty_info.expr(expr_id).unwrap().is_never(self.syms) {
                        builder.unreachable();
                        has_ret = None;
                        break
                    }

                    has_ret = Some(result);
                },


                NodeId::Err(error_id) => {
                    env.vars.truncate(len);
                    return Err(error_id);
                },
            }
        }

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

                let ty = self.ty_info.expr(rhs).unwrap();
                let ty = ty.resolve(&[env.gens], self.syms);

                let ty = self.to_llvm_ty(ty);

                Self::resolve_pattern(env, builder, ty, value, pat);

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

                    let func = Sym::Ty(sym, iter_sym.gens(&self.syms));
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

                    let tag = builder.field_load(call_ret_value, 1).as_integer();
                    let none_case = builder.const_int(tag.as_integer().ty(), 1, false);
                    let cond = builder.cmp_int(tag, none_case, IntCmp::Eq);

                    builder.ite(&mut (), cond, 
                    |builder, _| {
                        builder.loop_break(l);
                    }, |_, _| {});

                    let value = builder.field_load(call_ret_value, 0).as_ptr();
                    let value = builder.load(value, iter_fn_binding_value_ty_llvm.repr);

                    Self::resolve_pattern(env, builder, iter_fn_binding_value_ty_llvm, value, binding);

                    let _ = self.block(env, builder, &*body);

                    env.loop_id = lo;
                    env.vars.pop();
                });

                Ok(())
            },
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
                builder.local_set(local, value)
            }


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                let strct = self.expr(env, builder, val).unwrap();

                let ty = self.ty_info.expr(val).unwrap();
                if ty.is_err(self.syms) { unreachable!() }

                let sym = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);
                let llvm_ty = self.to_llvm_ty(ty);

                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!() };

                let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                    let name = f.0;
                    field_name == name
                }).unwrap();


                match cont.kind() {
                      ContainerKind::Tuple
                    | ContainerKind::Struct => {
                        let field = builder.field_ptr(
                            strct.as_ptr(), 
                            llvm_ty.strct.as_struct(),
                            i as _
                        );

                        builder.store(field, value);
                    },


                    ContainerKind::Enum => unreachable!(),
                    ContainerKind::Generic => unreachable!(),
                }

            }


            parser::nodes::expr::Expr::Unwrap(expr) => {
                match self.ast.expr(expr) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        let enum_ref = builder.local_get(local).as_struct();

                        let ty = self.ty_info.expr(expr).unwrap();
                        if ty.is_err(self.syms) { unreachable!() }

                        // unwrap
                        let some = builder.const_int(self.i32, 0, false);
                        let tag = builder.field_load(enum_ref, 1);

                        let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                        builder.ite(&mut (), comp,
                        |_, _| {}, 


                        |builder, _| {
                            builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                        }, 
                        );


                        let value = self.create_enum_from_llvm(builder, tag, value);
                        builder.local_set(local, *value);
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        let ty = self.ty_info.expr(val).unwrap();
                        if ty.is_err(self.syms) { unreachable!() }

                        let sym = ty.sym(self.syms).unwrap();
                        let sym = self.syms.sym(sym);
                        let llvm_ty = self.to_llvm_ty(ty);

                        let SymbolKind::Container(cont) = sym.kind()
                        else { unreachable!() };

                        let (i, _) = cont.fields().iter().enumerate().find(|(_, f)| {
                            let name = f.0;
                            field_name == name
                        }).unwrap();


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                let strct = self.expr(env, builder, val).unwrap();

                                let field = builder.field_ptr(
                                    strct.as_ptr(), 
                                    llvm_ty.strct.as_struct(),
                                    i as _
                                );


                                let enum_ref = builder
                                    .load(field, *self.enum_ref).as_struct();

                                // unwrap
                                let some = builder.const_int(self.i32, 0, false);
                                let tag = builder.field_load(enum_ref, 1);

                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                                }, 
                                );


                                let value = self.create_enum_from_llvm(builder, tag, value);
                                builder.store(field, *value);
                            },


                            ContainerKind::Enum => {
                                let strct = self.expr(env, builder, val).unwrap();

                                let tag = builder.field_load(
                                    strct.as_struct(),
                                    1
                                );

                                let payload = builder.field_load(
                                    strct.as_struct(),
                                    0
                                );

                                // unwrap
                                let some = builder.const_int(self.i32, i as i64, false);
                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(&mut (), comp,
                                |_, _| {}, 


                                |builder, _| {
                                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                                }, 
                                );


                                builder.store(payload.as_ptr(), value);
                            },


                            ContainerKind::Generic => unreachable!(),
                        }


                    }
                    _ => (),
                }
            }


            _ => unreachable!(),
        }
    }


    fn expr(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, expr: ExprId
    ) -> Result<Value<'ctx>, ErrorId> {
        macro_rules! out_if_err {
            () => {{

                match self.ty_info.expr(expr) {
                    Ok(e) => e,
                    Err(e) => {
                        return Err(e);
                    },
               }
            }};
        }


        let val = self.ast.expr(expr);


        Ok(match val {
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
                        *self.create_enum(builder, Sym::BOOL, *kind, value)
                    },
                }
            },


            parser::nodes::expr::Expr::Paren(expr_id) => self.expr(env, builder, expr_id)?,


            parser::nodes::expr::Expr::Identifier(name, _) => {
                let ty = out_if_err!();

                let ty = ty.resolve(&[env.gens], self.syms);

                // it's a function
                if let Some(Some(func)) = self.ty_info.idents.get(&expr) {
                    let func_gens = ty.gens(self.syms);
                    let func = Sym::Ty(*func, func_gens);

                    let func = func.resolve(&[env.gens], self.syms);

                    let func = self.get_func(func).unwrap();

                    // create func ref
                    // we want a null ptr as the environment pointer
                    // since named-funcs have no captures we don't 
                    // need to allocate anything
                    let null = builder.ptr_null();
                    let ptr = func.func_ptr;
                    let ty = self.func_ref;
                    let func_ref = builder.struct_instance(
                        ty,
                        &[*ptr, *null],
                    );


                    return Ok(*func_ref)
                }


                builder.local_get(env.find_var(name).unwrap())
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

                self.create_struct(builder, Sym::RANGE, &fields)
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

                    *self.create_enum(
                        builder,
                        Sym::BOOL,
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
                        let value = builder.field_load(rhs.as_struct(), 1).as_integer();
                        let value = builder.int_cast(value.as_integer(), *self.ctx.bool(), false);
                        let value = builder.bool_not(value.as_bool());

                        let ptr = builder.alloca_store(rhs);
                        let ptr = builder.field_ptr(ptr, rhs.ty().as_struct(), 1);
                        builder.store(ptr, *value);

                        builder.load(ptr, rhs.ty())
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

                let tag = builder.field_load(cond.as_struct(), 1).as_integer();
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
                let sym = sym.sym(self.syms).unwrap();
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!() };

                let value_ty = val.as_struct();
                let value_tag = builder.field_load(value_ty, 1).as_integer();

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
                    let value_ptr = builder.field_load(val.as_struct(), 0).as_ptr();
                    let value = builder.load(value_ptr, field_ty_llvm.repr);
                    builder.local_set(local, value);

                    env.vars.push((mapping.binding(), local));

                    // run the body
                    let ret_val = self.expr(env, builder, mapping.expr());
                    debug_assert_eq!(env.vars.pop().unwrap(), (mapping.binding(), local));

                    let Ok(ret_val) = ret_val
                    else { return };

                    builder.local_set(ret_local, ret_val);
                });

                if ty.is_never(self.syms) { *builder.const_unit() }
                else { builder.local_get(ret_local) }
            },


            parser::nodes::expr::Expr::IndexList { list, index } => {
                let list_value = self.expr(env, builder, list)?;
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
                let len = builder.field_load(strct, 0).as_integer();
                let len = builder.int_cast(len, *self.i64, true).as_integer();

                let buf = builder.field_load(strct, 2).as_ptr();

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

                let ty = out_if_err!().resolve(&[env.gens], self.syms);

                for (name, _, e) in fields {
                    let value = self.expr(env, builder, *e)?;
                    values.push((*name, value));
                }

                self.create_struct(builder, ty, &*values)
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
                            let llvm_ty = self.to_llvm_ty(val);
                            let value = builder.load(value.as_ptr(), llvm_ty.strct).as_struct();
                            return Ok(builder.field_load(value, i as _))
                        },

                        ContainerKind::Enum => {
                            let ptr = builder.alloca_store(value);
                            let tag_ptr = builder.field_ptr(ptr, value.ty().as_struct(), 1);

                            let field = builder.load(tag_ptr, *self.i32).as_integer();
                            let index = builder.const_int(self.i32, i as _, false);
                            let cond = builder.cmp_int(field, index, IntCmp::Eq);


                            // variant 0 is Some(T)
                            // variant 1 is None
                            //
                            // we don't need to change the payload
                            builder.ite(
                                &mut (),
                                cond,
                                |builder, _| {
                                    let num = builder.const_int(self.i32, 0, false);
                                    builder.store(tag_ptr, *num);
                                },
                                |builder, _| {
                                    let num = builder.const_int(self.i32, 1, false);
                                    builder.store(tag_ptr, *num);
                                },
                            );

                            return Ok(builder.load(ptr, value.ty()))
                        },


                        ContainerKind::Generic => unreachable!(),
                    }

                }


                let sym_gens = self.syms.get_gens(val.gens(self.syms));

                let ns = 
                if let Some(tr) = self.ty_info.accesses.get(&expr) {
                    self.syms.traits(ty)[tr].0
                } else {
                    self.syms.sym_ns(ty)
                };


                let ns = self.ns.get_ns(ns);


                if let Some(sym) = ns.get_sym(field_name) {
                    let sym = sym.unwrap();
                    let gens = slf.gens(self.syms);

                    let sym = Sym::Ty(sym, gens)
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
                        &[*ptr, *null],
                    );

                    return Ok(*func_ref)
                }

                todo!()

            },


            parser::nodes::expr::Expr::CallFunction { lhs, args } => {
                let func = self.expr(env, builder, lhs)?;

                let mut llvm_args = sti::vec::Vec::with_cap_in(self.ctx.arena, args.len());
                for arg in args {
                    let arg = self.expr(env, builder, *arg)?;
                    llvm_args.push(arg);
                }

                out_if_err!();

                if let Expr::AccessField { .. } = self.ast.expr(lhs) {
                    llvm_args.insert(0, env.info[&lhs]);
                }


                let func_ty = self.ty_info.expr(lhs).unwrap();
                let func_ty = func_ty.resolve(&[env.gens], self.syms);
                let func_ty = self.to_llvm_ty(func_ty);

                let func_ptr = builder.field_load(func.as_struct(), 0);
                let capture_ptr = builder.field_load(func.as_struct(), 1);

                llvm_args.push(capture_ptr);
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

                let ty = ty.resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(ty);

                let captured = self.syms.closure(closure).captured_variables.clone();

                let mut hash = FxHasher64::new();
                for capture in &captured {
                    let ty = capture.1.resolve(&[env.gens], self.syms);
                    ty.hash(self.syms).hash(&mut hash);
                }

                let hash = ty.hash_fn(self.syms, |h| {
                    expr.hash(h);
                    hash.hash.hash(h);
                    self.funcs.len().hash(h);
                });


                let closure = self.syms.closure(closure);
                let mut tys = Vec::with_capacity(closure.captured_variables.len());
                let mut vals = Vec::with_capacity(closure.captured_variables.len());
                for name in &closure.captured_variables {
                    let index = env.find_var(name.0).unwrap();
                    let value = builder.local_get(index);
                    tys.push(value.ty());
                    vals.push(value);
                }
                
                let strct_ty = self.ctx.structure("captures");
                strct_ty.set_fields(&tys, false);

                let captures = builder.struct_instance(strct_ty, &vals);

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
                        gens: env.gens,
                        info: HashMap::new(),
                    };


                    let captured_ptr = builder.arg(func_ty.args().len() as _).unwrap();
                    let captured_ptr = builder.local_get(captured_ptr).as_ptr();
                    let captured_strct = builder.load(captured_ptr, *strct_ty).as_struct();

                    for (i, capture) in captured.iter().enumerate() {
                        let value = builder.field_load(captured_strct, i);
                        let local = builder.local(value.ty());
                        builder.local_set(local, value);
                        env.alloc_var(capture.0, local);
                    }


                    for (i, arg) in func_ty.args().iter().enumerate() {
                        env.alloc_var(arg.name(), builder.arg(i).unwrap());
                    }

                    let result = self.expr(&mut env, &mut builder, body);

                    match result {
                        Ok(v) => builder.ret(v),
                        Err(e) => self.error(&mut builder, e),
                    };
 
                    &self.funcs[&hash]
                };


                let func_ref = builder.struct_instance(self.func_ref, &[*func.func_ptr, *buf]);
                *func_ref
            },


              parser::nodes::expr::Expr::WithinNamespace { action, .. }
            | parser::nodes::expr::Expr::WithinTypeNamespace { action, .. } => {
                out_if_err!();
                self.expr(env, builder, action)?
            },


            parser::nodes::expr::Expr::Loop { body } => {
                let lid = env.loop_id;
                let mut value = Ok(());

                builder.loop_indefinitely(
                |builder, l| {
                    env.loop_id = Some(l);
                    let result = self.block(env, builder, &body);

                    if let Err(e) = result { value = Err(e) };
                });

                env.loop_id = lid;
                value?;
                out_if_err!();

                *builder.const_unit()
            },


            parser::nodes::expr::Expr::Return(expr_id) => {
                let val = self.expr(env, builder, expr_id)?;
                out_if_err!();

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


                self.create_struct(builder, ty, &llvm_exprs)
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
                    let tag = builder.field_load(lhs.as_struct(), 1);
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

                let list_ty = self.ty_info.expr(expr).unwrap();
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

                let strct = builder.struct_instance(self.list_ty, &[*len, *len, *buf]);
                let buf = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]);

                builder.store(buf.as_ptr(), *strct);

                buf
            },


            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                let value = self.expr(env, builder, expr_id)?;
                out_if_err!();


                let some = builder.const_int(self.i32, 0, false);
                let tag = builder.field_load(value.as_struct(), 1);

                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                builder.ite(&mut (), comp,
                |_, _| {}, 


                |builder, _| {
                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                }, 
                );

                let payload_ptr = builder.field_load(value.as_struct(), 0);
                let field_ty = self.ty_info.expr(expr_id).unwrap();
                let gens = field_ty.gens(self.syms);
                let payload_ty = self.syms.get_gens(gens)[0].1;
                let payload_ty = payload_ty.resolve(&[env.gens], self.syms);
                let payload_ty = self.to_llvm_ty(payload_ty);

                builder.load(payload_ptr.as_ptr(), payload_ty.repr)
            },


            parser::nodes::expr::Expr::OrReturn(expr_id) => {
                let value = self.expr(env, builder, expr_id)?;
                out_if_err!();

                let some = builder.const_int(self.i32, 0, false);

                let tag = builder.field_load(value.as_struct(), 1);
                let payload = builder.field_load(value.as_struct(), 0).as_ptr();

                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                builder.ite(&mut (), comp,
                |_, _| {}, 


                |builder, _| {
                    builder.ret(value);
                }, 
                );

                let ty = self.ty_info.expr(expr_id).unwrap();
                let gens = ty.resolve(&[env.gens], self.syms).gens(self.syms);
                let gens = self.syms.get_gens(gens);

                let value_ty = gens[0].1;
                let value_ty = self.to_llvm_ty(value_ty);

                let payload = builder.load(payload, value_ty.repr);

                payload
            },
        })
    }



    /// expects the top of the stack to be the value
    fn resolve_pattern(
        env: &mut Env, builder: &mut Builder<'ctx>,
        sym: TypeMapping<'ctx>, value: Value<'ctx>, pattern: Pattern,
    ) {
        match pattern.kind() {
            PatternKind::Variable(name) => {
                let local = builder.local(value.ty());
                env.alloc_var(name, local);

                builder.local_set(local, value);
            },


            PatternKind::Tuple(items) => {
                let value = builder.load(value.as_ptr(), sym.strct);
                let value = value.as_struct();

                for (i, &item) in items.iter().enumerate() {
                    let field = builder.field_load(value, i);
                    let local = builder.local(field.ty());

                    env.alloc_var(item, local);
                    builder.local_set(local, field);
                }
            },
        }
    }


    fn create_struct(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Sym,
        values: &[(StringIndex, Value<'ctx>)]
    ) -> Value<'ctx> {
        let sym_id = ty.sym(self.syms).unwrap();
        let sym = self.syms.sym(sym_id);
        let SymbolKind::Container(cont) = sym.kind()
        else { unreachable!("type is not a container") };

        assert_eq!(values.len(), cont.fields().len());
        assert!(matches!(cont.kind(), ContainerKind::Struct | ContainerKind::Tuple));

        let ty = self.to_llvm_ty(ty);
        assert_eq!(ty.repr.kind(), TypeKind::Ptr);

        let strct = ty.strct.as_struct();
        let size = strct.size_of(self.module).unwrap();

        // allocate the struct
        let size = builder.const_int(self.i64, size as i64, false);
        let ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]);
        let ptr = ptr.as_ptr();


        // fill in fields
        for (i, (field_name, _)) in cont.fields().iter().enumerate() {
            let field_ptr = builder.field_ptr(ptr, strct, i);
            let value = values.iter().find(|x| x.0 == *field_name).unwrap();
            builder.store(field_ptr, value.1);

        }

        *ptr
    }


    fn create_enum(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Sym,
        kind: Value<'ctx>,
        value: Value<'ctx>,
    ) -> Struct<'ctx> {
        assert_eq!(kind.ty().kind(), TypeKind::Integer);

        let sym_id = ty.sym(self.syms).unwrap();
        let sym = self.syms.sym(sym_id);
        let SymbolKind::Container(cont) = sym.kind()
        else { unreachable!("type is not a container") };

        assert!(matches!(cont.kind(), ContainerKind::Enum));

        let ty = self.to_llvm_ty(ty);
        assert_eq!(ty.repr.kind(), TypeKind::Struct);

        self.create_enum_from_llvm(builder, kind, value)
    }


    fn create_enum_from_llvm(
        &self,
        builder: &mut Builder<'ctx>,
        tag: Value<'ctx>,
        value: Value<'ctx>,
    ) -> Struct<'ctx> {
        let tag = builder.int_cast(tag.as_integer(), *self.i32, false);

        let size = value.ty().size_of(self.module).unwrap();
        let ptr = 
        if size > 0 {
            let size = builder.const_int(self.i64, size as i64, false);
            let ptr = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]);
            builder.store(ptr.as_ptr(), value);
            ptr
        } else {
            *builder.ptr_null()
        };
        
        let repr = {
            builder.struct_instance(
                self.enum_ref, 
                &[ptr, tag]
            )
        };

        repr
    }


    fn eq(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Sym,
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
                let sym = self.syms.sym(sym);
                match sym.kind() {
                    SymbolKind::Function(_) => {
                        todo!()
                    },


                    SymbolKind::Trait(_) => {
                        unreachable!()
                    }


                    SymbolKind::Container(container) => {
                        let ty = self.to_llvm_ty(ty);

                        assert_eq!(ty.strct.kind(), TypeKind::Struct);

                        match container.kind() {
                              ContainerKind::Struct
                            | ContainerKind::Tuple => {
                                assert_eq!(ty.repr.kind(), TypeKind::Ptr);
                                for (i, (_, field_ty)) in container.fields().iter().enumerate() {
                                    let field_ty = field_ty.to_ty(gens, self.syms).unwrap();

                                    let lhs = builder.field_ptr(lhs.as_ptr(), ty.strct.as_struct(), i);
                                    let lhs = builder.load(lhs, ty.strct);

                                    let rhs = builder.field_ptr(rhs.as_ptr(), ty.strct.as_struct(), i);
                                    let rhs = builder.load(rhs, ty.strct);

                                    self.eq(builder, field_ty, accum, lhs, rhs);
                                }
                            },


                            ContainerKind::Enum => {
                                let tag_lhs = builder.field_load(lhs.as_struct(), 1);
                                let tag_rhs = builder.field_load(rhs.as_struct(), 1);

                                let cmp = builder.cmp_int(tag_lhs.as_integer(), tag_rhs.as_integer(), IntCmp::Eq);

                                builder.ite(
                                &mut (),
                                cmp,
                                |builder, _| {
                                    // @todo
                                    builder.call(self.abort_fn.0, self.abort_fn.1, &[]);
                                },


                                |builder, _| {
                                    let c = builder.const_bool(false);
                                    builder.local_set(accum, *c);
                                });
                            },


                            ContainerKind::Generic => unreachable!(),
                        }
                    },


                    SymbolKind::Opaque => {
                        unreachable!()
                    },


                    SymbolKind::Namespace => unreachable!(),
                };

            }
        }

    }


}





impl Env<'_, '_> {
    pub fn alloc_var(&mut self, name: StringIndex, local: Local) {
        self.vars.push((name, local));
    }


    pub fn find_var(&self, name: StringIndex) -> Option<Local> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.1)
    }
}
