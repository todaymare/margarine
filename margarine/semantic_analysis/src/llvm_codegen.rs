use core::str;
use std::{collections::HashMap, fmt, hash::Hash, path::{Path, PathBuf}};

use common::{string_map::{StringIndex, StringMap}, Swap};
use errors::ErrorId;
use llvm_api::{builder::{Builder, FPCmp, IntCmp, Local, Loop}, ctx::{Context, ContextRef}, module::Module, tys::{func::FunctionType, integer::IntegerTy, strct::StructTy, Type as LLVMType, TypeKind}, values::{bool::Bool, func::{AllocKind, FunctionPtr, Linkage}, int::Integer, ptr::Ptr, strct::Struct, Value}};
use parser::nodes::{decl::Decl, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::StmtId, NodeId, Pattern, PatternKind, AST};
use sti::{arena::Arena, ext::FromIn, hash::fxhash::FxHasher64};

use crate::{namespace::NamespaceMap, syms::{self, containers::ContainerKind, sym_map::{BoundedGeneric, GenListId, Generic, SymbolId, SymbolMap}, ty::{Type, TypeHash}, SymbolKind}, TyInfo};

pub struct Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    string_map: &'me mut StringMap<'str>,
    syms: &'me mut SymbolMap<'out>,
    ns: &'me NamespaceMap,
    ast: &'me AST<'ast>,
    target: CompilationTarget,

    ty_info: &'me TyInfo<'out>,
    ty_mappings: HashMap<TypeHash, TypeMapping<'ctx>>,
    errors: [Vec<Vec<String>>; 3],

    externs: HashMap<StringIndex, (FunctionType<'ctx>, FunctionPtr<'ctx>, ExternAbi<'ctx>)>,
    funcs: HashMap<TypeHash, Function<'ctx>>,

    func_counter: u32,
    current_function_name: Option<StringIndex>,

    i32: IntegerTy<'ctx>,
    i64: IntegerTy<'ctx>,
    usize: IntegerTy<'ctx>,

    /// fn(bytes: ptr<byte>, length: int): !
    panic_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(size: usize): ptr
    alloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr, size: usize): void
    dealloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(total_size: usize): ptr
    rc_alloc_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr): ptr
    rc_clone_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),
    /// fn(ptr): bool
    rc_drop_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),

    /// fn(ptr): void
    assert_not_null_fn: (FunctionPtr<'ctx>, FunctionType<'ctx>),


    /// struct(*collection_header, length: usize)
    collection_ty: StructTy<'ctx>,
    /// struct(rc: usize)
    collection_header: StructTy<'ctx>,
    /// struct(collection_header)
    collection_flat_payload: StructTy<'ctx>,
    /// struct(collection_header, offset, collection_ty)
    collection_slice_payload: StructTy<'ctx>,
    /// struct(collection_header, collection_ty, collection_ty)
    collection_concat_payload: StructTy<'ctx>,


    collection_iter: StructTy<'ctx>,
    collection_iter_frame: StructTy<'ctx>,
    collection_iter_new: (FunctionPtr<'ctx>, FunctionType<'ctx>),

    /// Sibling TBAA tags for header payload vs collection elements.
    tbaa_header: Value<'ctx>,
    tbaa_element: Value<'ctx>,

    collection_drop_funcs: HashMap<(LLVMType<'ctx>, Option<TypeHash>), (FunctionPtr<'ctx>, FunctionType<'ctx>)>,
    collection_element_ptr_funcs: HashMap<LLVMType<'ctx>, (FunctionPtr<'ctx>, FunctionType<'ctx>)>,
    collection_iter_next_funcs: HashMap<LLVMType<'ctx>, (FunctionPtr<'ctx>, FunctionType<'ctx>)>,
    collection_flatten_funcs: HashMap<(LLVMType<'ctx>, Option<TypeHash>), (FunctionPtr<'ctx>, FunctionType<'ctx>)>,

    // ptr1 is a function ptr
    // ptr2 is the environment ptr
    func_ref: StructTy<'ctx>,


    str_ty: StructTy<'ctx>,

    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,

}


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
#[allow(unused)]
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
    Extern,
}


#[derive(Debug, Clone, Copy)]
enum ExternAbi<'ctx> {
    Direct,
    SRet(LLVMType<'ctx>),
}


#[derive(Debug, Clone)]
pub struct CompilationSettings<'out> {
    pub compilation_target: CompilationTarget,
    pub preludes: Vec<Prelude>,
    pub entry: String,
    pub output: String,
    pub cache: PathBuf,
    pub arena: &'out Arena,
    pub tests: bool,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompilationTarget {
    Arm64AppleDarwin,
    X86_64UnknownLinuxGnu,
    Aarch64UnknownLinuxGnu,
    Wasm32UnknownUnknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnsupportedCompilationTarget {
    pub target: String,
}

impl fmt::Display for UnsupportedCompilationTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "unsupported compilation target: {}", self.target)
    }
}

impl std::error::Error for UnsupportedCompilationTarget {}


#[derive(Debug, Clone)]
pub struct Prelude {
    pub alias: String,
    pub url: String,
}


impl TryFrom<&str> for CompilationTarget {
    type Error = UnsupportedCompilationTarget;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "default" => Ok(Self::host()),
            "arm64-apple-darwin" => Ok(CompilationTarget::Arm64AppleDarwin),
            "x86_64-unknown-linux-gnu" => Ok(CompilationTarget::X86_64UnknownLinuxGnu),
            "aarch64-unknown-linux-gnu" => Ok(CompilationTarget::Aarch64UnknownLinuxGnu),
            "wasm32-unknown-unknown" => Ok(CompilationTarget::Wasm32UnknownUnknown),
            value => Err(UnsupportedCompilationTarget {
                target: value.to_owned(),
            }),
        }
    }
}

impl TryFrom<String> for CompilationTarget {
    type Error = UnsupportedCompilationTarget;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}


impl std::str::FromStr for CompilationTarget {
    type Err = UnsupportedCompilationTarget;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::try_from(value)
    }
}


impl CompilationTarget {
    pub fn host() -> Self {
        if cfg!(all(target_arch = "x86_64", target_os = "linux")) {
            CompilationTarget::X86_64UnknownLinuxGnu
        } else if cfg!(all(target_arch = "aarch64", target_os = "linux")) {
            CompilationTarget::Aarch64UnknownLinuxGnu
        } else {
            CompilationTarget::Arm64AppleDarwin
        }
    }

    pub fn output_suffix(self) -> String {
        match self {
            CompilationTarget::Arm64AppleDarwin
            | CompilationTarget::X86_64UnknownLinuxGnu
            | CompilationTarget::Aarch64UnknownLinuxGnu => "".into(),
            CompilationTarget::Wasm32UnknownUnknown => "wasm".into(),
        }
    }

    pub fn shared_library_suffix(self) -> &'static str {
        match self {
            CompilationTarget::Arm64AppleDarwin => "dylib",
            CompilationTarget::X86_64UnknownLinuxGnu
            | CompilationTarget::Aarch64UnknownLinuxGnu => "so",
            CompilationTarget::Wasm32UnknownUnknown => "wasm",
        }
    }


    pub fn margarine_target_triple(self) -> String {
        match self {
            CompilationTarget::Arm64AppleDarwin => "arm64-apple-darwin".into(),
            CompilationTarget::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu".into(),
            CompilationTarget::Aarch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu".into(),
            CompilationTarget::Wasm32UnknownUnknown => "wasm32-unknown-unknown".into(),
        }
    }


    pub fn llvm_target_triple(self) -> String {
        match self {
            CompilationTarget::Arm64AppleDarwin => "arm64-apple-darwin".into(),
            CompilationTarget::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu".into(),
            CompilationTarget::Aarch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu".into(),
            CompilationTarget::Wasm32UnknownUnknown => "wasm32-unknown-unknown".into(),
        }
    }


    pub fn c_target_triple(self) -> String {
        match self {
            CompilationTarget::Arm64AppleDarwin => "aarch64-apple-darwin".into(),
            CompilationTarget::X86_64UnknownLinuxGnu => "x86_64-unknown-linux-gnu".into(),
            CompilationTarget::Aarch64UnknownLinuxGnu => "aarch64-unknown-linux-gnu".into(),
            CompilationTarget::Wasm32UnknownUnknown => "wasm32-unknown-unknown".into(),
        }
    }
}


#[derive(Default)]
struct Env<'a, 'ctx> {
    vars: Vec<(StringIndex, Local, Type, bool)>,
    inouts: Vec<(Local, Local)>,
    loop_id: Option<(Loop, usize)>,
    gens: &'a [(BoundedGeneric<'a>, Type)],
    info: HashMap<ExprId, Value<'ctx>>,
    ret_llvm_ty: Option<TypeMapping<'ctx>>,
}


pub fn run<'a>(
    string_map: &mut StringMap, syms: &mut SymbolMap<'a>, nss: &mut NamespaceMap,
    ast: &mut AST<'a>, ty_info: &mut TyInfo<'a>, errors: [Vec<Vec<String>>; 3], 
    _file_count: u32, startups: &[SymbolId], tests: &[SymbolId], settings: &CompilationSettings,
) {
    let target = settings.compilation_target;
    let ctx = Context::new(ast.arena, &target.llvm_target_triple());
    let mut module = ctx.module("margarine");

    let usize_ty = ctx.integer((module.ptr_size_in_bytes() * 8) as u32);
    
    // generate the code 
    {
        let void = ctx.void();

        let ptr = ctx.ptr();

        let collection_ty = ctx.structure("collectionType");
        collection_ty.set_fields(&[*ctx.ptr(), *ctx.integer(64)], false);


        // Sibling TBAA: RC word, header payload, and collection elements
        // do not alias even when they share one allocation.
        let tbaa_root = ctx.tbaa_root("margarine");
        let tbaa_refcount = ctx.tbaa_tag(ctx.tbaa_type("runtime.refcount", tbaa_root));
        let tbaa_header = ctx.tbaa_tag(ctx.tbaa_type("collection.header", tbaa_root));
        let tbaa_element = ctx.tbaa_tag(ctx.tbaa_type("collection.element", tbaa_root));

        let str_ty = ctx.structure("strType");
        str_ty.set_fields(&[*collection_ty], false);

        // The error type: a concrete zero-size struct so containers that hold
        // an errored field still have valid layouts. It carries no payload;
        // errored values are skipped by out_if_err! before emission, so this
        // is the codegen totality net, not a runtime value.
        let error_ty = ctx.structure("{error}");
        error_ty.set_fields(&[], false);

        let panic_fn_ty = void.fn_ty(ctx.arena, &[*ptr, *ctx.integer(64)], false);
        let panic_fn = module.function("margarinePanic", panic_fn_ty);
        panic_fn.set_linkage(Linkage::External);
        panic_fn.set_noreturn(ctx.as_ctx_ref());

        let i32_ty = ctx.integer(32);
        let abort_fn_ty = void.fn_ty(ctx.arena, &[*i32_ty], false);
        let abort_fn = module.function("margarineAbort", abort_fn_ty);
        abort_fn.set_linkage(Linkage::External);
        abort_fn.set_noreturn(ctx.as_ctx_ref());

        let alloc_fn_ty = ptr.fn_ty(ctx.arena, &[*usize_ty], false);
        let alloc_fn = module.function("margarineAlloc", alloc_fn_ty);
        alloc_fn.set_linkage(Linkage::External);

        let dealloc_fn_ty = void.fn_ty(ctx.arena, &[*ctx.ptr(), *usize_ty], false);
        let dealloc_fn = module.function("margarineDealloc", dealloc_fn_ty);
        dealloc_fn.set_linkage(Linkage::External);
        alloc_fn.set_noalias_return(ctx.as_ctx_ref());
        alloc_fn.set_alloc_size(ctx.as_ctx_ref(), 0);
        alloc_fn.set_alloc_kind(ctx.as_ctx_ref(), AllocKind::AllocUninitialized);
        alloc_fn.set_malloc_family(ctx.as_ctx_ref());
        dealloc_fn.set_alloc_kind(ctx.as_ctx_ref(), AllocKind::Free);
        dealloc_fn.set_malloc_family(ctx.as_ctx_ref());

        let rc_alloc_fn_ty = ptr.fn_ty(ctx.arena, &[*usize_ty], false);
        let rc_alloc_fn = module.function("margarineRcAlloc", rc_alloc_fn_ty);
        rc_alloc_fn.set_linkage(Linkage::External);

        let rc_clone_fn_ty = ptr.fn_ty(ctx.arena, &[*ctx.ptr()], false);
        let rc_clone_fn = module.function("margarineRcClone", rc_clone_fn_ty);
        rc_clone_fn.set_linkage(Linkage::Internal);

        {
            let builder = rc_clone_fn.builder(ctx.as_ctx_ref(), rc_clone_fn_ty);
            let rc_ptr = builder.arg(0).unwrap();
            let rc_ptr = builder.local_get(rc_ptr).as_ptr();

            let one = builder.const_int(usize_ty, 1, false);
            let refcount = builder.load_tbaa(rc_ptr, *usize_ty, tbaa_refcount).as_integer();

            let max = builder.const_all_ones(usize_ty);
            // we can assume this bcs its impossible to have 
            // 2^size references without those values also existing in memory
            // which can not fit in a 2^size addressspace
            let is_lt_max = builder.cmp_int(refcount, max, IntCmp::UnsignedLt);
            let is_ge_one = builder.cmp_int(refcount, one, IntCmp::UnsignedGe);
            builder.assume(is_lt_max);
            builder.assume(is_ge_one);

            let refcount = builder.add_int(refcount, one);

            builder.store_tbaa(rc_ptr, *refcount, tbaa_refcount);
            builder.ret(*rc_ptr);
        }



        let rc_drop_fn_ty = ctx.bool().fn_ty(ctx.arena, &[*ctx.ptr()], false);
        let rc_drop_fn = module.function("margarineRcDrop", rc_drop_fn_ty);
        rc_drop_fn.set_linkage(Linkage::Internal);

        {
            let builder = rc_drop_fn.builder(ctx.as_ctx_ref(), rc_drop_fn_ty);
            let rc_ptr = builder.arg(0).unwrap();
            let rc_ptr = builder.local_get(rc_ptr).as_ptr();

            let one = builder.const_int(usize_ty, 1, false);
            let zero = builder.const_int(usize_ty, 0, false);

            let refcount = builder.load_tbaa(rc_ptr, *usize_ty, tbaa_refcount).as_integer();

            // if this is false, then it'd lead to a double-free
            let is_ge_one = builder.cmp_int(refcount, one, IntCmp::UnsignedGe);
            builder.assume(is_ge_one);

            let refcount = builder.sub_int(refcount, one);

            builder.store_tbaa(rc_ptr, *refcount, tbaa_refcount);

            let is_rc_zero = builder.cmp_int(refcount, zero, IntCmp::Eq);
            builder.ret(*is_rc_zero);
        }


        let assert_not_null_fn_ty = void.fn_ty(ctx.arena, &[*ctx.ptr()], false);
        let assert_not_null_fn = module.function("margarineAssertNotNull", assert_not_null_fn_ty);
        assert_not_null_fn.set_linkage(Linkage::External);

        let func_ref = ctx.structure("funcRef");
        func_ref.set_fields(&[*ctx.ptr(), *ctx.ptr()], false);


        let collection_header = ctx.structure("collectionHeader");
        collection_header.set_fields(&[*usize_ty], false);

        let collection_flat_payload = ctx.structure("collectionFlatHeader");
        collection_flat_payload.set_fields(&[*collection_header], false);

        let collection_slice_payload = ctx.structure("collectionSliceHeader");
        collection_slice_payload.set_fields(&[*collection_header, *ctx.integer(64), *collection_ty], false);

        let collection_concat_payload = ctx.structure("collectionConcatHeader");
        collection_concat_payload.set_fields(&[*collection_header, *collection_ty, *collection_ty], false);


        let collection_iter = ctx.structure("collectionIter");
        let collection_iter_frame = ctx.structure("collectionIterFrame");
        let collection_iter_frame_array =
            ctx.array(*collection_iter_frame, COLLECTION_ITER_FRAME_CAPACITY);
        // collectionIter(
        //     owned: collectionType,
        //     pending_count: i64,
        //     frames: [collectionIterFrame; COLLECTION_ITER_FRAME_CAPACITY],
        //     leaf_data: ptr,
        //     leaf_index: i64,
        //     leaf_end: i64,
        // )
        //
        // collectionIterFrame(
        //     tagged_collection: ptr,
        //     start: i64,
        //     end: i64,
        // )
        collection_iter_frame.set_fields(
            &[*ptr, *ctx.integer(64), *ctx.integer(64)],
            false,
        );
        collection_iter.set_fields(
            &[
                *collection_ty,
                *ctx.integer(64),
                *collection_iter_frame_array,
                *ptr,
                *ctx.integer(64), *ctx.integer(64),
            ],
            false,
        );

        let collection_iter_new_fn_ty = collection_iter.fn_ty(ctx.arena, &[*collection_ty], false);
        let collection_iter_new_fn = module.function("collectionIterNew", collection_iter_new_fn_ty);
        collection_iter_new_fn.set_linkage(Linkage::Internal);

        {
            let builder = collection_iter_new_fn.builder(ctx.as_ctx_ref(), collection_iter_new_fn_ty);
            let collection = builder.local_get(builder.arg(0).unwrap()).as_struct();
            let zero = builder.const_int(ctx.integer(64), 0, false);
            let one = builder.const_int(ctx.integer(64), 1, false);
            let length = {
                let metadata = builder.field_load(collection, 1).as_integer();
                let mask = builder.const_int(ctx.integer(64), COLLECTION_LENGTH_MASK, false);
                builder.and(metadata, mask)
            };

            let iter_slot = builder.alloca(*collection_iter);
            let owned_ptr = builder.field_ptr(iter_slot, collection_iter, 0);
            builder.store(owned_ptr, *collection);
            let pending_ptr = builder.field_ptr(iter_slot, collection_iter, 1);
            builder.store(pending_ptr, *one);

            let frames = builder.field_ptr(iter_slot, collection_iter, 2);
            let frame = builder.gep(frames, *collection_iter_frame, zero);
            let tagged_ptr = builder.field_load(collection, 0).as_ptr();
            let frame_value =
            builder.struct_instance(
                collection_iter_frame,
                [*tagged_ptr, *zero, *length],
            );
            builder.store(frame, *frame_value);

            let leaf_data_ptr = builder.field_ptr(iter_slot, collection_iter, 3);
            builder.store(leaf_data_ptr, *builder.ptr_null());
            let leaf_index_ptr = builder.field_ptr(iter_slot, collection_iter, 4);
            builder.store(leaf_index_ptr, *zero);
            let leaf_end_ptr = builder.field_ptr(iter_slot, collection_iter, 5);
            builder.store(leaf_end_ptr, *zero);

            builder.ret(builder.load(iter_slot, *collection_iter));
        }




        let mut conv = Conversion {
            string_map,
            target,
            syms,
            ns: nss,
            ast,
            ty_info,
            errors,
            funcs: HashMap::new(),
            externs: HashMap::new(),
            ty_mappings: HashMap::new(),
            func_counter: 0,
            current_function_name: None,
            panic_fn: (panic_fn, panic_fn_ty),
            alloc_fn: (alloc_fn, alloc_fn_ty),
            dealloc_fn: (dealloc_fn, dealloc_fn_ty),
            rc_alloc_fn: (rc_alloc_fn, rc_alloc_fn_ty),
            rc_clone_fn: (rc_clone_fn, rc_clone_fn_ty),
            rc_drop_fn: (rc_drop_fn, rc_drop_fn_ty),

            assert_not_null_fn: (assert_not_null_fn, assert_not_null_fn_ty),
            i32: i32_ty,
            i64: ctx.integer(64),
            usize: usize_ty,
            ctx: ctx.as_ctx_ref(),
            func_ref,
            module,
            collection_header,
            collection_ty,
            collection_flat_payload,
            collection_slice_payload,
            collection_concat_payload,
            collection_iter_frame,
            collection_iter,
            collection_iter_new: (collection_iter_new_fn, collection_iter_new_fn_ty),
            tbaa_header,
            tbaa_element,
            str_ty,
            collection_flatten_funcs: HashMap::new(),
            collection_drop_funcs: HashMap::new(),
            collection_element_ptr_funcs: HashMap::new(),
            collection_iter_next_funcs: HashMap::new(),
        };


        conv.externs.insert(conv.string_map.insert("margarineAlloc"), (alloc_fn_ty, alloc_fn, ExternAbi::Direct));
        conv.externs.insert(conv.string_map.insert("margarinePanic"), (panic_fn_ty, panic_fn, ExternAbi::Direct));


        // register primitives
        {
            macro_rules! register {
                ($enum: ident, $call: expr) => {{
                    let val = $call;
                    conv.ty_mappings.insert(Type::$enum.hash(conv.syms), TypeMapping { repr: *val, strct: *val });
                }};
            }


            register!(I64, ctx.integer(64));
            register!(BYTE, ctx.integer(8));
            register!(F64, ctx.f64());
            register!(UNIT, ctx.unit());
        }


        // Claim the entry symbol before any user function is emitted. Root-level
        // `fn main` is now named exactly "main" (root-relative paths), so it would
        // otherwise take the entry name and the linker would bind the process
        // entry to it, making the exit code whatever garbage `w0` held on return.
        let i32_ty = ctx.integer(32);
        let main_fn_ty = i32_ty.fn_ty(ctx.arena, &[], false);
        let main_fn = module.function("main", main_fn_ty);

        // create IR
        for sym in startups.iter() {
            let _ = conv.get_func(Type::Ty(*sym, GenListId::EMPTY));
        }

        for sym in tests.iter() {
            let _ = conv.get_func(Type::Ty(*sym, GenListId::EMPTY));
        }

        // build main
        let builder = main_fn.builder(ctx.as_ctx_ref(), main_fn_ty);

        for sym_id in startups {
            let hash = Type::Ty(*sym_id, GenListId::EMPTY).hash(&*conv.syms);
            if let Some(func) = conv.funcs.get(&hash) {
                let args: Vec<_> = func.func_ty.args().into_iter().map(|ty| builder.const_zero(ty)).collect();
                builder.call(func.func_ptr, func.func_ty, &args);
            }
        }

        builder.call(abort_fn, abort_fn_ty, &[*ctx.const_int(i32_ty, 0, false)]);
        builder.unreachable();

        module = conv.module;
    }


    module.validate()
        .unwrap_or_else(|error| panic!("generated invalid LLVM module: {error}"));
    module.optimize()
        .unwrap_or_else(|error| panic!("failed to optimize LLVM module: {error}"));

    if matches!(target, CompilationTarget::Wasm32UnknownUnknown) {
        let fuel_fn_ty = ctx.void().fn_ty(ctx.arena, &[], false);
        let fuel_fn = module.function("margarineConsumeFuel", fuel_fn_ty);

        fuel_fn.set_linkage(Linkage::External);
        module.instrument_basic_block_exits(ctx.as_ctx_ref(), fuel_fn, fuel_fn_ty);
        module.validate()
            .unwrap_or_else(|error| panic!("generated invalid LLVM module after fuel instrumentation: {error}"));
    }

    ctx.emit_bitcode(module, Path::new(&format!("{}.bc", settings.output)))
        .unwrap_or_else(|error| panic!("failed to emit bitcode: {error}"));
    ctx.emit_object(module, Path::new(&format!("{}.o", settings.output)))
        .unwrap_or_else(|error| panic!("failed to emit object file: {error}"));
}



const COLLECTION_LENGTH_BITS: u32 = 59;
const COLLECTION_DEPTH_SHIFT: u32 = COLLECTION_LENGTH_BITS;
const COLLECTION_LENGTH_MASK: i64 = (1i64 << COLLECTION_LENGTH_BITS) - 1;
const COLLECTION_DEPTH_MASK: i64 = (1i64 << (64 - COLLECTION_LENGTH_BITS)) - 1;
const COLLECTION_FLATTEN_DEPTH: i64 = 8;
const COLLECTION_ITER_FRAME_CAPACITY: usize = COLLECTION_FLATTEN_DEPTH as usize + 1;
const COLLECTION_FLAT_TAG : usize = 0b00;
const COLLECTION_SLICE_TAG : usize = 0b01;
const COLLECTION_CONCAT_TAG : usize = 0b10;
const COLLECTION_UNUSED_TAG : usize = 0b11;



impl<'me, 'out, 'ast, 'str, 'ctx> Conversion<'me, 'out, 'ast, 'str, 'ctx> {
    fn const_usize(&self, builder: &Builder<'ctx>, value: usize) -> Integer<'ctx> {
        builder.const_int(self.usize, value as i64, false)
    }


    fn int_to_usize(&mut self, builder: &mut Builder<'ctx>, value: Integer<'ctx>) -> Integer<'ctx> {
        let zero = builder.const_int(self.i64, 0, false);
        let negative = builder.cmp_int(value, zero, IntCmp::SignedLt);
        let out_of_range = if self.usize.bit_size() < 64 {
            let max = builder.const_int(self.i64, (1i64 << self.usize.bit_size()) - 1, false);
            builder.cmp_int(value, max, IntCmp::UnsignedGt)
        } else {
            builder.const_bool(false)
        };
        let invalid = unsafe { Bool::new(*builder.or(negative.as_integer(), out_of_range.as_integer())) };
        builder.iff(invalid, |builder| {
            self.emit_panic(builder, "integer does not fit the target address space");
        });
        builder.int_cast(value, *self.usize, false).as_integer()
    }


    fn collection_metadata(
        &self,
        builder: &Builder<'ctx>,
        length: Integer<'ctx>,
        depth: Integer<'ctx>,
    ) -> Integer<'ctx> {
        let length_mask = builder.const_int(self.i64, COLLECTION_LENGTH_MASK, false);
        let depth_mask = builder.const_int(self.i64, COLLECTION_DEPTH_MASK, false);
        let depth_shift = builder.const_int(self.i64, COLLECTION_DEPTH_SHIFT as _, false);
        let length = builder.and(length, length_mask);
        let depth = builder.and(depth, depth_mask);
        let depth = builder.shl(depth, depth_shift);
        builder.or(length, depth)
    }


    fn collection_length(
        &self, builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
    ) -> Integer<'ctx> {
        let metadata = builder.field_load(collection, 1).as_integer();
        let mask = builder.const_int(self.i64, COLLECTION_LENGTH_MASK, false);
        builder.and(metadata, mask)
    }


    fn collection_depth(
        &self, builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
    ) -> Integer<'ctx> {
        let metadata = builder.field_load(collection, 1).as_integer();
        let shift = builder.const_int(self.i64, COLLECTION_DEPTH_SHIFT as _, false);
        builder.shr(metadata, shift, false)
    }


    fn collection_tagged_ptr(
        &self, builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
    ) -> Ptr<'ctx> {
        let ptr = builder.field_load(collection, 0).as_ptr();
        ptr
    }


    fn collection_flat_header_type(&self, elem_ty: LLVMType<'ctx>) -> StructTy<'ctx> {
        let arr = self.ctx.array(elem_ty, 0);
        self.ctx.literal_struct(&[*self.collection_flat_payload, *arr], false)
    }


    fn collection_flat_allocation_size(
        &mut self, builder: &mut Builder<'ctx>,
        elem_ty: LLVMType<'ctx>, length: Integer<'ctx>
    ) -> Integer<'ctx> {
        let length = self.int_to_usize(builder, length);
        let payload_ty = self.collection_flat_header_type(elem_ty);
        let header_size = payload_ty.size_of(self.module).unwrap();
        let header_size = builder.const_int(self.usize, header_size as _, false);

        let elem_size = elem_ty.size_of(self.module).unwrap();
        let elem_size = builder.const_int(self.usize, elem_size as _, false);
        let payload_size = builder.mul_int(length, elem_size);

        builder.add_int(header_size, payload_size)
    }


    fn collection_iter_frame_ptr(
        &self,
        builder: &Builder<'ctx>,
        iter_ptr: Ptr<'ctx>,
        index: Integer<'ctx>,
    ) -> Ptr<'ctx> {
        let frames = builder.field_ptr(iter_ptr, self.collection_iter, 2);
        builder.gep(frames, *self.collection_iter_frame, index)
    }

    fn collection_iter_push_frame(
        &self,
        builder: &mut Builder<'ctx>,
        iter_ptr: Ptr<'ctx>,
        tagged_collection: Ptr<'ctx>,
        start: Integer<'ctx>,
        end: Integer<'ctx>,
    ) {
        let pending_ptr = builder.field_ptr(iter_ptr, self.collection_iter, 1);
        let pending = builder.load(pending_ptr, *self.i64).as_integer();
        let capacity = builder.const_int(self.i64, COLLECTION_ITER_FRAME_CAPACITY as i64, false);
        builder.assume(builder.cmp_int(pending, capacity, IntCmp::UnsignedLt));

        let frame = self.collection_iter_frame_ptr(builder, iter_ptr, pending);
        let value =
        builder.struct_instance(
            self.collection_iter_frame,
            [*tagged_collection, *start, *end],
        );
        builder.store(frame, *value);

        let one = builder.const_int(self.i64, 1, false);
        builder.store(pending_ptr, *builder.add_int_nuw(pending, one));
    }


    fn create_collection_iter(
        &mut self,
        elem_repr: LLVMType<'ctx>,
    ) -> (FunctionPtr<'ctx>, FunctionType<'ctx>) {
        if let Some(&(next_fn, next_fn_ty)) = self.collection_iter_next_funcs.get(&elem_repr) {
            return (next_fn, next_fn_ty);
        }

        let ptr = self.ctx.ptr();
        let next_fn_ty = ptr.fn_ty(self.ctx.arena, &[*ptr], false);
        let next_fn =
        self.module.function(
            &format!("<collection_iter_next>::{}", elem_repr.name()),
            next_fn_ty,
        );
        next_fn.set_linkage(Linkage::Internal);
        self.collection_iter_next_funcs.insert(elem_repr, (next_fn, next_fn_ty));

        let mut builder = next_fn.builder(self.ctx.as_ctx_ref(), next_fn_ty);
        let iter_ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
        let zero = builder.const_int(self.i64, 0, false);
        let one = builder.const_int(self.i64, 1, false);

        builder.loop_indefinitely(|builder, loop_id| {
            let leaf_index =
                builder.field_ptr_load(iter_ptr, self.collection_iter, 4).as_integer();
            let leaf_end =
                builder.field_ptr_load(iter_ptr, self.collection_iter, 5).as_integer();
            let has_leaf = builder.cmp_int(leaf_index, leaf_end, IntCmp::UnsignedLt);

            builder.ite(has_leaf,
                |builder| {
                    let leaf_data =
                        builder.field_ptr_load(iter_ptr, self.collection_iter, 3).as_ptr();
                    let result = builder.gep(leaf_data, elem_repr, leaf_index);
                    let next_index = builder.add_int_nuw(leaf_index, one);
                    builder.field_store(iter_ptr, self.collection_iter, 4, next_index);
                    builder.ret(*result);
                },
                |builder| {
                    let pending =
                        builder.field_ptr_load(iter_ptr, self.collection_iter, 1).as_integer();
                    let has_pending = builder.cmp_int(pending, zero, IntCmp::UnsignedGt);

                    builder.ite(has_pending,
                        |builder| {
                            let new_pending = builder.sub_int(pending, one);
                            builder.field_store(
                                iter_ptr,
                                self.collection_iter,
                                1,
                                new_pending,
                            );

                            let frame =
                                self.collection_iter_frame_ptr(builder, iter_ptr, new_pending);
                            let tagged_ptr =
                                builder.field_ptr_load(frame, self.collection_iter_frame, 0)
                                    .as_ptr();
                            let start = builder
                                .field_ptr_load(frame, self.collection_iter_frame, 1)
                                .as_integer();
                            let end = builder
                                .field_ptr_load(frame, self.collection_iter_frame, 2)
                                .as_integer();

                            let (header_ptr, kind) =
                                self.collection_split_tag(builder, tagged_ptr);

                            builder.assume(builder.cmp_int(start, end, IntCmp::UnsignedLe));

                            builder.switch(kind, 0..4, |builder, alloc_kind| {
                                match alloc_kind {
                                    COLLECTION_FLAT_TAG => {
                                        let payload_ty =
                                            self.collection_flat_header_type(elem_repr);
                                        let leaf_data =
                                            builder.field_ptr(header_ptr, payload_ty, 1);

                                        builder.field_store(
                                            iter_ptr,
                                            self.collection_iter,
                                            3,
                                            leaf_data,
                                        );
                                        builder.field_store(
                                            iter_ptr,
                                            self.collection_iter,
                                            4,
                                            start,
                                        );
                                        builder.field_store(
                                            iter_ptr,
                                            self.collection_iter,
                                            5,
                                            end,
                                        );
                                        builder.loop_continue(loop_id);
                                    }

                                    COLLECTION_SLICE_TAG => {
                                        let ty = self.collection_slice_payload;
                                        let offset = builder
                                            .field_ptr_load_tbaa(
                                                header_ptr,
                                                ty,
                                                1,
                                                self.tbaa_header,
                                            )
                                            .as_integer();
                                        let base = builder
                                            .field_ptr_load_tbaa(
                                                header_ptr,
                                                ty,
                                                2,
                                                self.tbaa_header,
                                            )
                                            .as_struct();
                                        let base_start = builder.add_int(start, offset);
                                        let base_end = builder.add_int(end, offset);
                                        let base_tagged_ptr =
                                            self.collection_tagged_ptr(builder, base);
                                        self.collection_iter_push_frame(
                                            builder,
                                            iter_ptr,
                                            base_tagged_ptr,
                                            base_start,
                                            base_end,
                                        );
                                        builder.loop_continue(loop_id);
                                    }

                                    COLLECTION_CONCAT_TAG => {
                                        let ty = self.collection_concat_payload;
                                        let left = builder
                                            .field_ptr_load_tbaa(
                                                header_ptr,
                                                ty,
                                                1,
                                                self.tbaa_header,
                                            )
                                            .as_struct();
                                        let right = builder
                                            .field_ptr_load_tbaa(
                                                header_ptr,
                                                ty,
                                                2,
                                                self.tbaa_header,
                                            )
                                            .as_struct();
                                        let left_tagged_ptr =
                                            self.collection_tagged_ptr(builder, left);
                                        let right_tagged_ptr =
                                            self.collection_tagged_ptr(builder, right);
                                        let left_length =
                                            self.collection_length(builder, left);
                                        let end_in_left = builder.cmp_int(
                                            end,
                                            left_length,
                                            IntCmp::UnsignedLe,
                                        );

                                        builder.ite(end_in_left,
                                            |builder| {
                                                self.collection_iter_push_frame(
                                                    builder,
                                                    iter_ptr,
                                                    left_tagged_ptr,
                                                    start,
                                                    end,
                                                );
                                            },
                                            |builder| {
                                                let start_in_right = builder.cmp_int(
                                                    start,
                                                    left_length,
                                                    IntCmp::UnsignedGe,
                                                );

                                                builder.ite(start_in_right,
                                                    |builder| {
                                                        let right_start =
                                                            builder.sub_int(start, left_length);
                                                        let right_end =
                                                            builder.sub_int(end, left_length);
                                                        self.collection_iter_push_frame(
                                                            builder,
                                                            iter_ptr,
                                                            right_tagged_ptr,
                                                            right_start,
                                                            right_end,
                                                        );
                                                    },
                                                    |builder| {
                                                        let right_end =
                                                            builder.sub_int(end, left_length);

                                                        self.collection_iter_push_frame(
                                                            builder,
                                                            iter_ptr,
                                                            right_tagged_ptr,
                                                            zero,
                                                            right_end,
                                                        );
                                                        self.collection_iter_push_frame(
                                                            builder,
                                                            iter_ptr,
                                                            left_tagged_ptr,
                                                            start,
                                                            left_length,
                                                        );
                                                    },
                                                );
                                            },
                                        );
                                        builder.loop_continue(loop_id);
                                    }

                                    COLLECTION_UNUSED_TAG => builder.unreachable(),
                                    _ => unreachable!(),
                                }
                            });
                        },
                        |builder| builder.ret(*builder.ptr_null()),
                    );
                },
            );
        });

        (next_fn, next_fn_ty)
    }


    /// IMPORTANT: the returned flat buffer is uninitialised
    /// returns:
    /// - collectionTy
    /// - ptr to buffer
    fn collection_flat(
        &mut self, builder: &mut Builder<'ctx>,
        length: Integer<'ctx>, elem_ty: LLVMType<'ctx>,
    ) -> (Struct<'ctx>, Ptr<'ctx>) {
        let payload_ty = self.collection_flat_header_type(elem_ty);
        let total_size = self.collection_flat_allocation_size(builder, elem_ty, length);
        let buf = builder.call(self.rc_alloc_fn.0, self.rc_alloc_fn.1, &[*total_size]).as_ptr();

        let data_ptr = builder.field_ptr(buf, payload_ty, 1);

        let tag = builder.const_int(self.usize, COLLECTION_FLAT_TAG as _, false);
        let tagged_ptr = self.collection_with_tag(builder, buf, tag);
        let zero = builder.const_int(self.i64, 0, false);
        let metadata = self.collection_metadata(builder, length, zero);

        let strct = builder.struct_instance(self.collection_ty, [*tagged_ptr, *metadata]);
        (strct, data_ptr)
    }

    /// Recursively copies a collection range into a flat destination.
    ///
    /// The input collection is borrowed. The caller remains responsible for
    /// dropping its owned collection reference after this function returns.
    fn collection_flatten(
        &mut self,
        builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
        start: Integer<'ctx>,
        end: Integer<'ctx>,
        destination: Ptr<'ctx>,
        output_index: Ptr<'ctx>,
        elem_repr: LLVMType<'ctx>,
        elem_ty: Option<Type>,
    ) {
        let key = (elem_repr, elem_ty.map(|ty| ty.hash(self.syms)));
        if let Some(&(func, func_ty)) = self.collection_flatten_funcs.get(&key) {
            builder.call(
                func,
                func_ty,
                &[*collection, *start, *end, *destination, *output_index],
            );
            return;
        }

        let void = self.ctx.void();
        let func_ty =
        void.fn_ty(
            self.ctx.arena,
            &[
                *self.collection_ty,
                *self.i64,
                *self.i64,
                *self.ctx.ptr(),
                *self.ctx.ptr(),
            ],
            false,
        );
        let name = elem_ty
            .map(|ty| ty.display(self.string_map, self.syms).to_string())
            .unwrap_or_else(|| elem_repr.name().to_string());
        let func = self.module.function(&format!("<collection_flatten>::{}", name), func_ty);
        func.set_linkage(Linkage::Internal);
        self.collection_flatten_funcs.insert(key, (func, func_ty));

        let mut flatten_builder = func.builder(self.ctx.as_ctx_ref(), func_ty);
        let flatten_collection = flatten_builder.local_get(flatten_builder.arg(0).unwrap()).as_struct();
        let flatten_start = flatten_builder.local_get(flatten_builder.arg(1).unwrap()).as_integer();
        let flatten_end = flatten_builder.local_get(flatten_builder.arg(2).unwrap()).as_integer();
        let flatten_destination = flatten_builder.local_get(flatten_builder.arg(3).unwrap()).as_ptr();
        let flatten_output_index = flatten_builder.local_get(flatten_builder.arg(4).unwrap()).as_ptr();

        let tagged_ptr = self.collection_tagged_ptr(&mut flatten_builder, flatten_collection);
        let (header_ptr, allocation_kind) =
            self.collection_split_tag(&mut flatten_builder, tagged_ptr);

        let collection_length = self.collection_length(&mut flatten_builder, flatten_collection);
        flatten_builder.assume(
            flatten_builder.cmp_int(flatten_start, flatten_end, IntCmp::UnsignedLe),
        );
        flatten_builder.assume(
            flatten_builder.cmp_int(flatten_end, collection_length, IntCmp::UnsignedLe),
        );

        flatten_builder.switch(allocation_kind, 0..4, |builder, kind| {
            match kind {
                COLLECTION_FLAT_TAG => {
                    let payload_ty = self.collection_flat_header_type(elem_repr);
                    let data_ptr = builder.field_ptr(header_ptr, payload_ty, 1);
                    let index_slot = builder.alloca(*self.i64);
                    builder.store(index_slot, *flatten_start);
                    let one = builder.const_int(self.i64, 1, false);

                    builder.loop_indefinitely(|builder, loop_id| {
                        let index = builder.load(index_slot, *self.i64).as_integer();
                        let done = builder.cmp_int(index, flatten_end, IntCmp::UnsignedGe);
                        builder.ite(done,
                            |builder| builder.loop_break(loop_id),
                            |builder| {
                                let source = builder.gep_inbounds(data_ptr, elem_repr, index);
                                let value = builder.load_tbaa(source, elem_repr, self.tbaa_element);
                                let value =
                                if let Some(elem_ty) = elem_ty {
                                    self.emit_copy(builder, value, elem_ty)
                                } else {
                                    value
                                };
                                let output = builder.load(flatten_output_index, *self.i64).as_integer();
                                let destination = builder.gep(flatten_destination, elem_repr, output);
                                builder.store_tbaa(destination, value, self.tbaa_element);
                                builder.store(flatten_output_index, *builder.add_int_nuw(output, one));
                                builder.store(index_slot, *builder.add_int_nuw(index, one));
                            },
                        );
                    });
                }

                COLLECTION_SLICE_TAG => {
                    let payload_ty = self.collection_slice_payload;
                    let offset = builder
                        .field_ptr_load_tbaa(header_ptr, payload_ty, 1, self.tbaa_header)
                        .as_integer();
                    let base = builder
                        .field_ptr_load_tbaa(header_ptr, payload_ty, 2, self.tbaa_header)
                        .as_struct();
                    let base_start = builder.add_int_nuw(flatten_start, offset);
                    let base_end = builder.add_int_nuw(flatten_end, offset);
                    builder.call(
                        func,
                        func_ty,
                        &[*base, *base_start, *base_end, *flatten_destination, *flatten_output_index],
                    );
                }

                COLLECTION_CONCAT_TAG => {
                    let payload_ty = self.collection_concat_payload;
                    let left = builder
                        .field_ptr_load_tbaa(header_ptr, payload_ty, 1, self.tbaa_header)
                        .as_struct();
                    let right = builder
                        .field_ptr_load_tbaa(header_ptr, payload_ty, 2, self.tbaa_header)
                        .as_struct();
                    let left_length = self.collection_length(builder, left);
                    let end_in_left = builder.cmp_int(flatten_end, left_length, IntCmp::UnsignedLe);

                    builder.ite(end_in_left,
                        |builder| {
                            builder.call(
                                func,
                                func_ty,
                                &[*left, *flatten_start, *flatten_end, *flatten_destination, *flatten_output_index],
                            );
                        },
                        |builder| {
                            let start_in_right =
                                builder.cmp_int(flatten_start, left_length, IntCmp::UnsignedGe);
                            builder.ite(start_in_right,
                                |builder| {
                                    let right_start = builder.sub_int(flatten_start, left_length);
                                    let right_end = builder.sub_int(flatten_end, left_length);
                                    builder.call(
                                        func,
                                        func_ty,
                                        &[*right, *right_start, *right_end, *flatten_destination, *flatten_output_index],
                                    );
                                },
                                |builder| {
                                    let right_end = builder.sub_int(flatten_end, left_length);
                                    let zero = builder.const_int(self.i64, 0, false);
                                    builder.call(
                                        func,
                                        func_ty,
                                        &[*left, *flatten_start, *left_length, *flatten_destination, *flatten_output_index],
                                    );
                                    builder.call(
                                        func,
                                        func_ty,
                                        &[*right, *zero, *right_end, *flatten_destination, *flatten_output_index],
                                    );
                                },
                            );
                        },
                    );
                }

                COLLECTION_UNUSED_TAG => builder.unreachable(),
                _ => unreachable!(),
            }
        });

        flatten_builder.ret_void();
        builder.call(
            func,
            func_ty,
            &[*collection, *start, *end, *destination, *output_index],
        );
    }



    /// IMPORTANT: base take ownership.
    ///
    /// Slicing a slice applies the extra offset to that slice's non-slice
    /// base. The intermediate slice is released through `collection_drop`
    /// after cloning the grandbase. Because every slice goes through this
    /// function, a slice base is never itself a slice.
    fn collection_flatten_range(
        &mut self,
        env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
        start: Integer<'ctx>,
        length: Integer<'ctx>,
        elem_repr: LLVMType<'ctx>,
        elem_ty: Option<Type>,
    ) -> Struct<'ctx> {
        let (flat, data) = self.collection_flat(builder, length, elem_repr);
        let output_index = builder.alloca(*self.i64);
        let zero = builder.const_int(self.i64, 0, false);
        builder.store(output_index, *zero);
        let end = builder.add_int_nuw(start, length);

        self.collection_flatten(
            builder,
            collection,
            start,
            end,
            data,
            output_index,
            elem_repr,
            elem_ty,
        );
        self.collection_drop(env, builder, collection, elem_repr, elem_ty);
        flat
    }




    fn collection_slice(
        &mut self,
        env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>,
        base: Struct<'ctx>,
        offset: Integer<'ctx>,
        length: Integer<'ctx>,
        elem_repr: LLVMType<'ctx>,
        elem_ty: Option<Type>,
    ) -> Struct<'ctx> {
        assert_eq!(base.ty(), self.collection_ty);

        let tagged_ptr = self.collection_tagged_ptr(builder, base);
        let (header_ptr, tag) = self.collection_split_tag(builder, tagged_ptr);
        let is_slice = builder.cmp_int(
            tag,
            self.const_usize(builder, COLLECTION_SLICE_TAG),
            IntCmp::Eq,
        );

        let base_slot = builder.alloca_store(*base);
        let offset_slot = builder.alloca_store(*offset);

        builder.iff(is_slice, |builder| {
                let ty = self.collection_slice_payload;
                let parent_offset = builder
                    .field_ptr_load_tbaa(header_ptr, ty, 1, self.tbaa_header)
                    .as_integer();
                let grandbase = builder
                    .field_ptr_load_tbaa(header_ptr, ty, 2, self.tbaa_header)
                    .as_struct();
                let current_offset = builder.load(offset_slot, *self.i64).as_integer();
                builder.store(offset_slot, *builder.add_int_nuw(current_offset, parent_offset));

                let grand_tagged = self.collection_tagged_ptr(builder, grandbase);
                let (grand_ptr, _) = self.collection_split_tag(builder, grand_tagged);
                builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[*grand_ptr]);
                builder.store(base_slot, *grandbase);
                self.collection_drop(env, builder, base, elem_repr, elem_ty);
            });

        let base = builder.load(base_slot, *self.collection_ty).as_struct();
        let offset = builder.load(offset_slot, *self.i64).as_integer();
        let one = builder.const_int(self.i64, 1, false);
        let base_depth = self.collection_depth(builder, base);
        let depth = builder.add_int_nuw(base_depth, one);
        let metadata = self.collection_metadata(builder, length, depth);
        let flatten_depth = builder.const_int(self.i64, COLLECTION_FLATTEN_DEPTH, false);
        let should_flatten = builder.cmp_int(depth, flatten_depth, IntCmp::UnsignedGt);
        let result_slot = builder.alloca(*self.collection_ty);

        let slice_ty = self.collection_slice_payload;
        let collection_ty = self.collection_ty;
        let usize_ty = self.usize;
        let ptr_ty = self.ctx.ptr();
        let tbaa_header = self.tbaa_header;
        let rc_alloc_fn = self.rc_alloc_fn;
        let total_size = self.const_usize(builder, slice_ty.size_of(self.module).unwrap());
        let slice_tag = self.const_usize(builder, COLLECTION_SLICE_TAG);

        builder.ite(should_flatten,
            |builder| {
                let flat =
                self.collection_flatten_range(
                    env,
                    builder,
                    base,
                    offset,
                    length,
                    elem_repr,
                    elem_ty,
                );
                builder.store(result_slot, *flat);
            },
            |builder| {
                let buf = builder.call(rc_alloc_fn.0, rc_alloc_fn.1, &[*total_size]).as_ptr();
                let offset_ptr = builder.field_ptr(buf, slice_ty, 1);
                builder.store_tbaa(offset_ptr, *offset, tbaa_header);

                let base_ptr = builder.field_ptr(buf, slice_ty, 2);
                builder.store_tbaa(base_ptr, *base, tbaa_header);

                let ptr_as_usize = builder.ptr_to_int(buf, usize_ty);
                let tagged =
                builder.int_to_ptr(
                    builder.or(ptr_as_usize, slice_tag),
                    ptr_ty,
                );
                let value = builder.struct_instance(collection_ty, [*tagged, *metadata]);
                builder.store(result_slot, *value);
            },
        );

        builder.load(result_slot, *collection_ty).as_struct()
    }


    /// IMPORTANT: a & b take ownership
    fn collection_concat(
        &mut self,
        env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>,
        a: Struct<'ctx>,
        b: Struct<'ctx>,
        elem_repr: LLVMType<'ctx>,
        elem_ty: Option<Type>,
    ) -> Struct<'ctx> {
        assert_eq!(a.ty(), self.collection_ty);
        assert_eq!(b.ty(), self.collection_ty);

        let len_a = self.collection_length(builder, a);
        let len_b = self.collection_length(builder, b);
        let length = builder.add_int_nuw(len_a, len_b);
        let left_depth = self.collection_depth(builder, a);
        let right_depth = self.collection_depth(builder, b);
        let depth =
            builder.add_int_nuw(
            builder.max_int(left_depth, right_depth),
            builder.const_int(self.i64, 1, false),
        );
        let metadata = self.collection_metadata(builder, length, depth);
        let flatten_depth = builder.const_int(self.i64, COLLECTION_FLATTEN_DEPTH, false);
        let should_flatten = builder.cmp_int(depth, flatten_depth, IntCmp::UnsignedGt);
        let result_slot = builder.alloca(*self.collection_ty);

        let concat_ty = self.collection_concat_payload;
        let collection_ty = self.collection_ty;
        let usize_ty = self.usize;
        let ptr_ty = self.ctx.ptr();
        let tbaa_header = self.tbaa_header;
        let rc_alloc_fn = self.rc_alloc_fn;
        let total_size = self.const_usize(builder, concat_ty.size_of(self.module).unwrap());
        let concat_tag = self.const_usize(builder, COLLECTION_CONCAT_TAG);

        builder.ite(should_flatten,
            |builder| {
                let left_length = self.collection_length(builder, a);
                let right_length = self.collection_length(builder, b);
                let length = builder.add_int_nuw(left_length, right_length);
                let (flat, data) = self.collection_flat(builder, length, elem_repr);
                let output_index = builder.alloca(*self.i64);
                let zero = builder.const_int(self.i64, 0, false);
                builder.store(output_index, *zero);

                self.collection_flatten(
                    builder,
                    a,
                    zero,
                    left_length,
                    data,
                    output_index,
                    elem_repr,
                    elem_ty,
                );
                self.collection_flatten(
                    builder,
                    b,
                    zero,
                    right_length,
                    data,
                    output_index,
                    elem_repr,
                    elem_ty,
                );
                self.collection_drop(env, builder, a, elem_repr, elem_ty);
                self.collection_drop(env, builder, b, elem_repr, elem_ty);
                builder.store(result_slot, *flat);
            },
            |builder| {
                let buf = builder.call(rc_alloc_fn.0, rc_alloc_fn.1, &[*total_size]).as_ptr();
                let left = builder.field_ptr(buf, concat_ty, 1);
                builder.store_tbaa(left, *a, tbaa_header);
                let right = builder.field_ptr(buf, concat_ty, 2);
                builder.store_tbaa(right, *b, tbaa_header);

                let ptr_as_usize = builder.ptr_to_int(buf, usize_ty);
                let tagged =
                builder.int_to_ptr(
                    builder.or(ptr_as_usize, concat_tag),
                    ptr_ty,
                );
                let value = builder.struct_instance(collection_ty, [*tagged, *metadata]);
                builder.store(result_slot, *value);
            },
        );

        builder.load(result_slot, *collection_ty).as_struct()
    }

    /// Resolves an already bounds-checked index to its flat leaf element.
    ///
    /// The collection is borrowed: this does not alter reference counts. The
    /// returned pointer remains valid only while the collection remains alive.
    fn collection_element_ptr(
        &mut self,
        builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
        index: Integer<'ctx>,
        elem_repr: LLVMType<'ctx>,
    ) -> Ptr<'ctx> {
        assert_eq!(collection.ty(), self.collection_ty);
        assert_eq!(*index.ty(), *self.i64);

        if let Some(&(func_ptr, func_ty)) = self.collection_element_ptr_funcs.get(&elem_repr) {
            return builder.call(func_ptr, func_ty, &[*collection, *index]).as_ptr();
        }

        let func_ty = self.ctx.ptr().fn_ty(
            self.ctx.arena,
            &[*self.collection_ty, *self.i64],
            false,
        );

        let func_ptr = self.module.function(
            &format!("<collection_element_ptr>::{}", elem_repr.name()),
            func_ty,
        );

        func_ptr.set_linkage(Linkage::Internal);
        self.collection_element_ptr_funcs.insert(elem_repr, (func_ptr, func_ty));

        let result = builder.call(func_ptr, func_ty, &[*collection, *index]).as_ptr();

        let mut accessor_builder = func_ptr.builder(self.ctx, func_ty);
        let builder = &mut accessor_builder;
        let this = &*self;

        let collection = builder.local_get(builder.arg(0).unwrap()).as_struct();
        let index = builder.local_get(builder.arg(1).unwrap()).as_integer();

        let collection_slot = builder.alloca(*this.collection_ty);
        let index_slot = builder.alloca(*this.i64);
        let result_slot = builder.alloca(*this.ctx.ptr());


        builder.store(collection_slot, *collection);
        builder.store(index_slot, *index);

        builder.loop_indefinitely(|builder, loop_id| {
            let collection = builder.load(collection_slot, *this.collection_ty).as_struct();
            let tagged_ptr = self.collection_tagged_ptr(builder, collection);
            let length = self.collection_length(builder, collection);
            let index = builder.load(index_slot, *this.i64).as_integer();

            let (header_ptr, allocation_kind) = this.collection_split_tag(builder, tagged_ptr);

            builder.assume(builder.cmp_int(index, length, IntCmp::UnsignedLt));

            builder.switch(allocation_kind, 0..4, |builder, allocation_kind| {
                match allocation_kind {
                    COLLECTION_FLAT_TAG => {
                        let payload_ty = this.collection_flat_header_type(elem_repr);
                        let data_ptr = builder.field_ptr(header_ptr, payload_ty, 1);
                        let element_ptr = builder.gep_inbounds(data_ptr, elem_repr, index);
                        builder.store(result_slot, *element_ptr);
                        builder.loop_break(loop_id);
                    }

                    COLLECTION_SLICE_TAG => {
                        let ty = this.collection_slice_payload;
                        let offset = builder.field_ptr_load_tbaa(header_ptr, ty, 1, this.tbaa_header).as_integer();
                        let base = builder.field_ptr_load_tbaa(header_ptr, ty, 2, this.tbaa_header).as_struct();

                        let base_index = builder.add_int_nuw(index, offset);
                        let base_length = this.collection_length(builder, base);

                        builder.assume(builder.cmp_int(base_index, base_length, IntCmp::UnsignedLt));
                        builder.assume(builder.cmp_int(offset, base_length, IntCmp::UnsignedLt));
                        builder.assume(builder.cmp_int(index, base_length, IntCmp::UnsignedLt));

                        builder.store(collection_slot, *base);
                        builder.store(index_slot, *base_index);
                        builder.loop_continue(loop_id);
                    }

                    COLLECTION_CONCAT_TAG => {
                        let ty = this.collection_concat_payload;
                        let left = builder.field_ptr_load_tbaa(header_ptr, ty, 1, this.tbaa_header).as_struct();
                        let right = builder.field_ptr_load_tbaa(header_ptr, ty, 2, this.tbaa_header).as_struct();

                        let left_len = this.collection_length(builder, left);
                        let in_left = builder.cmp_int(index, left_len, IntCmp::UnsignedLt);

                        builder.ite(in_left,
                            |builder| {
                                builder.store(collection_slot, *left);
                            },
                            |builder| {
                                let right_index = builder.sub_int(index, left_len);
                                let right_len = this.collection_length(builder, right);
                                builder.assume(builder.cmp_int(right_index, right_len, IntCmp::UnsignedLt));

                                builder.store(collection_slot, *right);
                                builder.store(index_slot, *right_index);
                            },
                        );
                        builder.loop_continue(loop_id);
                    }

                    COLLECTION_UNUSED_TAG => builder.unreachable(),

                    _ => unreachable!(),
                }
            });
        });

        builder.ret(builder.load(result_slot, *this.ctx.ptr()));
        result
    }



    fn collection_with_tag(
        &self, builder: &mut Builder<'ctx>,
        ptr: Ptr<'ctx>, num: Integer<'ctx>
    ) -> Ptr<'ctx> {
        let ptr_as_usize = builder.ptr_to_int(ptr, self.usize);
        let tagged_ptr = builder.or(ptr_as_usize, num);
        builder.int_to_ptr(tagged_ptr, self.ctx.ptr())
    }


    fn collection_split_tag(
        &self, builder: &mut Builder<'ctx>, 
        ptr: Ptr<'ctx>,
    ) -> (Ptr<'ctx>, Integer<'ctx>) {
        let ptr_as_usize = builder.ptr_to_int(ptr, self.usize);
        let tag_mask = 0x3;
        let tag_mask = builder.const_int(self.usize, tag_mask, false);
        let tag = builder.and(ptr_as_usize, tag_mask);

        let ptr_mask = builder.int_not(tag_mask);
        let ptr = builder.ptr_mask(ptr, ptr_mask);

        let align = self.collection_header.align_of(self.module).unwrap();
        let align = self.const_usize(builder, align);

        builder.assume_bundles(&[
            ("align", &[*ptr, *align]),
            ("nonnull", &[*ptr]),
            ("dereferenceable", &[*ptr, *align]),
        ]);


        (ptr, tag)
    }


    fn collection_drop(
        &mut self, 
        env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>,
        collection: Struct<'ctx>,
        elem_repr: LLVMType<'ctx>,
        elem_ty: Option<Type>,
    ) {
        let hash = elem_ty.map(|ty| ty.hash(self.syms));
        let entry_key = (elem_repr, hash);
        if let Some(func) = self.collection_drop_funcs.get(&entry_key) {
            let tagged_ptr = self.collection_tagged_ptr(builder, collection);
            let (header_ptr, _) = self.collection_split_tag(builder, tagged_ptr);

            let should_drop = self.emit_rc_decrement(builder, header_ptr);

            builder.expect(should_drop, false);
            builder.iff(
                should_drop,
                |builder| { builder.call(func.0, func.1, &[*collection, *builder.ptr_null()]); },
            );

            return;
        }


        let func_ty = self.ctx.void().fn_ty(self.ctx.arena, &[*self.collection_ty, *self.ctx.ptr()], false);
        let name = elem_ty
            .map(|s| s.display(self.string_map, self.syms).to_string())
            .unwrap_or_else(|| elem_repr.name().to_string());

        let func_ptr = self.module.function(&format!("<collection_drop>::{}", name), func_ty);

        self.collection_drop_funcs.insert(entry_key, (func_ptr, func_ty));
        self.collection_drop(env, builder, collection, elem_repr, elem_ty);

        let mut builder = func_ptr.builder(self.ctx, func_ty);
        let builder = &mut builder;

        let collection = builder.arg(0).unwrap();
        let collection = builder.local_get(collection).as_struct();

        let tagged_ptr = self.collection_tagged_ptr(builder, collection);
        let (header_ptr, allocation_kind) = self.collection_split_tag(builder, tagged_ptr);
        let length = self.collection_length(builder, collection);

        let total_size_ptr = builder.alloca(*self.usize);

        builder.switch(allocation_kind, 0..4, |builder, idx| {
            match idx {
                COLLECTION_FLAT_TAG => {
                    let total_size = self.collection_flat_allocation_size(
                        builder,
                        elem_repr, 
                        length
                    );

                    builder.store(total_size_ptr, *total_size);

                    let payload_ty = self.collection_flat_header_type(elem_repr);
                    let data_ptr = builder.field_ptr(header_ptr, payload_ty, 1);

                    let counter = builder.alloca(*self.i64);
                    builder.store(counter, *length);


                    if let Some(elem_ty) = elem_ty {
                        let one = builder.const_int(self.i64, 1, false);
                        let zero_val = builder.const_int(self.i64, 0, false);

                        builder.loop_indefinitely(|builder, l| {
                            let i = builder.load(counter, *self.i64).as_integer();
                            let done = builder.cmp_int(i, zero_val, IntCmp::Eq);
                            builder.ite(done,
                                |builder| { builder.loop_break(l); },
                                |builder| {
                                    let i = builder.sub_int(i, one);
                                    builder.store(counter, *i);
                                    let ptr = builder.gep(data_ptr, elem_repr, i);
                                    let elem = builder.load_tbaa(ptr, elem_repr, self.tbaa_element);
                                    self.emit_drop(env, builder, elem, elem_ty);
                                },
                            );
                        });
                    }
                }

                COLLECTION_SLICE_TAG => {
                    let total_size = self.collection_slice_payload.size_of(self.module).unwrap();
                    let total_size = self.const_usize(builder, total_size);
                    builder.store(total_size_ptr, *total_size);

                    let base = builder.field_ptr_load_tbaa(header_ptr, self.collection_slice_payload, 2, self.tbaa_header).as_struct();
                    self.collection_drop(env, builder, base, elem_repr, elem_ty);
                }


                COLLECTION_CONCAT_TAG => {
                    let total_size = self.collection_concat_payload.size_of(self.module).unwrap();
                    let total_size = self.const_usize(builder, total_size);
                    builder.store(total_size_ptr, *total_size);

                    let a = builder.field_ptr_load_tbaa(header_ptr, self.collection_concat_payload, 1, self.tbaa_header).as_struct();
                    let b = builder.field_ptr_load_tbaa(header_ptr, self.collection_concat_payload, 2, self.tbaa_header).as_struct();
                    self.collection_drop(env, builder, a, elem_repr, elem_ty);
                    self.collection_drop(env, builder, b, elem_repr, elem_ty);
                }


                COLLECTION_UNUSED_TAG => builder.unreachable(),

                _ => unreachable!(),
            }
        });


        let total_size = builder.load(total_size_ptr, *self.usize);

        builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*header_ptr, total_size]);
        builder.ret_void();

    }



    fn extern_type_error(
        &mut self,
        ty: Type,
        active: &mut Vec<SymbolId>,
    ) -> Option<ErrorId> {
        let ty = ty.resolve(&[], self.syms);
        if let Some(error) = ty.as_err(self.syms) {
            return Some(error);
        }

        let Type::Ty(sym, gens) = ty
        else { return None };

        if active.contains(&sym) {
            return None;
        }
        active.push(sym);

        let kind = self.syms.sym(sym).kind();
        let nested = match kind {
            SymbolKind::Container(cont) => {
                let gens = self.syms.get_gens(gens);
                let mut fields = Vec::with_capacity(cont.fields().len());
                for (_, field) in cont.fields() {
                    fields.push(field.to_ty(gens, self.syms));
                }
                fields
            },

            SymbolKind::Function(func) => {
                let gens = self.syms.get_gens(gens);
                let mut fields = Vec::with_capacity(func.args().len() + 1);
                for arg in func.args() {
                    fields.push(arg.symbol().to_ty(gens, self.syms));
                }
                fields.push(func.ret().to_ty(gens, self.syms));
                fields
            },

            _ => Vec::new(),
        };

        let error = nested.into_iter()
            .find_map(|nested| self.extern_type_error(nested, active));
        active.pop();
        error
    }


    fn get_func(&mut self, ty: Type) -> Result<&Function<'ctx>, ErrorId> {
        let ty = ty.resolve(&[], self.syms);
        assert!(ty.is_resolved(&mut self.syms));

        let sym_id = ty.sym(self.syms).unwrap();
        let gens_id = ty.gens(self.syms);

        let hash = ty.hash(&self.syms);

        if let Some(func) = self.funcs.get(&hash) { 
            assert!(func.sym.eq(self.syms, ty));
            return Ok(self.funcs.get(&hash).unwrap())
        }

        // create
        let sym = self.syms.sym(sym_id);
        let SymbolKind::Function(sym_func) = sym.kind()
        else { unreachable!() };

        // Calls to validation-broken methods fail fast with the reported
        // diagnostic: the trait-impl validation records the error on the
        // method's decl, and functions are only ever created through here.
        if let Some(decl) = sym_func.decl()
        && let Some(err) = self.ty_info.decls[decl] {
            return Err(err);
        }

        let gens = self.syms.gens()[gens_id];

        assert_eq!(gens.len(), sym.generics().len());
        for ((g0, _), n1) in gens.iter().zip(sym.generics()) {
            assert_eq!(g0.name, n1.name);
        }

        let ret = sym_func.ret().to_ty(gens, self.syms);
        let ret = ret.resolve(&[gens], self.syms);
        let is_never = ret.is_never(self.syms) || ret.is_err(self.syms);

        let args = sym_func
            .args().iter()
            .map(|x| 
                x.symbol()
                .to_ty(gens, self.syms)
                .resolve(&[gens], self.syms)
            ).collect::<Vec<_>>();
        let is_extern = matches!(sym_func.kind(), syms::func::FunctionKind::Extern(_));
        if is_extern {
            let mut active = Vec::new();
            let error = args.iter().copied()
                .find_map(|arg| self.extern_type_error(arg, &mut active))
                .or_else(|| self.extern_type_error(ret, &mut active));
            if let Some(error) = error {
                return Err(error);
            }
        }

        let llvm_ret = self.to_llvm_ty(ret);

        let external_abi = self.extern_abi(llvm_ret.repr);
        let llvm_args = {
            let mut vec = sti::vec::Vec::with_cap_in(&*self.ctx.arena, sym_func.args().len());
            for (arg, ty) in sym_func.args().iter().zip(&args) {
                if arg.is_inout() {
                    vec.push(*self.ctx.ptr());
                } else {
                    vec.push(self.to_llvm_ty(*ty).repr);
                }
            }

            vec.push(*self.ctx.ptr());
            vec.leak()
        };

        let name = ty.display(self.string_map, self.syms);
        let name_idx = self.string_map.insert(name);

        match sym_func.kind() {

            syms::func::FunctionKind::Extern(path) => {
                let (func_ty, func_ptr) =
                if let Some((func_ty, func_ptr, _)) = self.externs.get(&path) { (*func_ty, *func_ptr) }
                else {
                    let external_ret =
                    match external_abi {
                        ExternAbi::Direct => llvm_ret.repr,
                        ExternAbi::SRet(_) => *self.ctx.void(),
                    };

                    let mut external_args = Vec::with_capacity(sym_func.args().len() + 1);
                    if matches!(external_abi, ExternAbi::SRet(_)) {
                        external_args.push(*self.ctx.ptr());
                    }

                    for (arg, ty) in sym_func.args().iter().zip(&args) {
                        if arg.is_inout() {
                            external_args.push(*self.ctx.ptr());
                        } else {
                            external_args.push(self.to_llvm_ty(*ty).repr);
                        }
                    }

                    let external_fn_ty = external_ret.fn_ty(
                        self.ctx.arena,
                        &external_args,
                        false,
                    );
                    let external_fn = self.module.function(self.string_map.get(path), external_fn_ty);
                    external_fn.set_linkage(Linkage::External);
                    if let ExternAbi::SRet(ret) = external_abi {
                        external_fn.set_sret(self.ctx, ret);
                    }
                    if is_never {
                        external_fn.set_noreturn(self.ctx);
                    }

                    // margarine function values carry a trailing capture pointer. Keep that
                    // internal ABI behind a wrapper so runtime imports retain their C ABI.
                    let wrapper_name = format!("__margarine_extern_wrapper.{}", self.func_counter);
                    let wrapper_name = self.string_map.insert(&wrapper_name);
                    self.func_counter += 1;
                    let func_ty = llvm_ret.repr.fn_ty(self.ctx.arena, llvm_args.as_slice(), false);
                    let func_ptr = self.module.function(self.string_map.get(wrapper_name), func_ty);
                    if is_never {
                        func_ptr.set_noreturn(self.ctx);
                    }

                    let builder = func_ptr.builder(self.ctx, func_ty);
                    let mut call_args = Vec::with_capacity(args.len());
                    for i in 0..args.len() {
                        call_args.push(builder.local_get(builder.arg(i).unwrap()));
                    }

                    let result =
                    match external_abi {
                        ExternAbi::Direct => {
                            builder.call(external_fn, external_fn_ty, &call_args)
                        }
                        ExternAbi::SRet(ret) => {
                            let result = builder.alloca(ret);
                            let mut external_call_args = Vec::with_capacity(call_args.len() + 1);
                            external_call_args.push(*result);
                            external_call_args.extend_from_slice(&call_args);
                            builder.call_sret(external_fn, external_fn_ty, ret, &external_call_args);
                            builder.load(result, ret)
                        }
                    };

                    if is_never {
                        builder.unreachable();
                    } else {
                        builder.ret(result);
                    }


                    self.externs.insert(path, (func_ty, func_ptr, external_abi));
                    (func_ty, func_ptr)
                };

                let func = Function {
                    sym: ty,
                    name: self.string_map.insert(self.string_map.get(path)),
                    kind: FunctionKind::Extern,
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

                let previous_function_name = self.current_function_name.replace(name_idx);
                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let mut env = Env {
                    vars: Vec::new(),
                    inouts: Vec::new(),
                    loop_id: None,
                    gens: self.syms.get_gens(gens_id),
                    info: HashMap::new(),
                    ret_llvm_ty: Some(llvm_ret),
                };

                let Decl::Function { body, .. } = self.ast.decl(sym_func.decl().unwrap())
                else { unreachable!() };

                for (i, arg) in sym_func.args().iter().enumerate() {
                    let arg_ty = arg.symbol().to_ty(gens, self.syms);
                    let arg_ty = arg_ty.resolve(&[], self.syms);
                    let param = builder.arg(i).unwrap();
                    if arg.is_inout() {
                        let llvm_ty = self.to_llvm_ty(arg_ty);
                        let local = builder.local(llvm_ty.repr);
                        let value = builder.load(builder.local_get(param).as_ptr(), llvm_ty.repr);
                        builder.local_set(local, value);
                        env.alloc_var(arg.name(), local, arg_ty, true);
                        env.inouts.push((param, local));
                    } else {
                        // Own every by-value parameter: the caller keeps its own reference and
                        // releases it after the call, so the callee must hold its own copy to
                        // make every intra-body drop (assignment overwrites, exit cleanup)
                        // target an owned value. The pair is local to this function, so LLVM
                        // can fold it when the body never touches the refcount.
                        let value = builder.local_get(param);
                        let owned = self.emit_copy(&mut builder, value, arg_ty);
                        builder.local_set(param, owned);
                        env.alloc_var(arg.name(), param, arg_ty, false);
                    }
                }


                let result = self.block(&mut env, &mut builder, &*body);
                self.current_function_name = previous_function_name;

                if let Some(e) = self.ty_info.decl(sym_func.decl().unwrap()) {
                    self.error(&mut env, &mut builder, e);
                } else {
                    match result {
                        Ok((v, body_ty)) => {
                            if !is_never && !body_ty.is_never(self.syms) {
                                self.update_inouts(&env, &mut builder);
                                self.drop_all_locals(&mut env, &mut builder);
                                builder.ret(v);

                            } else {
                                builder.unreachable();
                            }
                        },


                        Err(e) => {
                            self.error(&mut env, &mut builder, e);
                        },
                    }
                }


                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::FloatSqrt => {
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
                let result = builder.sqrt_fp(arg.as_fp());
                builder.ret(*result);

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

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let rc_ty = self.ctx.literal_struct(&[*self.usize, llvm_elem.repr], false);
                let size_val = self.const_usize(&builder, rc_ty.size_of(self.module).unwrap());
                let ptr = builder.call(self.rc_alloc_fn.0, self.rc_alloc_fn.1, &[*size_val]).as_ptr();

                let arg = builder.arg(0).unwrap();
                let arg = builder.local_get(arg);
                let arg = self.emit_copy(&mut builder, arg, elem_ty);

                let one = self.const_usize(&builder, 1);
                builder.store(ptr, *builder.struct_instance(rc_ty, [*one, arg]));

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

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let arg = builder.arg(0).unwrap();
                let arg = builder.local_get(arg);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let rc_ty = self.ctx.literal_struct(&[*self.usize, llvm_elem.repr], false);
                let rc = builder.load(arg.as_ptr(), *rc_ty).as_struct();
                let result = builder.field_load(rc, 1);
                let result = self.emit_copy(&mut builder, result, elem_ty);

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

                let mut builder = func_ptr.builder(self.ctx, func_ty);
                let mut env = Env::default();

                let rc_arg = builder.arg(0).unwrap();
                let rc_arg = builder.local_get(rc_arg);

                let val_arg = builder.arg(1).unwrap();
                let val_arg = builder.local_get(val_arg);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let rc_ty = self.ctx.literal_struct(&[*self.usize, llvm_elem.repr], false);
                let data_ptr = builder.field_ptr(rc_arg.as_ptr(), rc_ty, 1);
                let old_val = builder.load(data_ptr, llvm_elem.repr);
                let val_arg = self.emit_copy(&mut builder, val_arg, elem_ty);
                builder.store(data_ptr, val_arg);

                // Drop the ownership Rc previously had over old_val.
                self.emit_drop(&mut env, &mut builder, old_val, args[1]);

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

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let elem_size_val = self.const_usize(&builder, llvm_elem.repr.size_of(self.module).unwrap());

                let count = builder.local_get(builder.arg(0).unwrap()).as_integer();
                let count = self.int_to_usize(&mut builder, count);
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

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);
                let elem_size_val = self.const_usize(&builder, llvm_elem.repr.size_of(self.module).unwrap());

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let count = builder.local_get(builder.arg(1).unwrap()).as_integer();
                let count = self.int_to_usize(&mut builder, count);
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
                let mut env = Env::default();

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                builder.call(self.assert_not_null_fn.0, self.assert_not_null_fn.1, &[*ptr]);
                let val = builder.local_get(builder.arg(1).unwrap());
                let val = self.emit_copy(&mut builder, val, elem_ty);

                let old = builder.load(ptr, llvm_elem.repr);
                self.emit_drop(&mut env, &mut builder, old, elem_ty);

                builder.store(ptr, val);
                builder.ret(*builder.const_unit());

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrWriteUninit => {
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
                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                builder.call(self.assert_not_null_fn.0, self.assert_not_null_fn.1, &[*ptr]);
                let val = builder.local_get(builder.arg(1).unwrap());
                let val = self.emit_copy(&mut builder, val, elem_ty);
                builder.store(ptr, val);

                builder.ret(*builder.const_unit());

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::PtrDrop => {
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
                let mut env = Env::default();

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                builder.call(self.assert_not_null_fn.0, self.assert_not_null_fn.1, &[*ptr]);
                let old = builder.load(ptr, llvm_elem.repr);
                self.emit_drop(&mut env, &mut builder, old, elem_ty);
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

                let mut builder = func_ptr.builder(self.ctx, func_ty);

                let elem_ty = gens[0].1;
                let llvm_elem = self.to_llvm_ty(elem_ty);

                let ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let off = builder.local_get(builder.arg(1).unwrap()).as_integer();
                let off = self.int_to_usize(&mut builder, off);

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

                let ptr = builder.local_get(builder.arg(0).unwrap());
                builder.ret(ptr);

                return Ok(&self.funcs[&hash]);
            },


            // $list_len<T>(list: [T]): int
            syms::func::FunctionKind::ListLen => {
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
                let list_value = builder.local_get(builder.arg(0).unwrap());
                let list = list_value.as_struct();
                let len = self.collection_length(&mut builder, list);
                builder.ret(*len);

                return Ok(&self.funcs[&hash]);
            },

            // $list_iter<T>(list: [T]): ListIter<T>
            syms::func::FunctionKind::ListIter => {
                let func_ty =
                llvm_ret.repr.fn_ty(
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
                let list_value = builder.local_get(builder.arg(0).unwrap());
                let list = self.emit_copy(&mut builder, list_value, args[0]).as_struct();
                let iter =
                builder.call(
                    self.collection_iter_new.0,
                    self.collection_iter_new.1,
                    &[*list],
                ).as_struct();
                builder.ret(*iter);

                return Ok(&self.funcs[&hash]);
            },



            // $list_iter_next<T>(&iter: ListIter<T>): Option<T>
            syms::func::FunctionKind::ListIterNext => {
                let func_ty =
                llvm_ret.repr.fn_ty(
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
                let (next_fn, next_fn_ty) =
                self.create_collection_iter(llvm_elem.repr);

                let iter_ptr = builder.local_get(builder.arg(0).unwrap()).as_ptr();
                let element_ptr = builder.call(next_fn, next_fn_ty, &[*iter_ptr]).as_ptr();
                let result = builder.alloca(llvm_ret.repr);

                let none_tag = *builder.const_int(self.i32, 1, false);
                let some_tag = *builder.const_int(self.i32, 0, false);
                let unit = *builder.const_unit();
                let none = self.create_enum(&mut builder, ret, none_tag, unit, Type::UNIT);
                let is_end = builder.ptr_is_null(element_ptr);

                builder.ite(is_end,
                    |builder| {
                        builder.store(result, none);
                    },
                    |builder| {
                        let element = builder.load_tbaa(element_ptr, llvm_elem.repr, self.tbaa_element);
                        let element = self.emit_copy(builder, element, elem_ty);
                        let some = self.create_enum(builder, ret, some_tag, element, elem_ty);
                        builder.store(result, some);
                    },
                );

                builder.ret(builder.load(result, llvm_ret.repr));

                return Ok(&self.funcs[&hash]);
            },


            // $list_concat([T], [T]): [T]
            syms::func::FunctionKind::ListConcat => {
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

                let left = builder.local_get(builder.arg(0).unwrap());
                let left = self.emit_copy(&mut builder, left, args[0]);
                let right = builder.local_get(builder.arg(1).unwrap());
                let right = self.emit_copy(&mut builder, right, args[1]);
                let elem_ty = gens[0].1;
                let elem_repr = self.to_llvm_ty(elem_ty).repr;
                let mut env = Env::default();
                let result =
                self.collection_concat(
                    &mut env,
                    &mut builder,
                    left.as_struct(),
                    right.as_struct(),
                    elem_repr,
                    Some(elem_ty),
                );

                builder.ret(*result);


                return Ok(&self.funcs[&hash]);
            },


            // $list_slice([T], int): Option<([T], [T])>
            syms::func::FunctionKind::ListSlice => {
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
                let list_value = builder.local_get(builder.arg(0).unwrap()).as_struct();
                let index_i64 = builder.local_get(builder.arg(1).unwrap()).as_integer();
                let list_len = self.collection_length(&mut builder, list_value);
                let zero_i64 = builder.const_int(self.i64, 0, false);
                let negative = builder.cmp_int(index_i64, zero_i64, IntCmp::SignedLt);
                let past_end = builder.cmp_int(index_i64, list_len, IntCmp::SignedGt);
                let invalid = unsafe { Bool::new(*builder.or(negative.as_integer(), past_end.as_integer())) };
                let result = builder.alloca(llvm_ret.repr);

                let ret_gens = ret.gens(self.syms);
                let option_gens = self.syms.get_gens(ret_gens);
                let pair_ty = option_gens[0].1;
                let zero = builder.const_int(self.i64, 0, false);
                let none_tag = *builder.const_int(self.i32, 1, false);
                let unit = *builder.const_unit();

                let none = self.create_enum(&mut builder, ret, none_tag, unit, Type::UNIT);
                let pair_first = self.string_map.num(0);
                let pair_second = self.string_map.num(1);

                builder.ite(invalid,
                    |builder| {
                        builder.store(result, none);
                    },
                    |builder| {
                        let index = index_i64;
                        let right_len = builder.sub_int(list_len, index);
                        let left_base = self.emit_copy(builder, *list_value, args[0]).as_struct();
                        let right_base = self.emit_copy(builder, *list_value, args[0]).as_struct();
                        let elem_ty = gens[0].1;
                        let llvm_elem = self.to_llvm_ty(elem_ty);
                        let mut env = Env::default();
                        let left = self.collection_slice(
                            &mut env,
                            builder,
                            left_base,
                            zero,
                            index,
                            llvm_elem.repr,
                            Some(elem_ty),
                        );
                        let right = self.collection_slice(
                            &mut env,
                            builder,
                            right_base,
                            index,
                            right_len,
                            llvm_elem.repr,
                            Some(elem_ty),
                        );

                        let pair = self.create_struct(builder, pair_ty, &[
                            (pair_first, *left),
                            (pair_second, *right),
                        ]);
                        let some = self.create_enum(builder, ret, *builder.const_int(self.i32, 0, false), *pair, pair_ty);
                        builder.store(result, some);
                    },
                );

                builder.ret(builder.load(result, llvm_ret.repr));

                return Ok(&self.funcs[&hash]);
            },








            syms::func::FunctionKind::Enum { sym: sym_id, index } => {
                let sym = self.syms.sym(sym_id);
                // add_enum registers the constructor with the enum's own
                // just-finalized container symbol, and errored constructions
                // are skipped by out_if_err before reaching get_func.
                let SymbolKind::Container(cont) = sym.kind()
                else { unreachable!("enum-constructor symbol is a finalized container") };

                let arg = cont.fields()[index];
                let arg_ty = arg.1.to_ty(gens, self.syms);

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
                    self.create_enum(&mut builder, ret, *kind, unit, arg_ty)
                } else {
                    let kind = builder.const_int(self.i32, index as _, false);
                    let value = builder.arg(0).unwrap();
                    let value = builder.local_get(value);
                    let value = self.emit_copy(&mut builder, value, arg_ty);
                    self.create_enum(&mut builder, ret, *kind, value, arg_ty)
                };

                builder.ret(en);

                return Ok(&self.funcs[&hash]);
            },


            syms::func::FunctionKind::Closure(_) => unreachable!(),
            syms::func::FunctionKind::Trait => unreachable!(),
        }
    }


    fn error(&mut self, env: &mut Env<'_, 'ctx>, builder: &mut Builder<'ctx>, e: errors::ErrorId) {
        self.drop_all_locals(env, builder);
        let message = match e {
            ErrorId::Lexer((file, error)) => self.errors[0][file as usize][error.0 as usize].clone(),
            ErrorId::Parser((file, error)) => self.errors[1][file as usize][error.0 as usize].clone(),
            ErrorId::Sema(error) => self.errors[2][0][error.0 as usize].clone(),
        };

        self.emit_panic(builder, &message);
        builder.unreachable();
    }



    fn to_llvm_ty(&mut self, ty: Type) -> TypeMapping<'ctx> {
        let ty = ty.resolve(&[], self.syms);
        assert!(ty.is_resolved(self.syms));

        let hash = ty.hash(self.syms);
        
        if let Some(ty) = self.ty_mappings.get(&hash) { return *ty }

        let sym_id = ty.sym(self.syms).unwrap();

        let def =
        match sym_id {
            SymbolId::LIST => Some(*self.collection_ty),
            SymbolId::LIST_ITER => Some(*self.collection_iter),
            SymbolId::STR => Some(*self.str_ty),
            SymbolId::UNIT => Some(*self.ctx.unit()),
            SymbolId::I64 => Some(*self.i64),
            SymbolId::F64 => Some(*self.ctx.f64()),
            _ => None,
        };

        if let Some(def) = def {
            self.ty_mappings.insert(hash, TypeMapping { repr: def, strct: def });
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

                let ret = function_ty.ret().to_ty(gens, self.syms);

                let ret = self.to_llvm_ty(ret).repr;
                let llvm_args = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*self.ctx.arena, function_ty.args().len());
                    for i in function_ty.args().iter() {
                        let arg = i.symbol().to_ty(gens, self.syms);
                        if i.is_inout() {
                            vec.push(*self.ctx.ptr());
                        } else {
                            vec.push(self.to_llvm_ty(arg).repr);
                        }
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
                            let ty = i.1.to_ty(gens, self.syms);
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
                                let ty = i.1.to_ty(gens, self.syms);
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
                            let ft = field.1.to_ty(gens, self.syms);
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
                let mapping = TypeMapping { repr: *self.ctx.ptr(), strct: *self.ctx.ptr(), };
                self.ty_mappings.insert(hash, mapping);
            },


            SymbolKind::Error(_) => {
                // Keep the zero-field error aggregate concrete: error-typed
                // fields may be stored inside otherwise valid structs.
                let strct = self.ctx.structure("{error}");
                strct.set_fields(&[], false);
                let mapping = TypeMapping { repr: *strct, strct: *strct };
                self.ty_mappings.insert(hash, mapping);
            },


            SymbolKind::Alias(_) => unreachable!(),
            SymbolKind::Namespace => unreachable!(),
            SymbolKind::Trait(_) => unreachable!(),
        };


        self.ty_mappings[&hash]
    }



    fn block(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, block: &[NodeId]
    ) -> Result<(Value<'ctx>, Type), ErrorId> {
        if let Some(NodeId::Err(error)) = block.iter().find(|node| matches!(node, NodeId::Err(_))) {
            return Err(*error);
        }

        let mut has_ret : Option<(Value<'ctx>, Type)> = None;
        let len = env.vars.len();


        for (_, &n) in block.iter().enumerate() {
            if let Some((value, ty)) = has_ret.take() {
                self.emit_drop(env, builder, value, ty);
            }

            match n {
                NodeId::Decl(_) => (),


                NodeId::Stmt(stmt_id) => self.stmt(env, builder, stmt_id)?,


                NodeId::Expr(expr_id) => {
                    let result = self.expr_ex(env, builder, expr_id)?;
                    assert!(result.1.is_resolved(self.syms));
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
            Some((value, ty)) => Ok((value, ty)),
            None => Ok((*builder.const_unit(), Type::UNIT)),
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
                out_if_err!();
                let value = self.expr(env, builder, rhs)?;

                let margarine_ty = self.ty_info.expr(rhs);
                let margarine_ty = margarine_ty.resolve(&[env.gens], self.syms);

                let ty = self.to_llvm_ty(margarine_ty);

                Self::resolve_pattern(self, env, builder, margarine_ty, ty, value, pat);

                Ok(())
            },


            parser::nodes::stmt::Stmt::UpdateValue { lhs, rhs } => {
                out_if_err!();
                if let Some(e) = self.ty_info.expr(lhs).as_err(&mut self.syms) { return Err(e) }
                let rhs = self.expr(env, builder, rhs)?;

                self.assign(env, builder, lhs, rhs);
                Ok(())
            },


            parser::nodes::stmt::Stmt::ForLoop { binding, expr, body } => {
                out_if_err!();
                let iter_value = self.expr(env, builder, expr)?;
                let iter_sym = self.ty_info.expr(expr).resolve(&[env.gens], self.syms);

                let (iter_fn_ret_ty, iter_fn_is_inout, func_ptr, func_ty) = {
                    let sym = iter_sym.sym(self.syms).unwrap();
                    let ns = self.syms.sym_ns(sym);
                    let ns = self.ns.get_ns(ns);

                    let Ok(sym) = ns.get_sym(StringMap::ITER_NEXT_FUNC).unwrap()
                    else { unreachable!() };

                    let func = Type::Ty(sym, iter_sym.gens(self.syms));
                    let func = func.resolve(&[], self.syms);

                    let ret_ty = self.syms.sym(sym);
                    let SymbolKind::Function(ret_ty) = ret_ty.kind()
                    else { unreachable!() };
                    let iter_fn_is_inout = ret_ty.args().first().is_some_and(|arg| arg.is_inout());

                    let gens = iter_sym.gens(self.syms);
                    let gens = self.syms.get_gens(gens);
                    let ret_ty = ret_ty.ret().to_ty(gens, self.syms);

                    let func = self.get_func(func)?;
                    (ret_ty, iter_fn_is_inout, func.func_ptr, func.func_ty)
                };

                // Iterator advancement mutates the iterator through its in-out receiver.
                let (iter_expr, iter_slot) = 
                if iter_fn_is_inout {
                    let value = self.emit_copy(builder, iter_value, iter_sym);
                    let slot = builder.alloca_store(value);
                    (*slot, Some(slot))
                } else {
                    (iter_value, None)
                };

                let iter_fn_binding_value_ty = iter_fn_ret_ty.gens(self.syms);
                let iter_fn_binding_value_ty = self.syms.get_gens(iter_fn_binding_value_ty)[0].1;
                let iter_fn_binding_value_ty_sym = iter_fn_binding_value_ty.resolve(&[env.gens], self.syms);
                let iter_fn_binding_value_ty_llvm = self.to_llvm_ty(iter_fn_binding_value_ty_sym);

                builder.loop_indefinitely(|builder, l| {
                    let null = builder.ptr_null();
                    let call_ret_value = builder.call(func_ptr, func_ty, &[iter_expr, *null]).as_struct();

                    let lo = env.loop_id.swap(Some((l, env.vars.len())));

                    let tag = builder.field_load(call_ret_value, 0).as_integer();
                    let none_case = builder.const_int(tag.as_integer().ty(), 1, false);
                    let cond = builder.cmp_int(tag, none_case, IntCmp::Eq);

                    builder.iff(cond, |builder| {
                        builder.loop_break(l);
                    });

                    let ret_alloca = builder.alloca_store(*call_ret_value);
                    let data_ptr = builder.field_ptr(ret_alloca, call_ret_value.ty(), 1);
                    let value = builder.load(data_ptr, iter_fn_binding_value_ty_llvm.repr);

                    Self::resolve_pattern(self, env, builder, iter_fn_binding_value_ty_sym, iter_fn_binding_value_ty_llvm, value, binding);

                    let result = self.block(env, builder, &*body);
                    if let Err(e) = result {
                        self.error(env, builder, e);
                    };

                    env.loop_id = lo;
                    if let Some((_, local, ty, _)) = env.vars.last().copied() {
                        let value = builder.local_get(local);
                        self.emit_drop(env, builder, value, ty);
                    }
                    env.vars.pop();
                });

                if let Some(slot) = iter_slot {
                    let value = builder.load(slot, self.to_llvm_ty(iter_sym).repr);
                    self.emit_drop(env, builder, value, iter_sym);
                }
                self.emit_drop(env, builder, iter_value, iter_sym);

                Ok(())
            },


            parser::nodes::stmt::Stmt::Attribute { node, .. } => {
                match node {
                    NodeId::Stmt(stmt) => self.stmt(env, builder, stmt),
                    NodeId::Expr(expr) => {
                        let (value, ty) = self.expr_ex(env, builder, expr)?;
                        self.emit_drop(env, builder, value, ty);
                        Ok(())
                    },
                    NodeId::Err(error) => Err(error),
                    NodeId::Decl(_) => unreachable!(),
                }
            },
        }

    }






    fn emit_panic(&mut self, builder: &mut Builder<'ctx>, message: &str) {
        let array_ty = self.ctx.array(*self.ctx.integer(8), message.len());
        let bytes = *self.ctx.const_str(message);
        let string_data = self.module.add_global(*array_ty, "panic_message");
        string_data.set_initialiser(bytes);
        let len = builder.const_int(self.i64, message.len() as i64, false);

        builder.call(self.panic_fn.0, self.panic_fn.1, &[*string_data, *len]);
    }


    fn check_list_index(
        &mut self,
        builder: &mut Builder<'ctx>,
        list: Struct<'ctx>,
        index: Integer<'ctx>,
    ) -> Integer<'ctx> {
        let len = self.collection_length(builder, list);
        let is_lt_len = builder.cmp_int(index, len, IntCmp::SignedLt);
        let zero = builder.const_int(self.i64, 0, false);
        let is_ge_zero = builder.cmp_int(index, zero, IntCmp::SignedGe);
        let is_in_bounds = builder.bool_and(is_lt_len, is_ge_zero);

        builder.ite(is_in_bounds,
            |_| {},
            |builder| {
                self.emit_panic(builder, "list index out of bounds");
            },
        );
        index
    }


    fn resolve_mut_lvalue_ptr(
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
                let parent_ptr = self.resolve_mut_lvalue_ptr(env, builder, val);

                let ty = self.ty_info.expr(val);
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
                let list_slot = self.resolve_mut_lvalue_ptr(env, builder, list);
                let index_val = self.expr(env, builder, index).unwrap().as_integer();

                let elem_ty = self.ty_info.expr(list);
                let elem_ty = elem_ty.gens(self.syms);
                let elem_ty = self.syms.get_gens(elem_ty)[0].1;
                let elem_ty = elem_ty.resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(elem_ty);

                let list_ty = self.ty_info.expr(list).resolve(&[env.gens], self.syms);

                let collection = builder.load(list_slot, *self.collection_ty).as_struct();
                let index_val = self.check_list_index(builder, collection, index_val);
                let length = self.collection_length(builder, collection);

                let zero = builder.const_int(self.i64, 0, false);
                let one = builder.const_int(self.i64, 1, false);

                let (this, this_data) = self.collection_flat(builder, one, llvm_ty.repr);

                // initialise the data
                let old_entry = self.collection_element_ptr(
                    builder, 
                    collection, 
                    index_val, 
                    llvm_ty.repr,
                );

                let old_value = builder.load_tbaa(old_entry, llvm_ty.repr, self.tbaa_element);

                let old_value = self.emit_copy(builder, old_value, elem_ty);
                builder.store_tbaa(this_data, old_value, self.tbaa_element);

                let a_base = self.emit_copy(builder, *collection, list_ty).as_struct();
                let a = self.collection_slice(
                    env,
                    builder,
                    a_base,
                    zero,
                    index_val,
                    llvm_ty.repr,
                    Some(elem_ty),
                );
                let concat =
                self.collection_concat(
                    env,
                    builder,
                    a,
                    this,
                    llvm_ty.repr,
                    Some(elem_ty),
                );

                let rest = builder.add_int(index_val, one);
                let has_rest = builder.cmp_int(rest, length, IntCmp::UnsignedLt);

                builder.store(list_slot, *concat);

                builder.iff(
                    has_rest, 
                    |builder| {
                        let rest_len = builder.sub_int(length, rest);
                        let b_base = self.emit_copy(builder, *collection, list_ty).as_struct();
                        let b = self.collection_slice(
                            env,
                            builder,
                            b_base,
                            rest,
                            rest_len,
                            llvm_ty.repr,
                            Some(elem_ty),
                        );
                        let concat =
                        self.collection_concat(
                            env,
                            builder,
                            concat,
                            b,
                            llvm_ty.repr,
                            Some(elem_ty),
                        );
                        builder.store(list_slot, *concat);
                    });

                self.emit_drop(env, builder, *collection, list_ty);

                let final_collection =
                    builder.load(list_slot, *self.collection_ty).as_struct();
                self.collection_element_ptr(
                    builder,
                    final_collection,
                    index_val,
                    llvm_ty.repr,
                )
            }


            _ => unreachable!("invalid lvalue"),
        }
    }


    fn assign(
        &mut self, env: &mut Env<'_, 'ctx>,
        builder: &mut Builder<'ctx>, 
        expr: ExprId, value: Value<'ctx>
    ) {
        self.ty_info.expr(expr);

        match self.ast.expr(expr) {
            parser::nodes::expr::Expr::Identifier(name, _) => {
                let local = env.find_var(name).unwrap();
                if let Some(old_ty) = env.find_var_ty(name) {
                    let old_value = builder.local_get(local);
                    self.emit_drop(env, builder, old_value, old_ty);
                }
                builder.local_set(local, value)
            }


            parser::nodes::expr::Expr::AccessField { .. } => {
                let ty = self.ty_info.expr(expr).resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(ty);
                let ptr = self.resolve_mut_lvalue_ptr(env, builder, expr);
                let old_value = builder.load(ptr, llvm_ty.repr);
                self.emit_drop(env, builder, old_value, ty);
                builder.store(ptr, value);
            }


            parser::nodes::expr::Expr::Unwrap(expr) => {
                let ty = self.ty_info.expr(expr);
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

                        builder.ite(comp,
                        |_| {},


                        |builder| {
                            self.emit_panic(builder, "attempted to unwrap a none value");
                        }, 
                        );


                        let value = self.create_enum_from_llvm(builder, tag, value, llvm_enum_ty);
                        builder.local_set(local, value);
                    },


                    parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                        let ty = self.ty_info.expr(val);
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
                        let field_ty = cont.fields()[i].1.to_ty(gens, self.syms);
                        let field_ty = field_ty.resolve(&[env.gens], self.syms);
                        let field_llvm_ty = self.to_llvm_ty(field_ty);


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                let field = self.resolve_mut_lvalue_ptr(env, builder, expr);


                                let enum_struct = builder
                                    .load(field, field_llvm_ty.repr).as_struct();

                                // unwrap
                                let some = builder.const_int(self.i32, 0, false);
                                let tag = builder.field_load(enum_struct, 0);

                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(comp,
                                |_| {},


                                |builder| {
                                    self.emit_panic(builder, "attempted to unwrap a none value");
                                }, 
                                );


                                let value = self.create_enum_from_llvm(builder, tag, value, field_llvm_ty);
                                builder.store(field, value);
                            },


                            ContainerKind::Enum => {
                                let val_ty = ty.resolve(&[env.gens], self.syms);
                                let val_llvm_ty = self.to_llvm_ty(val_ty);
                                let enum_ptr = self.resolve_mut_lvalue_ptr(env, builder, val);
                                let strct = builder.load(enum_ptr, val_llvm_ty.repr).as_struct();
                                let tag = builder.field_load(strct, 0);

                                // unwrap
                                let some = builder.const_int(self.i32, i as i64, false);
                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(comp,
                                |_| {},


                                |builder| {
                                    self.emit_panic(builder, "attempted to unwrap an invalid enum variant");
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
                let ty = self.ty_info.expr(expr);
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

                        builder.ite(comp,
                        |_| {},


                        |builder| {
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
                        let ty = self.ty_info.expr(val);
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
                        let field_ty = cont.fields()[i].1.to_ty(gens, self.syms);
                        let field_ty = field_ty.resolve(&[env.gens], self.syms);
                        let field_llvm_ty = self.to_llvm_ty(field_ty);


                        match cont.kind() {
                              ContainerKind::Tuple
                            | ContainerKind::Struct => {
                                let field = self.resolve_mut_lvalue_ptr(env, builder, expr);


                                let enum_struct = builder
                                    .load(field, field_llvm_ty.repr).as_struct();

                                // unwrap
                                let some = builder.const_int(self.i32, 0, false);
                                let tag = builder.field_load(enum_struct, 0);

                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(comp,
                                |_| {},


                                |builder| {
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
                                let enum_ptr = self.resolve_mut_lvalue_ptr(env, builder, val);
                                let strct = builder.load(enum_ptr, val_llvm_ty.repr).as_struct();
                                let tag = builder.field_load(strct, 0);

                                // unwrap
                                let some = builder.const_int(self.i32, i as i64, false);
                                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                                builder.ite(comp,
                                |_| {},


                                |builder| {
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


            Expr::IndexList { .. } => {
                let elem_ty = self.ty_info.expr(expr).resolve(&[env.gens], self.syms);
                let llvm_ty = self.to_llvm_ty(elem_ty);
                let elem_ptr = self.resolve_mut_lvalue_ptr(env, builder, expr);
                let old_elem = builder.load(elem_ptr, llvm_ty.repr);
                self.emit_drop(env, builder, old_elem, elem_ty);
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
                let ty = self.ty_info.expr(expr);
                if let Some(err) = ty.as_err(&mut self.syms) {
                    return Err(err);
                }
                ty
            }};
        }


        let val = self.ast.expr(expr);
        let result_ty = self.ty_info.exprs[expr].unwrap().ty.resolve(&[env.gens], self.syms);
        let result_err = result_ty.as_err(&mut self.syms);

        let llvm_value = 
        match val {
            parser::nodes::expr::Expr::Unit => *builder.const_unit(),
            parser::nodes::expr::Expr::Literal(literal) => {
                match literal {
                    lexer::Literal::Integer(v) => *builder.const_int(self.i64, v, true),
                    lexer::Literal::Float(f) => *builder.const_f64(f.inner()),


                    lexer::Literal::String(string_index) => {
                        let string = self.string_map.get(string_index);
                        let len = builder.const_int(self.i64, string.len() as _, false);

                        let byte_ty = self.ctx.integer(8);
                        let byte_arr_ty = self.ctx.array(*self.ctx.integer(8), string.len());
                        let bytes = *self.ctx.const_str(string);
                        let global = self.module.add_global(*byte_arr_ty, "str_literal");
                        global.set_initialiser(bytes);
                        let bytes = builder.load(global.as_ptr(), *byte_arr_ty);
 
                        let (collection, data) = self.collection_flat(builder, len, *byte_ty);
                        builder.store_tbaa(data, bytes, self.tbaa_element);
                        *builder.struct_instance(self.str_ty, [*collection])
                    },


                    lexer::Literal::Bool(v) => {
                        let kind = builder.const_bool(v);
                        let value = *builder.const_unit();
                        self.create_enum(builder, Type::BOOL, *kind, value, Type::UNIT)
                    },
                }
            },


            parser::nodes::expr::Expr::Paren(expr_id) => self.expr(env, builder, expr_id)?,


            parser::nodes::expr::Expr::Identifier(name, _) => {
                let ty = out_if_err!();

                let ty = ty.resolve(&[env.gens], self.syms);

                let func =
                // its a trait func
                if let Some(trait_ty) = self.ty_info.trait_funcs.get(&expr).copied() {
                    let sym = self.ty_info.idents.get(&expr).unwrap().unwrap();
                    let sym = Type::Ty(sym, GenListId::EMPTY);
                    let sym = sym.resolve(&[env.gens], self.syms);
                    self.trait_accessor_function(sym, trait_ty, ty, name, &[env.gens])
                    
                } else if let Some(Some(sym)) = self.ty_info.idents.get(&expr) {
                    let func_gens = ty.gens(self.syms);
                    Some(Type::Ty(*sym, func_gens).resolve(&[env.gens], self.syms))

                } else {
                    None
                };

                // it's a function
                if let Some(sym) = func {
                    let func_ref_ty = self.func_ref;
                    let func = self.get_func(sym)?;

                    // create func ref
                    // we want a null ptr as the environment pointer
                    // since named-funcs have no captures we don't 
                    // need to allocate anything
                    let null = builder.ptr_null();
                    let ptr = func.func_ptr;
                    let func_ref = builder.struct_instance(
                        func_ref_ty,
                        [*ptr, *null],
                    );

                    return Ok((*func_ref, sym))
                }


                let value = builder.local_get(env.find_var(name).unwrap());
                self.emit_copy(builder, value, ty)
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

                let sym = self.ty_info.expr(lhs);
                let sym = sym.resolve(&[env.gens], self.syms);

                let result = match operator {
                    BinaryOperator::Eq => {
                        let local = builder.local(*self.ctx.bool());
                        let f = builder.const_bool(true);
                        builder.local_set(local, *f);

                        self.eq(env, builder, sym, local, lhs_val, rhs_val);

                        builder.local_get(local)
                    },


                    BinaryOperator::Ne => {
                        let local = builder.local(*self.ctx.bool());
                        let f = builder.const_bool(true);
                        builder.local_set(local, *f);

                        self.eq(env, builder, sym, local, lhs_val, rhs_val);

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
                              BinaryOperator::Gt => *builder.cmp_int(l, r, if signed { IntCmp::SignedGt } else { IntCmp::UnsignedGt }),
                              BinaryOperator::Ge => *builder.cmp_int(l, r, if signed { IntCmp::SignedGe } else { IntCmp::UnsignedGe }),
                              BinaryOperator::Lt => *builder.cmp_int(l, r, if signed { IntCmp::SignedLt } else { IntCmp::UnsignedLt }),
                              BinaryOperator::Le => *builder.cmp_int(l, r, if signed { IntCmp::SignedLe } else { IntCmp::UnsignedLe }),

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
                        Type::UNIT,
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

                builder.ite_ex(&mut (self, env), tag,
                |builder, (this, env)| {
                    let value = 
                    match this.expr(env, builder, body) {
                        Ok(v) => v,
                        Err(err) => {
                            this.error(env, builder, err);
                            return;
                        },
                    };

                    if let Some(local) = local {
                        builder.local_set(local, value);
                    }
                },


                |builder, (this, env)| {
                    let Some(body) = else_block
                    else { return; };

                    let value =
                    match this.expr(env, builder, body) {
                        Ok(v) => v,
                        Err(err) => {
                            this.error(env, builder, err);
                            return;
                        },
                    };


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
                let match_value = self.expr(env, builder, value)?;
                let ty = out_if_err!().resolve(&[env.gens], self.syms);

                let sym = self.ty_info.expr(value);
                let sym = sym.resolve(&[env.gens], self.syms);

                let gens = sym.gens(self.syms);
                let sym_id = sym.sym(self.syms).unwrap();
                let sym_data = self.syms.sym(sym_id);

                let SymbolKind::Container(cont) = sym_data.kind()
                else { unreachable!() };

                let value_ty = match_value.as_struct();
                let value_tag = builder.field_load(value_ty, 0).as_integer();

                let enum_llvm_ty = self.to_llvm_ty(sym);
                let val_alloca = builder.alloca_store(match_value);
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
                    let field_ty = field.1.to_ty(gens, self.syms);
                    let field_ty = field_ty.resolve(&[env.gens], self.syms);
                    let field_ty_llvm = self.to_llvm_ty(field_ty);

                    let local = builder.local(field_ty_llvm.repr);
                    let binding = 
                    if field_ty.sym(self.syms).unwrap() == SymbolId::UNIT {
                        *builder.const_unit()
                    } else {
                        let payload = builder.load(val_data_ptr, field_ty_llvm.repr);
                        self.emit_copy(builder, payload, field_ty)
                    };

                    builder.local_set(local, binding);
                    self.emit_drop(env, builder, match_value, sym);

                    let match_body_is_never = self.ty_info.expr(mapping.expr())
                        .is_never(self.syms);

                    env.vars.push((mapping.binding(), local, field_ty, false));

                    // run the body
                    let ret_val = self.expr(env, builder, mapping.expr());

                    let ret_val = match ret_val {
                        Ok(value) => value,
                        Err(error) => {
                            self.error(env, builder, error);
                            let binding = env.vars.pop().unwrap();
                            debug_assert_eq!(binding, (mapping.binding(), local, field_ty, false));
                            return;
                        },
                    };

                    let binding = env.vars.pop().unwrap();
                    debug_assert_eq!(binding, (mapping.binding(), local, field_ty, false));
                    if !match_body_is_never {
                        let binding = builder.local_get(local);
                        self.emit_drop(env, builder, binding, field_ty);
                        builder.local_set(ret_local, ret_val);
                    }
                });

                if ty.is_never(self.syms) { *builder.const_unit() }
                else { builder.local_get(ret_local) }
            },


            parser::nodes::expr::Expr::IndexList { list, index } => {
                let list_ty = self.ty_info.expr(list).resolve(&[env.gens], self.syms);
                let (list_value, list_is_temporary) = 
                match self.ast.expr(list) {
                    parser::nodes::expr::Expr::Identifier(name, _) => {
                        let local = env.find_var(name).unwrap();
                        (builder.local_get(local), false)
                    },
                    _ => (self.expr(env, builder, list)?, true),
                };
                let index = self.expr(env, builder, index)?.as_integer();
                out_if_err!();

                let elem_ty = {
                    let ty = list_ty.gens(self.syms);
                    let ty = self.syms.get_gens(ty);
                    ty[0].1.resolve(&[env.gens], self.syms)
                };
                let elem_llvm = self.to_llvm_ty(elem_ty);

                let list = list_value.as_struct();
                let index = self.check_list_index(builder, list, index);
                let element_ptr = self.collection_element_ptr(
                    builder,
                    list,
                    index,
                    elem_llvm.repr,
                );
                let value = builder.load_tbaa(element_ptr, elem_llvm.repr, self.tbaa_element);
                let value = self.emit_copy(builder, value, elem_ty);
                if list_is_temporary {
                    self.emit_drop(env, builder, list_value, list_ty);
                }
                value
            },


            parser::nodes::expr::Expr::Block { block } => {
                self.block(env, builder, &*block)?.0
            },


            parser::nodes::expr::Expr::CreateStruct { fields, .. } => {
                let mut values = sti::vec::Vec::with_cap_in(self.ctx.arena, fields.len());

                let ty = out_if_err!();
                let ty = ty.resolve(&[env.gens], self.syms);
                let gens = ty.gens(self.syms);
                let gens = self.syms.get_gens(gens);

                let sym = ty.sym(self.syms).unwrap();
                let field_types = match self.syms.sym(sym).kind() {
                    SymbolKind::Container(cont) => {
                        cont.fields().iter().map(|(name, f)| (*name, f.to_ty(gens, self.syms))).collect::<Vec<_>>()
                    },
                    _ => unreachable!("type is not a container"),
                };

                for (name, _, e) in fields {
                    let fty = field_types.iter().find(|(n, _)| *n == *name).map(|(_, t)| *t);
                    let value = match fty {
                        // The errored field's slot is the zero-size error type;
                        // evaluate its value for side effects but absorb its
                        // error so the rest of the literal stays constructible.
                        Some(fty) if fty.is_err(self.syms) => {
                            let _ = self.expr(env, builder, *e);
                            builder.const_zero(self.to_llvm_ty(fty).strct)
                        },
                        _ => self.expr(env, builder, *e)?,
                    };
                    values.push((*name, value));
                }

                *self.create_struct(builder, ty, &*values)
            },


            parser::nodes::expr::Expr::AccessField { val, field_name, .. } => {
                let value = self.expr(env, builder, val)?;
                env.info.insert(expr, value);

                let this = out_if_err!();

                let val = self.ty_info.expr(val);
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
                            let field = builder.field_load(value.as_struct(), i as _);
                            let field_ty = this.resolve(&[env.gens], self.syms);
                            let field = self.emit_copy(builder, field, field_ty);
                            self.emit_drop(env, builder, value, val);
                            return Ok((field, field_ty))
                        },

                        ContainerKind::Enum => {
                            let val_ty = val.resolve(&[env.gens], self.syms);
                            let val_llvm_ty = self.to_llvm_ty(val_ty);
                            let result_ty = this.resolve(&[env.gens], self.syms);
                            let result_llvm_ty = self.to_llvm_ty(result_ty);

                            let enum_strct = value.as_struct();
                            let tag = builder.field_load(enum_strct, 0).as_integer();
                            let index = builder.const_int(self.i32, i as _, false);
                            let cond = builder.cmp_int(tag, index, IntCmp::Eq);

                            let val_gens = val.gens(self.syms);
                            let gens_list = self.syms.get_gens(val_gens);
                            let field_gen = cont.fields()[i].1;
                            let field_ty = field_gen.to_ty(gens_list, self.syms);
                            let field_llvm = self.to_llvm_ty(field_ty);

                            let src_buf = builder.alloca_store(value);
                            let src_data_ptr = builder.field_ptr(src_buf, val_llvm_ty.strct.as_struct(), 1);

                            let result_buf = builder.alloca(result_llvm_ty.repr);
                            let result_strct_ty = result_llvm_ty.strct.as_struct();

                            builder.ite(cond,
                                |builder| {
                                    let some_tag = builder.const_int(self.i32, 0, false);
                                    let tag_ptr = builder.field_ptr(result_buf, result_strct_ty, 0);
                                    builder.store(tag_ptr, *some_tag);
                                    let payload = builder.load(src_data_ptr, field_llvm.repr);
                                    let dst_data = builder.field_ptr(result_buf, result_strct_ty, 1);
                                    builder.store(dst_data, payload);
                                },
                                |builder| {
                                    let none_tag = builder.const_int(self.i32, 1, false);
                                    let tag_ptr = builder.field_ptr(result_buf, result_strct_ty, 0);
                                    builder.store(tag_ptr, *none_tag);
                                },
                            );

                            return Ok((builder.load(result_buf, result_llvm_ty.repr), this))
                        },


                        ContainerKind::Generic => unreachable!(),
                    }

                }


                let val_gens = val.gens(self.syms);
                let val_gens = self.syms.get_gens(val_gens);

                if let Some(trait_ty) = self.ty_info.trait_funcs.get(&expr).copied() {
                    let sym = self.trait_accessor_function(
                        val,
                        trait_ty,
                        this,
                        field_name,
                        &[env.gens, val_gens],
                    ).expect("trait implementation registered for accessor");
                    let func_ref_ty = self.func_ref;
                    let func = self.get_func(sym)?;

                    let null = builder.ptr_null();
                    let func_ref = builder.struct_instance(
                        func_ref_ty,
                        [*func.func_ptr, *null],
                    );

                    return Ok((*func_ref, sym))
                }


                let ns = self.ns.get_ns(self.syms.sym_ns(ty));


                if let Some(sym) = ns.get_sym(field_name) {
                    let sym = sym.unwrap();
                    let gens = this.gens(self.syms);

                    let sym = Type::Ty(sym, gens)
                        .resolve(&[env.gens, val_gens], self.syms);

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

                // Type errors are attached to individual arguments, not necessarily the call.
                // Check them before materializing the callee, which may instantiate generic code.
                for arg in args {
                    if let Some(e) = self.ty_info.expr(arg.expr).as_err(&mut self.syms) {
                        return Err(e);
                    }
                }

                let (func, func_ty) = self.expr_ex(env, builder, lhs)?;
                out_if_err!();


                let callable_ty = func_ty.resolve(&[env.gens], self.syms);
                let func_sym = callable_ty.sym(self.syms).unwrap();
                let SymbolKind::Function(function) = self.syms.sym(func_sym).kind()
                else { unreachable!() };


                let accessor = 
                if let Expr::AccessField { val, .. } = self.ast.expr(lhs) {
                    Some(Ok::<(Value<'_>, ExprId), ErrorId>((env.info[&lhs], val)))
                } else { 
                    None 
                };

                let args_iter = accessor.into_iter()
                    .chain(args.iter().map(|a| Ok((self.expr(env, builder, a.expr)?, a.expr))))
                    .collect::<Vec<_>>();

                let mut value_args = Vec::new();
                for (index, arg_expr) in args_iter.into_iter().enumerate() {
                    let (value, expr) = arg_expr?;

                    let ty = self.ty_info.expr(expr);
                    let ty = ty.resolve(&[env.gens], self.syms);

                    let is_inout = function.args()[index].is_inout();
                    let (value, inout) =
                    if is_inout {
                        (*builder.alloca_store(value), Some(expr))
                    } else {
                        (value, None)
                    };

                    value_args.push((value, ty, inout));
                }


                assert!(callable_ty.is_resolved(self.syms));

                let func_ty = self.to_llvm_ty(callable_ty);

                let func_ref = func.as_struct();
                let func_ptr = builder.field_load(func_ref, 0).as_func();
                let func_env = builder.field_load(func_ref, 1).as_ptr();
                let result =
                self.call_function(
                    env,
                    builder,
                    (func_ptr, func_env),
                    func_ty.strct.as_func(),
                    &*value_args,
                );

                self.emit_drop(env, builder, func, callable_ty);

                result
            },


            parser::nodes::expr::Expr::Closure { body, .. } => {
                let ty = out_if_err!();

                let closure = ty.sym(self.syms).unwrap();
                let sym = self.syms.sym(closure);
                let SymbolKind::Function(func_ty) = sym.kind()
                else { unreachable!() };

                let syms::func::FunctionKind::Closure(closure) = func_ty.kind()
                else { unreachable!() };

                let outer_gens = env.gens;
                let ty = ty.resolve(&[outer_gens], self.syms);
                let closure_type_gens = ty.gens(self.syms);
                let closure_gens = self.syms.get_gens(closure_type_gens);
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


                let closure_name =
                match self.current_function_name {
                    Some(parent) => format!("{}.closure.{}", self.string_map.get(parent), self.func_counter),
                    None => format!("<closure>.{}", self.func_counter),
                };


              
                let (buf, strct_ty) =
                if !captured.is_empty() {
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
                    strct_fields.push(*self.usize);
                    strct_fields.push(*self.ctx.ptr());
                    strct_fields.extend_from_slice(&tys);

                    let strct_ty = self.ctx.structure("captures");
                    strct_ty.set_fields(&strct_fields, false);

                    let one = self.const_usize(builder, 1);
                    let zero = self.const_usize(builder, 0);


                    let void = self.ctx.void();
                    let drop_fn_ty = void.fn_ty(self.ctx.arena, &[*self.ctx.ptr()], false);
                    let drop_fn = self.module.function(&format!("{closure_name}.drop"), drop_fn_ty);

                    let mut drop_builder = drop_fn.builder(self.ctx, drop_fn_ty);
                    let arg = drop_builder.arg(0).unwrap();
                    let drop_ptr = drop_builder.local_get(arg).as_ptr();

                    let drop_header = drop_builder.load(drop_ptr, *strct_ty).as_struct();
                    let rc = drop_builder.field_load(drop_header, 0).as_integer();
                    let new_rc = drop_builder.sub_int(rc, one);

                    let rc_ptr = drop_builder.field_ptr(drop_ptr, strct_ty, 0);
                    drop_builder.store(rc_ptr, *new_rc);

                    let is_zero = drop_builder.cmp_int(new_rc, zero, IntCmp::Eq);

                    drop_builder.iff(is_zero,
                    |builder| {
                        for i in (0..drop_tys.len()).rev() {
                            let capture_ty = drop_tys[i];
                            let value = builder.field_load(drop_header, i + 2);
                            self.emit_drop(env, builder, value, capture_ty);
                        }
                        let size_val = self.const_usize(builder, strct_ty.size_of(self.module).unwrap());
                        builder.call(self.dealloc_fn.0, self.dealloc_fn.1, &[*drop_ptr, *size_val]);
                    });

                    drop_builder.ret_void();


                    let mut all_vals: Vec<Value<'ctx>> = Vec::with_capacity(captured.len() + 2);
                    all_vals.push(*one);
                    all_vals.push(*drop_fn);
                    all_vals.extend_from_slice(&vals);

                    let captures = builder.struct_instance(strct_ty, all_vals);

                    let size = self.const_usize(builder, strct_ty.size_of(self.module).unwrap());
                    let buf = builder.call(self.alloc_fn.0, self.alloc_fn.1, &[*size]).as_ptr();
                    builder.store(buf, *captures);
                    (buf, Some(strct_ty))
                } else {
                    (builder.ptr_null(), None)
                };

                self.func_counter += 1;
                let closure_name = self.string_map.insert(&closure_name);

                let func = {
                    let llvm_func_ty = llvm_ty.strct.as_func();
                    let func_ptr = self.module.function(self.string_map.get(closure_name), llvm_func_ty);

                    let func = Function {
                        sym: ty,
                        name: closure_name,
                        kind: FunctionKind::Code,
                        error: None,
                        func_ty: llvm_func_ty,
                        func_ptr,
                    };


                    assert!(self.funcs.insert(hash, func).is_none());

                    let mut builder = func_ptr.builder(self.ctx, llvm_func_ty);
                    let closure_ret = func_ty.ret().to_ty(combined_gens, self.syms).resolve(&[], self.syms);
                    let closure_ret = self.to_llvm_ty(closure_ret);

                    let mut env = Env {
                        vars: Vec::new(),
                        inouts: Vec::new(),
                        loop_id: None,
                        gens: combined_gens,
                        info: HashMap::new(),
                        ret_llvm_ty: Some(closure_ret),
                    };


                    let captured_ptr = builder.arg(func_ty.args().len() as _).unwrap();
                    let captured_ptr = builder.local_get(captured_ptr).as_ptr();

                    if let Some(strct_ty) = strct_ty { 
                        for (i, capture) in captured.iter().enumerate() {
                            let capture_ty = capture.1.resolve(&[combined_gens], self.syms);
                            let capture_llvm_ty = self.to_llvm_ty(capture_ty);
                            let capture_ptr = builder.field_ptr(captured_ptr, strct_ty, i + 2);
                            let local = builder.local(capture_llvm_ty.repr);
                            builder.local_set(local, builder.load(capture_ptr, capture_llvm_ty.repr));
                            env.alloc_var(capture.0, local, capture_ty, true);
                        }
                    }


                    for (i, arg) in func_ty.args().iter().enumerate() {
                        let arg_ty = arg.symbol().to_ty(env.gens, self.syms);
                        let arg_ty = arg_ty.resolve(&[], self.syms);
                        let param = builder.arg(i).unwrap();
                        if arg.is_inout() {
                            let llvm_ty = self.to_llvm_ty(arg_ty);
                            let local = builder.local(llvm_ty.repr);
                            let value = builder.load(builder.local_get(param).as_ptr(), llvm_ty.repr);
                            builder.local_set(local, value);
                            env.alloc_var(arg.name(), local, arg_ty, true);
                            env.inouts.push((param, local));
                        } else {
                            // Own every by-value parameter: the caller keeps its own reference and
                            // releases it after the call, so the callee must hold its own copy to
                            // make every intra-body drop (assignment overwrites, exit cleanup)
                            // target an owned value. The pair is local to this function, so LLVM
                            // can fold it when the body never touches the refcount.
                            let value = builder.local_get(param);
                            let owned = self.emit_copy(&mut builder, value, arg_ty);
                            builder.local_set(param, owned);
                            env.alloc_var(arg.name(), param, arg_ty, false);
                        }
                    }

                    let result = self.expr(&mut env, &mut builder, body);

                    match result {
                        Ok(v) => {
                            self.update_inouts(&env, &mut builder);
                            self.drop_all_locals(&mut env, &mut builder);
                            builder.ret(v);
                        },
                        Err(e) => self.error(&mut env, &mut builder, e),
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
                    env.loop_id = Some((l, env.vars.len()));
                    let result = self.block(env, builder, &body);

                    if let Err(e) = result {
                        self.error(env, builder, e);
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

                self.update_inouts(env, builder);
                self.drop_all_locals(env, builder);
                builder.ret(val);
                builder.unreachable();
                *builder.const_unit()
            },



            parser::nodes::expr::Expr::Continue => {
                out_if_err!();

                let (loop_id, cleanup_start) = env.loop_id.unwrap();
                self.drop_locals(env, builder, cleanup_start);
                builder.loop_continue(loop_id);
                *builder.const_unit()
            },


            parser::nodes::expr::Expr::Break => {
                out_if_err!();

                let (loop_id, cleanup_start) = env.loop_id.unwrap();
                self.drop_locals(env, builder, cleanup_start);
                builder.loop_break(loop_id);
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

                let ty = self.ty_info.expr(expr);

                let ty = ty.resolve(&[env.gens], self.syms);
                *self.create_struct(builder, ty, &llvm_exprs)
            },


            parser::nodes::expr::Expr::AsCast { lhs, .. } => {
                let lhs_val = self.expr(env, builder, lhs)?;
                let lsym = self.ty_info.expr(lhs).sym(self.syms).unwrap();
                out_if_err!();


                let ty = out_if_err!();
                let dest = self.to_llvm_ty(ty);


                if lsym.is_int() && ty.is_float(self.syms) {
                    if lsym.is_sint() {
                        let value = lhs_val.as_integer();
                        builder.si_to_fp(value, dest.repr)
                    } else {
                        let value = builder.int_cast(lhs_val.as_integer(), *self.i64, false).as_integer();
                        builder.ui_to_fp(value, dest.repr)
                    }
                } else if lsym.is_float() && ty.is_int(self.syms) {
                    if ty.sym(self.syms).unwrap() == SymbolId::BYTE {
                        builder.fp_to_ui(lhs_val.as_fp(), dest.repr.as_integer())
                    } else {
                        builder.fp_to_si(lhs_val.as_fp(), dest.repr.as_integer())
                    }

                } else if lsym.is_int() && ty.is_int(self.syms) {
                    builder.int_cast(lhs_val.as_integer(), dest.repr, lsym.is_sint())
                } else if lsym == SymbolId::BOOL && ty.is_int(self.syms) {
                    let tag = builder.field_load(lhs_val.as_struct(), 0);
                    builder.int_cast(tag.as_integer(), dest.repr, false)
                } else if lsym == ty.sym(self.syms).unwrap() {
                    lhs_val
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

                let list_type = out_if_err!();
                let list_type = list_type.resolve(&[env.gens], self.syms);

                let list_type_gens = list_type.gens(self.syms);
                let elem_type = self.syms.get_gens(list_type_gens)[0].1;
                let elem_repr = self.to_llvm_ty(elem_type).repr;

                let len = builder.const_int(self.i64, exprs.len() as _, false);
                let (value, buf) = self.collection_flat(builder, len, elem_repr);

                for (i, &value) in llvm_exprs.iter().enumerate() {
                    let index = builder.const_int(self.i64, i as _, false);
                    let ptr = builder.gep(buf, elem_repr, index);
                    builder.store_tbaa(ptr, value, self.tbaa_element);
                }

                *value
            },

            parser::nodes::expr::Expr::Unwrap(expr_id) => {
                let value = self.expr(env, builder, expr_id)?;
                out_if_err!();


                let some = builder.const_int(self.i32, 0, false);
                let tag = builder.field_load(value.as_struct(), 0);

                let comp = builder.cmp_int(tag.as_integer(), some, IntCmp::Eq);

                builder.ite(comp,
                |_| {},


                |builder| {
                    self.emit_panic(builder, "attempted to unwrap a none value");
                }, 
                );


                let buf = builder.alloca_store(value);
                let field_ty = self.ty_info.expr(expr_id);
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

                builder.ite(comp,
                |_| {},


                |builder| {
                    self.drop_all_locals(env, builder);
                    if let Some(ret_ty) = env.ret_llvm_ty {
                        let none_tag = builder.const_int(self.i32, 1, false);
                        let none_value = *builder.const_unit();
                        let ret_val = self.create_enum_from_llvm(builder, *none_tag, none_value, ret_ty);
                        builder.ret(ret_val);
                    } else {
                        builder.unreachable();
                    }
                }, 
                );

                let ty = self.ty_info.expr(expr_id);
                let gens = ty.resolve(&[env.gens], self.syms).gens(self.syms);
                let gens = self.syms.get_gens(gens);

                let value_ty = gens[0].1.resolve(&[env.gens], self.syms);
                let value_llvm = self.to_llvm_ty(value_ty);

                let buf = builder.alloca_store(value);
                let data_ptr = builder.field_ptr(buf, value.ty().as_struct(), 1);
                let payload = builder.load(data_ptr, value_llvm.repr);

                payload
            },
        };

        if let Some(err) = result_err {
            return Err(err);
        }


        Ok((llvm_value, result_ty))
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
                else { return };
                let item_type_gens = ty.gens(self.syms);
                let gens = self.syms.get_gens(item_type_gens);

                for (i, &item) in items.iter().enumerate() {
                    let field = builder.field_load(value, i);
                    let local = builder.local(field.ty());

                    let field_ty = cont.fields()[i].1.to_ty(gens, self.syms);
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
        let cont = match sym.kind() {
            SymbolKind::Container(cont) => cont,
            SymbolKind::Error(_) => return builder.const_zero(self.to_llvm_ty(ty).strct).as_struct(),
            _ => unreachable!("type is not a container"),
        };

        assert!(values.len() <= cont.fields().len());
        assert!(matches!(cont.kind(), ContainerKind::Struct | ContainerKind::Tuple));

        let gens = ty.gens(self.syms);
        let gens = self.syms.get_gens(gens);

        let ty = self.to_llvm_ty(ty);

        // A field whose type failed to resolve maps to the zero-size error
        // type; store a zero in that slot so the struct stays constructible
        // and only the errored field is unusable (reads of it propagate the
        // error). The CreateStruct arm absorbs the errored field's value.
        builder.struct_instance(
            ty.strct.as_struct(),
            cont.fields().iter().map(|(field_name, field)| {
                let value = values.iter().find(|x| x.0 == *field_name);
                let fty = field.to_ty(gens, self.syms);
                if fty.is_err(self.syms) {
                    // Error-typed fields are represented by the zero-size
                    // `{error}` type even when the source omitted the field.
                    builder.const_zero(self.to_llvm_ty(fty).strct)
                } else {
                    let Some((_, value)) = value
                    else {
                        unreachable!("non-error struct field missing after semantic analysis");
                    };
                    *value
                }
            })
        )
    }


    fn create_enum(
        &mut self,
        builder: &mut Builder<'ctx>,
        ty: Type,
        kind: Value<'ctx>,
        value: Value<'ctx>,
        payload_ty: Type,
    ) -> Value<'ctx> {
        assert_eq!(kind.ty().kind(), TypeKind::Integer);

        let tag_val = builder.int_cast(kind.as_integer(), *self.i32, false);
        let llvm_ty = self.to_llvm_ty(ty);

        let buf = builder.alloca(llvm_ty.repr);
        let strct_ty = llvm_ty.strct.as_struct();

        let tag_ptr = builder.field_ptr(buf, strct_ty, 0);

        // The selected payload may have failed to resolve. Its LLVM slot is
        // the zero-size error type, so storing the real value would produce
        // invalid LLVM. Other variants with errored payloads remain usable.
        let type_is_error = ty.sym(self.syms).ok()
            .is_some_and(|sym| matches!(self.syms.sym(sym).kind(), SymbolKind::Error(_)));

        if type_is_error 
        || payload_ty.is_err(self.syms) {
            let zero = builder.const_zero(llvm_ty.repr);
            builder.store(buf, zero);
            builder.store(tag_ptr, tag_val);
            return builder.load(buf, llvm_ty.repr);
        }

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
        let Ok(sym_id) = ty.sym(self.syms)
        else {
            return value;
        };

        if sym_id == SymbolId::RC {
            return builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[value]);
        }

        if sym_id == SymbolId::LIST 
        || sym_id == SymbolId::LIST_ITER {

            let collection = 
            if sym_id == SymbolId::LIST {
                value.as_struct()
            } else {
                builder.field_load(value.as_struct(), 0).as_struct()
            };

            let tagged_ptr = self.collection_tagged_ptr(builder, collection);
            let (ptr, _) = self.collection_split_tag(builder, tagged_ptr);
            builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[*ptr]);

            return value;
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
                let is_closure = builder.bool_not(builder.ptr_is_null(capture_ptr));
                builder.iff(is_closure,
                    |builder| {
                        builder.call(self.rc_clone_fn.0, self.rc_clone_fn.1, &[*capture_ptr]);
                    });
                return value;
            }
            return value;
        };

        if cont.fields().is_empty() {
            return value;
        }

        let container_type_gens = ty.gens(self.syms);
        let cont_gens = self.syms.get_gens(container_type_gens);
        let llvm_ty = self.to_llvm_ty(ty);

        match cont.kind() {
            ContainerKind::Struct | ContainerKind::Tuple => {
                let struct_val = value.as_struct();
                let mut fields = Vec::with_capacity(cont.fields().len());
                for (i, (_, field_gen)) in cont.fields().iter().enumerate() {
                    let field_ty = field_gen.to_ty(cont_gens, self.syms);
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
                    let field_ty = field_gen.to_ty(cont_gens, self.syms);
                    let field_ty = field_ty.resolve(&[], self.syms);
                    if field_ty.sym(self.syms) == Ok(SymbolId::UNIT) {
                        continue;
                    }
                    let field_llvm = self.to_llvm_ty(field_ty);

                    let index = builder.const_int(self.i32, i as _, false);
                    let cond = builder.cmp_int(tag.as_integer(), index, IntCmp::Eq);

                    builder.iff(
                        cond,
                        |builder| {
                            let payload = builder.load(src_data_ptr, field_llvm.repr);
                            let copied = self.emit_copy(builder, payload, field_ty);
                            builder.store(dst_data_ptr, copied);
                        },
                    );
                }

                let tag_ptr = builder.field_ptr(buf, llvm_ty.strct.as_struct(), 0);
                builder.store(tag_ptr, tag);
                builder.load(buf, llvm_ty.repr)
            },

            ContainerKind::Generic => unreachable!(),
        }
    }

    /// Decrements the reference count at `ptr` and reports whether it reached zero.
    ///
    /// `ptr` must point to the start of an untagged reference-counted allocation.
    /// This only changes the count; it does not destroy or deallocate the allocation.
    fn emit_rc_decrement(&self, builder: &mut Builder<'ctx>, ptr: Ptr<'ctx>) -> Bool<'ctx> {
        unsafe { Bool::new(builder.call(self.rc_drop_fn.0, self.rc_drop_fn.1, &[*ptr])) }
    }

    /// Releases one ownership reference and deallocates the allocation when it
    /// reaches zero.
    ///
    /// `ptr` must be the start of an untagged allocation whose first field is
    /// the runtime reference count. `size_val` must be the exact allocation
    /// size passed to `margarineDealloc`.
    ///
    /// `on_zero` runs only when the count reaches zero. It must destroy owned
    /// child values or payload elements, but must not deallocate `ptr`; this
    /// function performs the final deallocation afterward.
    fn emit_rc_drop(
        &mut self,
        builder: &mut Builder<'ctx>,
        ptr: Ptr<'ctx>,
        size_val: Integer<'ctx>,
        on_zero: impl FnOnce(&mut Self, &mut Builder<'ctx>),
    ) {
        let is_zero = self.emit_rc_decrement(builder, ptr);
        let dealloc_fn = self.dealloc_fn;

        builder.expect(is_zero, false);
        builder.iff(is_zero,
            |builder| {
                on_zero(self, builder);
                builder.call(dealloc_fn.0, dealloc_fn.1, &[*ptr, *size_val]);
            });
    }

    fn emit_drop<'env>(
        &mut self, 
        env: &mut Env<'env, 'ctx>, 
        builder: &mut Builder<'ctx>, 
        value: Value<'ctx>, 
        ty: Type
    ) {
        let ty = ty.resolve(&[], self.syms);

        assert!(ty.is_resolved(self.syms));

        let Ok(sym_id) = ty.sym(self.syms)
        else { return; };

        if sym_id == SymbolId::RC {
            let gens_id = ty.gens(self.syms);
            let gens = self.syms.get_gens(gens_id);
            let elem_ty = gens[0].1;
            let elem_ty = elem_ty.instantiate(self.syms, 0);

            let llvm_elem = self.to_llvm_ty(elem_ty);
            let rc_ty = self.ctx.literal_struct(&[*self.usize, llvm_elem.repr], false);
            let size_val = self.const_usize(builder, rc_ty.size_of(self.module).unwrap());

            let ptr = value.as_ptr();

            self.emit_rc_drop(builder, ptr, size_val, |slf, builder| {
                let rc = builder.load(ptr, *rc_ty).as_struct();
                let data = builder.field_load(rc, 1);
                let trait_func = slf.resolve_trait_method(
                    elem_ty,
                    SymbolId::DESTROY_TRAIT,
                    StringMap::DESTROY_FUNC,
                ).map(|func| (func.func_ptr, func.func_ty));
                if let Some((func_ptr, func_ty)) = trait_func {
                    let data_copy = slf.emit_copy(builder, data, elem_ty);
                    slf.call_function(
                        env,
                        builder,
                        (func_ptr, builder.ptr_null()),
                        func_ty,
                        &[(data_copy, elem_ty, None)]
                    );
                }

                slf.emit_drop(env, builder, data, elem_ty);
            });

            return;
        }

        if sym_id == SymbolId::LIST 
        || sym_id == SymbolId::LIST_ITER {
            let collection = 
            if sym_id == SymbolId::LIST {
                value.as_struct()
            } else {
                builder.field_load(value.as_struct(), 0).as_struct()
            };

            let gens_id = ty.gens(self.syms);
            let gens = self.syms.get_gens(gens_id);
            let elem_ty = gens[0].1;
            let elem_ty = elem_ty.instantiate(self.syms, 0);

            let llvm_elem = self.to_llvm_ty(elem_ty);

            self.collection_drop(env, builder, collection, llvm_elem.repr, Some(elem_ty));
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
                let is_closure = builder.bool_not(builder.ptr_is_null(capture_ptr));
                builder.iff(is_closure,
                    |builder| {
                        let capture_header = self.ctx.literal_struct(&[*self.usize, *self.ctx.ptr()], false);
                        let drop_fn_ptr = builder.field_ptr(capture_ptr, capture_header, 1);
                        let drop_fn_val = builder.load(drop_fn_ptr, *self.ctx.ptr());
                        let drop_fn_ptr = drop_fn_val.as_func();
                        let void = self.ctx.void();
                        let drop_fn_ty = void.fn_ty(self.ctx.arena, &[*self.ctx.ptr()], false);
                        builder.call(drop_fn_ptr, drop_fn_ty, &[*capture_ptr]);
                    });
                return;
            }
            return;
        };

        if cont.fields().is_empty() {
            return;
        }

        let container_type_gens = ty.gens(self.syms);
        let cont_gens = self.syms.get_gens(container_type_gens);
        let llvm_ty = self.to_llvm_ty(ty);

        match cont.kind() {
            ContainerKind::Struct | ContainerKind::Tuple => {
                let struct_val = value.as_struct();
                for i in (0..cont.fields().len()).rev() {
                    let (_, field_gen) = cont.fields()[i];
                    let field_ty = field_gen.to_ty(cont_gens, self.syms);

                    let field_ty = field_ty.resolve(&[], self.syms);
                    let field_val = builder.field_load(struct_val, i);
                    self.emit_drop(env, builder, field_val, field_ty);
                }
            },

            ContainerKind::Enum => {
                let tag = builder.field_load(value.as_struct(), 0);
                let src_buf = builder.alloca_store(value);
                let src_data_ptr = builder.field_ptr(src_buf, llvm_ty.strct.as_struct(), 1);

                for (i, (_, field_gen)) in cont.fields().iter().enumerate() {
                    let field_ty = field_gen.to_ty(cont_gens, self.syms);
                    let field_ty = field_ty.resolve(&[], self.syms);
                    if field_ty.sym(self.syms) == Ok(SymbolId::UNIT) {
                        continue;
                    }
                    let field_llvm = self.to_llvm_ty(field_ty);

                    let index = builder.const_int(self.i32, i as _, false);
                    let cond = builder.cmp_int(tag.as_integer(), index, IntCmp::Eq);

                    builder.iff(
                        cond,
                        |builder| {
                            let payload = builder.load(src_data_ptr, field_llvm.repr);
                            self.emit_drop(env, builder, payload, field_ty);
                        },
                    );
                }
            },

            ContainerKind::Generic => unreachable!(),
        }
    }


    fn resolve_trait_method(
        &mut self,
        ty: Type,
        trait_id: SymbolId,
        func_name: StringIndex,
    ) -> Option<&Function<'ctx>> {
        let trait_ty = Type::Ty(trait_id, GenListId::EMPTY);
        let (ns, bindings) = self.syms.trait_implementation(ty, trait_ty)?;

        let func_sym = self.ns.get_ns(ns).get_sym(func_name).unwrap().ok()?;
        let func_gens = self.syms.sym(func_sym).generics();

        let mut gens = sti::vec::Vec::with_cap_in(self.syms.arena(), func_gens.len());
        for generic in func_gens {
            let (_, ty) = bindings.iter().find(|(binding, _)| binding.name == generic.name)?;
            gens.push((*generic, *ty));
        }

        let gens = self.syms.add_gens(gens.leak());

        self.get_func(Type::Ty(func_sym, gens)).ok()
    }


    fn trait_accessor_function(
        &mut self,
        receiver: Type,
        trait_pattern: Generic<'out>,
        accessor: Type,
        name: StringIndex,
        env_gens: &[&[(BoundedGeneric<'_>, Type)]],
    ) -> Option<Type> {
        let accessor_gens_id = accessor.gens(self.syms);
        let accessor_gens = self.syms.get_gens(accessor_gens_id);
        let trait_ty = trait_pattern.to_ty(accessor_gens, self.syms);
        let trait_ty = trait_ty.resolve(env_gens, self.syms);
        let (ns, bindings) = self.syms.trait_implementation(receiver, trait_ty)?;
        let func = self.ns.get_ns(ns).get_sym(name)?.ok()?;

        let implementation_gens = self.syms.sym(func).generics();
        let accessor_sym = accessor.sym(self.syms).ok()?;
        let method_gen_count =
        match self.syms.sym(accessor_sym).kind() {
            SymbolKind::Function(func) => func.declared_generics().len(),
            _ => return None,
        };
        let implementation_gen_count = implementation_gens.len().checked_sub(method_gen_count)?;
        let accessor_gens_id = accessor.gens(self.syms);
        let accessor_gens = self.syms.get_gens(accessor_gens_id);
        let method_gens = &accessor_gens[accessor_gens.len().checked_sub(method_gen_count)?..];

        let mut gens = sti::vec::Vec::with_cap_in(self.syms.arena(), implementation_gens.len());
        for implementation_gen in &implementation_gens[..implementation_gen_count] {
            let (_, ty) = bindings.iter()
                .find(|(binding, _)| binding.name == implementation_gen.name)?;
            gens.push((*implementation_gen, *ty));
        }
        gens.extend_from_slice(method_gens);

        let gens = self.syms.add_gens(gens.leak());
        Some(Type::Ty(func, gens).resolve(env_gens, self.syms))
    }




    fn call_function<'env>(
        &mut self,
        env: &mut Env<'env, 'ctx>,
        builder: &mut Builder<'ctx>,
        (func_ptr, func_env): (FunctionPtr<'ctx>, Ptr<'ctx>),
        func_ty: FunctionType<'ctx>,
        values: &[(Value<'ctx>, Type, Option<ExprId>)]
    ) -> Value<'ctx> {
        let llvm_args = sti::vec::Vec::from_in(
            self.ctx.arena,
            values.iter()
                .map(|s| s.0)
                .chain(core::iter::once(*func_env))
        );

        let result = builder.call(
            func_ptr,
            func_ty,
            &llvm_args
        );

        for (value, ty, inout_expr) in values {

            let value =
            if inout_expr.is_some() { builder.load(value.as_ptr(), self.to_llvm_ty(*ty).repr) }
            else { *value };

            if let Some(expr) = *inout_expr
            && self.is_inout_place(expr) {
                self.assign(env, builder, expr, value);
            } else {
                self.emit_drop(env, builder, value, *ty);
            }
        }

        result
    }


    fn extern_abi(&self, ret: LLVMType<'ctx>) -> ExternAbi<'ctx> {
        let uses_large_struct_sret = matches!(
            self.target,
            CompilationTarget::Arm64AppleDarwin
            | CompilationTarget::X86_64UnknownLinuxGnu
            | CompilationTarget::Aarch64UnknownLinuxGnu
        );
        if uses_large_struct_sret
        && ret.kind() == TypeKind::Struct
        && ret.size_of(self.module).is_some_and(|size| size > 16) {
            ExternAbi::SRet(ret)
        } else {
            ExternAbi::Direct
        }
    }


    fn drop_locals(&mut self, env: &mut Env<'_, 'ctx>, builder: &mut Builder<'ctx>, start: usize) {
        for i in (start..env.vars.len()).rev() {
            let (_, local, ty, borrowed) = env.vars[i];

            // Borrowed locals (inout writeback slots, closure captures) are owned by their
            // origin — the caller or the closure environment — and must not be released by
            // this function's drop protocol.
            if borrowed {
                continue;
            }

            let value = builder.local_get(local);
            self.emit_drop(env, builder, value, ty);
        }
    }

    fn drop_all_locals(&mut self, env: &mut Env<'_, 'ctx>, builder: &mut Builder<'ctx>) {
        self.drop_locals(env, builder, 0);
    }


    fn update_inouts(&mut self, env: &Env<'_, 'ctx>, builder: &mut Builder<'ctx>) {
        for (param, local) in &env.inouts {
            let value = builder.local_get(*local);
            builder.store(builder.local_get(*param).as_ptr(), value);
        }
    }


    fn is_inout_place(&self, expr: ExprId) -> bool {
        match self.ast.expr(expr) {
            Expr::Identifier(_, _) => true,
            Expr::AccessField { val, .. }
            | Expr::IndexList { list: val, .. }
            | Expr::Unwrap(val)
            | Expr::OrReturn(val) => self.is_inout_place(val),
            _ => false,
        }
    }




    fn eq<'env>(
        &mut self,
        env: &mut Env<'env, 'ctx>,
        builder: &mut Builder<'ctx>,
        ty: Type,
        accum: Local,
        lhs: Value<'ctx>,
        rhs: Value<'ctx>,
    ) {

        assert!(ty.is_resolved(self.syms));

        let sym = ty.sym(self.syms).unwrap();

        match sym {
            SymbolId::I64 | SymbolId::BYTE => {

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
                let b = self.resolve_trait_method(
                    ty, 
                    SymbolId::EQ_TRAIT, 
                    StringMap::EQ_FUNC,
                );


                let b = b.expect("semantic analysis must ensure eq doesn't process");

                let func_ptr = b.func_ptr;
                let func_ty = b.func_ty;

                let b =
                self.call_function(
                    env, 
                    builder, 
                    (func_ptr, builder.ptr_null()), 
                    func_ty, 
                    &[
                        (lhs, ty, None),
                        (rhs, ty, None),
                    ]
                );


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

#[cfg(test)]
mod tests {
    use super::CompilationTarget;

    #[test]
    fn linux_targets_expose_platform_triples() {
        let x86 = CompilationTarget::try_from("x86_64-unknown-linux-gnu").unwrap();
        assert_eq!(x86.margarine_target_triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(x86.llvm_target_triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(x86.c_target_triple(), "x86_64-unknown-linux-gnu");
        assert_eq!(x86.shared_library_suffix(), "so");

        let arm = CompilationTarget::try_from("aarch64-unknown-linux-gnu").unwrap();
        assert_eq!(arm.margarine_target_triple(), "aarch64-unknown-linux-gnu");
        assert_eq!(arm.llvm_target_triple(), "aarch64-unknown-linux-gnu");
        assert_eq!(arm.c_target_triple(), "aarch64-unknown-linux-gnu");
        assert_eq!(arm.shared_library_suffix(), "so");
    }
}





impl Env<'_, '_> {
    pub fn alloc_var(&mut self, name: StringIndex, local: Local, ty: Type, borrowed: bool) {
        self.vars.push((name, local, ty, borrowed));
    }


    pub fn find_var(&self, name: StringIndex) -> Option<Local> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.1)
    }


    pub fn find_var_ty(&self, name: StringIndex) -> Option<Type> {
        self.vars.iter().rev().find(|x| x.0 == name).map(|x| x.2)
    }
}
