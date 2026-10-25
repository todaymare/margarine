use common::string_map::StringIndex;

use llvm_api::{
    attributes::{AbiAttribute, AttributeKind, AttributeLocation},
    builder::{Builder, IntCmp},
    ctx::ContextRef,
    module::Module,
    tys::{func::FunctionType, Type, TypeKind},
    values::{func::FunctionPtr, ptr::Ptr, Value},
};

use parser::nodes::{decl::DeclId, expr::ExprId};


use crate::{
    abi::{DefinitionBinding, StartupArg},
    llvm_codegen::{CompilationTarget, Conversion, Env},
    syms::{containers::ContainerKind, func::FunctionArgument, ty::Type as SemanticType, SymbolKind},
};

#[derive(Clone, Debug)]
pub(crate) struct LoweredSignature<'ctx> {
    pub function_type: FunctionType<'ctx>,
    pub return_mode: ReturnMode<'ctx>,
    pub arguments: Vec<ArgumentMode<'ctx>>,
    pub attributes: Vec<AbiAttribute<'ctx>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ReturnMode<'ctx> {
    Ignore { original: Type<'ctx> },
    Direct { original: Type<'ctx>, passed_as: Type<'ctx> },
    Indirect { original: Type<'ctx>, alignment: u32 },
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum ArgumentMode<'ctx> {
    Ignore { original: Type<'ctx> },
    Direct { original: Type<'ctx>, passed_as: Type<'ctx> },
    Expand { original: Type<'ctx>, parts: Vec<AbiPart<'ctx>> },
    Indirect { original: Type<'ctx>, byval: bool, alignment: u32 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct AbiPart<'ctx> {
    pub ty: Type<'ctx>,
    pub byte_offset: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Leaf<'ctx> {
    pub ty: Type<'ctx>,
    pub offset: u64,
}

fn requires_zero_extension(ty: Type<'_>) -> bool {
    ty.kind() == TypeKind::Integer
        && matches!(ty.as_integer().bit_size(), 1 | 8)
}


/// Asserts that `ty` contains only supported Margarine source leaves:
/// i1 (bool), i8 (byte), i32 (discriminant), i64 (int/metadata), target-width
/// int, double (f64), opaque ptr, and arrays/structs composed from those leaves.
/// No i16, float, vector, x87, or variadic is emitted by source Margarine.
pub(crate) fn assert_valid_source_type<'ctx>(ty: Type<'ctx>, module: Module<'ctx>) {
    match ty.kind() {
        TypeKind::Void => {}
        TypeKind::Integer => {
            let bits = ty.as_integer().bit_size();
            assert!(
                bits == 1 || bits == 8 || bits == 32 || bits == 64,
                "unsupported source integer bit size: {bits}"
            );
        }
        TypeKind::F64 => {}
        TypeKind::Ptr => {}
        TypeKind::Struct => {
            let strct = ty.as_struct();
            assert!(!strct.is_opaque(), "opaque struct cannot cross C boundary");
            for i in 0..strct.fields_count() {
                assert_valid_source_type(strct.field_at(i).unwrap(), module);
            }
        }
        TypeKind::Array => {
            let arr = ty.as_array();
            assert_valid_source_type(arr.element_ty(), module);
        }
        TypeKind::F32 => panic!("source Margarine does not emit f32/float leaves"),
        TypeKind::Vector => panic!("source Margarine does not emit vector leaves"),
        TypeKind::Function => panic!("bare function types do not cross boundary directly"),
    }
}

pub(crate) fn flatten_leaves<'ctx>(
    ty: Type<'ctx>,
    base_offset: u64,
    module: Module<'ctx>,
    leaves: &mut Vec<Leaf<'ctx>>,
) {
    match ty.kind() {
        TypeKind::Void => {}
        TypeKind::Integer | TypeKind::F64 | TypeKind::Ptr | TypeKind::F32 => {
            leaves.push(Leaf { ty, offset: base_offset });
        }
        TypeKind::Struct => {
            let strct = ty.as_struct();
            assert!(!strct.is_opaque(), "opaque struct cannot be flattened");
            for i in 0..strct.fields_count() {
                let field_ty = strct.field_at(i).unwrap();
                let field_offset = strct.offset_of_element(module, i).unwrap();
                flatten_leaves(field_ty, base_offset + field_offset, module, leaves);
            }
        }
        TypeKind::Array => {
            let arr = ty.as_array();
            let elem_ty = arr.element_ty();
            let count = arr.element_count();
            let elem_size = elem_ty.size_of(module).unwrap() as u64;
            for i in 0..count {
                flatten_leaves(elem_ty, base_offset + i * elem_size, module, leaves);
            }
        }
        TypeKind::Vector => {
            leaves.push(Leaf { ty, offset: base_offset });
        }
        TypeKind::Function => panic!("cannot flatten function type"),
    }
}

/// AArch64 ABIInfo::isHomogeneousAggregate
/// Exactly 1-4 identical double leaves, every leaf naturally aligned (offset % 8 == 0),
/// and member_count * 8 == aggregate_size (no padding).
pub(crate) fn is_aarch64_hfa<'ctx>(ty: Type<'ctx>, module: Module<'ctx>) -> Option<u32> {
    if !matches!(ty.kind(), TypeKind::Struct | TypeKind::Array) {
        return None;
    }
    let mut leaves = Vec::new();
    flatten_leaves(ty, 0, module, &mut leaves);
    if leaves.is_empty() || leaves.len() > 4 {
        return None;
    }
    let total_size = ty.size_of(module).unwrap() as u64;
    if total_size != leaves.len() as u64 * 8 {
        return None;
    }
    for (idx, leaf) in leaves.iter().enumerate() {
        if leaf.ty.kind() != TypeKind::F64 {
            return None;
        }
        if leaf.offset != idx as u64 * 8 {
            return None;
        }
    }
    Some(leaves.len() as u32)
}

/// WebAssemblyABIInfo / CodeGen::isSingleElementStruct
/// Recursively unwrap structs and one-element arrays only when there is exactly
/// one non-empty scalar leaf and scalar_size == complete_aggregate_size.
pub(crate) fn wasm_single_element_scalar<'ctx>(
    ty: Type<'ctx>,
    module: Module<'ctx>,
) -> Option<Type<'ctx>> {
    if !matches!(ty.kind(), TypeKind::Struct | TypeKind::Array) {
        return None;
    }
    let total_size = ty.size_of(module).unwrap() as u64;
    if total_size == 0 {
        return None;
    }
    let mut leaves = Vec::new();
    flatten_leaves(ty, 0, module, &mut leaves);
    if leaves.len() == 1 {
        let leaf_size = leaves[0].ty.size_of(module).unwrap() as u64;
        if leaf_size == total_size {
            return Some(leaves[0].ty);
        }
    }
    None
}

// -----------------------------------------------------------------------------
// SysV x86-64 Classification
// -----------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SysVClass {
    NoClass,
    Integer,
    Sse,
    Memory,
}

pub(crate) fn sysv_merge(current: SysVClass, new_class: SysVClass) -> SysVClass {
    if current == new_class {
        current
    } else if current == SysVClass::NoClass {
        new_class
    } else if new_class == SysVClass::NoClass {
        current
    } else if current == SysVClass::Memory || new_class == SysVClass::Memory {
        SysVClass::Memory
    } else if current == SysVClass::Integer || new_class == SysVClass::Integer {
        SysVClass::Integer
    } else {
        SysVClass::Sse
    }
}

pub(crate) fn sysv_classify_leaves<'ctx>(
    leaves: &[Leaf<'ctx>],
    total_size: u64,
    module: Module<'ctx>,
) -> (SysVClass, SysVClass) {
    if total_size == 0 {
        return (SysVClass::NoClass, SysVClass::NoClass);
    }
    if total_size > 16 {
        return (SysVClass::Memory, SysVClass::Memory);
    }

    let mut lo = SysVClass::NoClass;
    let mut hi = SysVClass::NoClass;

    for leaf in leaves {
        let leaf_size = leaf.ty.size_of(module).unwrap() as u64;
        let leaf_end = leaf.offset + leaf_size;

        // Leaf spanning across 8-byte boundary is unaligned -> Memory
        if leaf.offset < 8 && leaf_end > 8 {
            return (SysVClass::Memory, SysVClass::Memory);
        }

        let leaf_class = match leaf.ty.kind() {
            TypeKind::Integer | TypeKind::Ptr => SysVClass::Integer,
            TypeKind::F64 | TypeKind::F32 => SysVClass::Sse,
            _ => SysVClass::Memory,
        };

        if leaf.offset < 8 {
            lo = sysv_merge(lo, leaf_class);
        } else {
            hi = sysv_merge(hi, leaf_class);
        }
    }

    // Clang post-merge: If any eightbyte is Memory, the whole aggregate is Memory.
    if lo == SysVClass::Memory || hi == SysVClass::Memory {
        lo = SysVClass::Memory;
        hi = SysVClass::Memory;
    }

    (lo, hi)
}

/// Port of Clang's GetINTEGERTypeAtOffset and GetSSETypeAtOffset
pub(crate) fn sysv_build_eightbyte_carrier<'ctx>(
    class: SysVClass,
    eightbyte_idx: usize,
    total_size: u64,
    leaves: &[Leaf<'ctx>],
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
) -> Type<'ctx> {
    let start_offset = (eightbyte_idx * 8) as u64;
    let end_offset = total_size.min(start_offset + 8);
    let eightbyte_size = end_offset.saturating_sub(start_offset);

    let matching_leaves: Vec<&Leaf<'ctx>> = leaves
        .iter()
        .filter(|l| l.offset >= start_offset && l.offset < start_offset + 8)
        .collect();

    match class {
        SysVClass::Integer => {
            // Preserve ptr or exact integer if it starts at start_offset and matches eightbyte size
            if matching_leaves.len() == 1 {
                let leaf = matching_leaves[0];
                let leaf_size = leaf.ty.size_of(module).unwrap() as u64;
                if leaf.offset == start_offset && leaf_size == eightbyte_size {
                    if leaf.ty.kind() == TypeKind::Ptr {
                        return *ctx.ptr();
                    }
                    if leaf.ty.kind() == TypeKind::Integer {
                        return leaf.ty;
                    }
                }
            }
            *ctx.integer((eightbyte_size * 8) as u32)
        }
        SysVClass::Sse => {
            // Check for packed floats: e.g. <2 x float>
            if matching_leaves.len() == 2
                && matching_leaves.iter().all(|l| l.ty.kind() == TypeKind::F32)
            {
                return *ctx.vector(*ctx.f32(), 2);
            }
            if matching_leaves.len() == 1 && matching_leaves[0].ty.kind() == TypeKind::F32 {
                return *ctx.f32();
            }
            *ctx.f64()
        }
        _ => *ctx.integer((eightbyte_size * 8) as u32),
    }
}

// -----------------------------------------------------------------------------
// Target Classifiers
// -----------------------------------------------------------------------------

pub(crate) fn lower_signature<'ctx>(
    target: CompilationTarget,
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
    ret: Type<'ctx>,
    args: &[Type<'ctx>],
) -> LoweredSignature<'ctx> {
    assert_valid_source_type(ret, module);
    for arg in args {
        assert_valid_source_type(*arg, module);
    }

    match target {
        CompilationTarget::Arm64AppleDarwin => classify_darwin_pcs(ctx, module, ret, args),
        CompilationTarget::Aarch64UnknownLinuxGnu => classify_linux_aapcs(ctx, module, ret, args),
        CompilationTarget::X86_64UnknownLinuxGnu => classify_x86_64_sysv(ctx, module, ret, args),
        CompilationTarget::Wasm32UnknownUnknown => classify_wasm32(ctx, module, ret, args),
    }
}

/// AArch64 Darwin PCS
fn classify_darwin_pcs<'ctx>(
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
    ret: Type<'ctx>,
    args: &[Type<'ctx>],
) -> LoweredSignature<'ctx> {
    let mut attributes = Vec::new();

    // 1. Return classification
    let return_mode = if let Some(_) = is_aarch64_hfa(ret, module) {
        ReturnMode::Direct { original: ret, passed_as: ret }
    } else if matches!(ret.kind(), TypeKind::Struct | TypeKind::Array) {
        let size_bytes = ret.size_of(module).unwrap() as u64;
        let align_bytes = ret.align_of(module).unwrap() as u64;
        let size_bits = size_bytes * 8;
        let align_bits = align_bytes * 8;

        if size_bits == 0 {
            ReturnMode::Ignore { original: ret }
        } else if size_bits <= 64 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.integer(size_bits as u32),
            }
        } else if size_bits <= 128 && align_bits < 128 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.array(*ctx.integer(64), 2),
            }
        } else if size_bits <= 128 && align_bits >= 128 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.integer(128),
            }
        } else {
            ReturnMode::Indirect {
                original: ret,
                alignment: align_bytes as u32,
            }
        }
    } else if ret.kind() == TypeKind::Void {
        ReturnMode::Ignore { original: ret }
    } else {
        if requires_zero_extension(ret) {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Return,
                AttributeKind::Enum { name: "zeroext", value: 0 },
            ));
        }
        ReturnMode::Direct { original: ret, passed_as: ret }
    };

    // 2. Argument classification
    let mut argument_modes = Vec::with_capacity(args.len());
    let mut physical_param_index = match &return_mode {
        ReturnMode::Indirect { original, alignment } => {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Type { name: "sret", ty: *original },
            ));
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Enum { name: "align", value: *alignment as u64 },
            ));
            1
        }
        _ => 0,
    };

    for arg in args {
        let arg = *arg;
        if let Some(hfa_count) = is_aarch64_hfa(arg, module) {
            let carrier = *ctx.array(*ctx.f64(), hfa_count as usize);
            argument_modes.push(ArgumentMode::Direct {
                original: arg,
                passed_as: carrier,
            });
            physical_param_index += 1;
        } else if matches!(arg.kind(), TypeKind::Struct | TypeKind::Array) {
            let size_bytes = arg.size_of(module).unwrap() as u64;
            let align_bytes = arg.align_of(module).unwrap() as u64;
            let size_bits = size_bytes * 8;
            let align_bits = align_bytes * 8;

            if size_bits == 0 {
                argument_modes.push(ArgumentMode::Ignore { original: arg });
            } else if size_bits > 128 {
                argument_modes.push(ArgumentMode::Indirect {
                    original: arg,
                    byval: false,
                    alignment: align_bytes as u32,
                });
                physical_param_index += 1;
            } else {
                let alignment_bits = align_bits.max(64);
                let rounded_size =
                    (size_bits + alignment_bits - 1) / alignment_bits * alignment_bits;
                let base = *ctx.integer(alignment_bits as u32);
                let carrier = if rounded_size == alignment_bits {
                    base
                } else {
                    *ctx.array(base, (rounded_size / alignment_bits) as usize)
                };
                argument_modes.push(ArgumentMode::Direct {
                    original: arg,
                    passed_as: carrier,
                });
                physical_param_index += 1;
            }
        } else if arg.kind() == TypeKind::Void {
            argument_modes.push(ArgumentMode::Ignore { original: arg });
        } else {
            if requires_zero_extension(arg) {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "zeroext", value: 0 },
                ));
            }
            argument_modes.push(ArgumentMode::Direct { original: arg, passed_as: arg });
            physical_param_index += 1;
        }
    }

    build_lowered_signature(ctx, return_mode, argument_modes, attributes)
}

/// AArch64 Linux AAPCS
fn classify_linux_aapcs<'ctx>(
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
    ret: Type<'ctx>,
    args: &[Type<'ctx>],
) -> LoweredSignature<'ctx> {
    let mut attributes = Vec::new();

    // 1. Return classification
    let return_mode = if let Some(_) = is_aarch64_hfa(ret, module) {
        ReturnMode::Direct { original: ret, passed_as: ret }
    } else if matches!(ret.kind(), TypeKind::Struct | TypeKind::Array) {
        let size_bytes = ret.size_of(module).unwrap() as u64;
        let align_bytes = ret.align_of(module).unwrap() as u64;
        let size_bits = size_bytes * 8;
        let align_bits = align_bytes * 8;

        if size_bits == 0 {
            ReturnMode::Ignore { original: ret }
        } else if size_bits <= 64 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.integer(size_bits as u32),
            }
        } else if size_bits <= 128 && align_bits < 128 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.array(*ctx.integer(64), 2),
            }
        } else if size_bits <= 128 && align_bits >= 128 {
            ReturnMode::Direct {
                original: ret,
                passed_as: *ctx.integer(128),
            }
        } else {
            ReturnMode::Indirect {
                original: ret,
                alignment: align_bytes as u32,
            }
        }
    } else if ret.kind() == TypeKind::Void {
        ReturnMode::Ignore { original: ret }
    } else {
        // AAPCS Linux does NOT apply zeroext to i1
        ReturnMode::Direct { original: ret, passed_as: ret }
    };

    // 2. Argument classification
    let mut argument_modes = Vec::with_capacity(args.len());
    let mut physical_param_index = match &return_mode {
        ReturnMode::Indirect { original, alignment } => {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Type { name: "sret", ty: *original },
            ));
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Enum { name: "align", value: *alignment as u64 },
            ));
            1
        }
        _ => 0,
    };

    for arg in args {
        let arg = *arg;
        if let Some(hfa_count) = is_aarch64_hfa(arg, module) {
            let carrier = *ctx.array(*ctx.f64(), hfa_count as usize);
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(physical_param_index),
                AttributeKind::Enum { name: "alignstack", value: 8 },
            ));
            argument_modes.push(ArgumentMode::Direct {
                original: arg,
                passed_as: carrier,
            });
            physical_param_index += 1;
        } else if matches!(arg.kind(), TypeKind::Struct | TypeKind::Array) {
            let size_bytes = arg.size_of(module).unwrap() as u64;
            let align_bytes = arg.align_of(module).unwrap() as u64;
            let size_bits = size_bytes * 8;

            if size_bits == 0 {
                argument_modes.push(ArgumentMode::Ignore { original: arg });
            } else if size_bits > 128 {
                argument_modes.push(ArgumentMode::Indirect {
                    original: arg,
                    byval: false,
                    alignment: align_bytes as u32,
                });
                physical_param_index += 1;
            } else {
                let alignment_bits = if align_bytes < 16 { 64 } else { 128 };
                let rounded_size =
                    (size_bits + alignment_bits - 1) / alignment_bits * alignment_bits;
                let base = *ctx.integer(alignment_bits as u32);
                let carrier = if rounded_size == alignment_bits {
                    base
                } else {
                    *ctx.array(base, (rounded_size / alignment_bits) as usize)
                };
                argument_modes.push(ArgumentMode::Direct {
                    original: arg,
                    passed_as: carrier,
                });
                physical_param_index += 1;
            }
        } else if arg.kind() == TypeKind::Void {
            argument_modes.push(ArgumentMode::Ignore { original: arg });
        } else {
            // Linux AAPCS has no zeroext on i1
            argument_modes.push(ArgumentMode::Direct { original: arg, passed_as: arg });
            physical_param_index += 1;
        }
    }

    build_lowered_signature(ctx, return_mode, argument_modes, attributes)
}

/// x86_64 SysV
fn classify_x86_64_sysv<'ctx>(
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
    ret: Type<'ctx>,
    args: &[Type<'ctx>],
) -> LoweredSignature<'ctx> {
    let mut attributes = Vec::new();
    let mut remaining_int = 6usize;
    let mut remaining_sse = 8usize;

    // 1. Return classification
    let return_mode = if matches!(ret.kind(), TypeKind::Struct | TypeKind::Array) {
        let total_size = ret.size_of(module).unwrap() as u64;
        let align = ret.align_of(module).unwrap() as u32;

        if total_size == 0 {
            ReturnMode::Ignore { original: ret }
        } else {
            let mut leaves = Vec::new();
            flatten_leaves(ret, 0, module, &mut leaves);
            let (lo, hi) = sysv_classify_leaves(&leaves, total_size, module);

            if lo == SysVClass::Memory {
                remaining_int = remaining_int.saturating_sub(1);
                ReturnMode::Indirect {
                    original: ret,
                    alignment: align,
                }
            } else {
                let carrier0 = sysv_build_eightbyte_carrier(lo, 0, total_size, &leaves, ctx, module);
                let passed_as = if total_size > 8 {
                    let carrier1 = sysv_build_eightbyte_carrier(hi, 1, total_size, &leaves, ctx, module);
                    *ctx.literal_struct(&[carrier0, carrier1], false)
                } else {
                    carrier0
                };
                ReturnMode::Direct { original: ret, passed_as }
            }
        }
    } else if ret.kind() == TypeKind::Void {
        ReturnMode::Ignore { original: ret }
    } else {
        if requires_zero_extension(ret) {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Return,
                AttributeKind::Enum { name: "zeroext", value: 0 },
            ));
        }
        ReturnMode::Direct { original: ret, passed_as: ret }
    };

    // 2. Argument classification
    let mut argument_modes = Vec::with_capacity(args.len());
    let mut physical_param_index = match &return_mode {
        ReturnMode::Indirect { original, alignment } => {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Type { name: "sret", ty: *original },
            ));
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Enum { name: "align", value: *alignment as u64 },
            ));
            1
        }
        _ => 0,
    };

    for arg in args {
        let arg = *arg;
        if matches!(arg.kind(), TypeKind::Struct | TypeKind::Array) {
            let total_size = arg.size_of(module).unwrap() as u64;
            let align = arg.align_of(module).unwrap() as u32;
            let byval_align = align.max(8);

            if total_size == 0 {
                argument_modes.push(ArgumentMode::Ignore { original: arg });
                continue;
            }

            let mut leaves = Vec::new();
            flatten_leaves(arg, 0, module, &mut leaves);
            let (lo, hi) = sysv_classify_leaves(&leaves, total_size, module);

            if lo == SysVClass::Memory {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Type { name: "byval", ty: arg },
                ));
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "align", value: byval_align as u64 },
                ));
                argument_modes.push(ArgumentMode::Indirect {
                    original: arg,
                    byval: true,
                    alignment: byval_align,
                });
                physical_param_index += 1;
            } else {
                let mut needed_int = 0;
                let mut needed_sse = 0;
                match lo {
                    SysVClass::Integer => needed_int += 1,
                    SysVClass::Sse => needed_sse += 1,
                    _ => {}
                }
                if total_size > 8 {
                    match hi {
                        SysVClass::Integer => needed_int += 1,
                        SysVClass::Sse => needed_sse += 1,
                        _ => {}
                    }
                }

                if needed_int <= remaining_int && needed_sse <= remaining_sse {
                    remaining_int -= needed_int;
                    remaining_sse -= needed_sse;

                    let carrier0 = sysv_build_eightbyte_carrier(lo, 0, total_size, &leaves, ctx, module);
                    let mut parts = vec![AbiPart { ty: carrier0, byte_offset: 0 }];
                    physical_param_index += 1;

                    if total_size > 8 {
                        let carrier1 = sysv_build_eightbyte_carrier(hi, 1, total_size, &leaves, ctx, module);
                        parts.push(AbiPart { ty: carrier1, byte_offset: 8 });
                        physical_param_index += 1;
                    }
                    argument_modes.push(ArgumentMode::Expand { original: arg, parts });
                } else {
                    // Register exhaustion: check Clang's exception rule:
                    // when no integer register remains, effective alignment is 8 bytes,
                    // and aggregate size is at most 8 bytes, use a direct exact-width integer stack carrier.
                    if remaining_int == 0 && align <= 8 && total_size <= 8 {
                        let carrier = *ctx.integer((total_size * 8) as u32);
                        argument_modes.push(ArgumentMode::Direct {
                            original: arg,
                            passed_as: carrier,
                        });
                        physical_param_index += 1;
                    } else {
                        attributes.push(AbiAttribute::new(
                            AttributeLocation::Parameter(physical_param_index),
                            AttributeKind::Type { name: "byval", ty: arg },
                        ));
                        attributes.push(AbiAttribute::new(
                            AttributeLocation::Parameter(physical_param_index),
                            AttributeKind::Enum { name: "align", value: byval_align as u64 },
                        ));
                        argument_modes.push(ArgumentMode::Indirect {
                            original: arg,
                            byval: true,
                            alignment: byval_align,
                        });
                        physical_param_index += 1;
                    }
                }
            }
        } else if arg.kind() == TypeKind::Void {
            argument_modes.push(ArgumentMode::Ignore { original: arg });
        } else {
            if requires_zero_extension(arg) {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "zeroext", value: 0 },
                ));
            }

            match arg.kind() {
                TypeKind::Integer | TypeKind::Ptr if remaining_int > 0 => {
                    remaining_int -= 1;
                }
                TypeKind::F64 if remaining_sse > 0 => {
                    remaining_sse -= 1;
                }
                _ => {}
            }
            argument_modes.push(ArgumentMode::Direct { original: arg, passed_as: arg });
            physical_param_index += 1;
        }
    }

    build_lowered_signature(ctx, return_mode, argument_modes, attributes)
}

/// wasm32-unknown-unknown
fn classify_wasm32<'ctx>(
    ctx: ContextRef<'ctx>,
    module: Module<'ctx>,
    ret: Type<'ctx>,
    args: &[Type<'ctx>],
) -> LoweredSignature<'ctx> {
    let mut attributes = Vec::new();

    // 1. Return classification
    let return_mode = if let Some(unwrapped) = wasm_single_element_scalar(ret, module) {
        if requires_zero_extension(unwrapped) {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Return,
                AttributeKind::Enum { name: "zeroext", value: 0 },
            ));
        }
        ReturnMode::Direct { original: ret, passed_as: unwrapped }
    } else if matches!(ret.kind(), TypeKind::Struct | TypeKind::Array) {
        let size = ret.size_of(module).unwrap() as u64;
        let align = ret.align_of(module).unwrap() as u32;
        if size == 0 {
            ReturnMode::Ignore { original: ret }
        } else {
            ReturnMode::Indirect {
                original: ret,
                alignment: align,
            }
        }
    } else if ret.kind() == TypeKind::Void {
        ReturnMode::Ignore { original: ret }
    } else {
        if requires_zero_extension(ret) {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Return,
                AttributeKind::Enum { name: "zeroext", value: 0 },
            ));
        }
        ReturnMode::Direct { original: ret, passed_as: ret }
    };

    // 2. Argument classification
    let mut argument_modes = Vec::with_capacity(args.len());
    let mut physical_param_index = match &return_mode {
        ReturnMode::Indirect { original, alignment } => {
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Type { name: "sret", ty: *original },
            ));
            attributes.push(AbiAttribute::new(
                AttributeLocation::Parameter(0),
                AttributeKind::Enum { name: "align", value: *alignment as u64 },
            ));
            1
        }
        _ => 0,
    };

    for arg in args {
        let arg = *arg;
        if let Some(unwrapped) = wasm_single_element_scalar(arg, module) {
            if requires_zero_extension(unwrapped) {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "zeroext", value: 0 },
                ));
            }
            argument_modes.push(ArgumentMode::Direct { original: arg, passed_as: unwrapped });
            physical_param_index += 1;
        } else if matches!(arg.kind(), TypeKind::Struct | TypeKind::Array) {
            let size = arg.size_of(module).unwrap() as u64;
            let align = arg.align_of(module).unwrap() as u32;
            if size == 0 {
                argument_modes.push(ArgumentMode::Ignore { original: arg });
            } else {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Type { name: "byval", ty: arg },
                ));
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "align", value: align as u64 },
                ));
                argument_modes.push(ArgumentMode::Indirect {
                    original: arg,
                    byval: true,
                    alignment: align,
                });
                physical_param_index += 1;
            }
        } else if arg.kind() == TypeKind::Void {
            argument_modes.push(ArgumentMode::Ignore { original: arg });
        } else {
            if requires_zero_extension(arg) {
                attributes.push(AbiAttribute::new(
                    AttributeLocation::Parameter(physical_param_index),
                    AttributeKind::Enum { name: "zeroext", value: 0 },
                ));
            }
            argument_modes.push(ArgumentMode::Direct { original: arg, passed_as: arg });
            physical_param_index += 1;
        }
    }

    build_lowered_signature(ctx, return_mode, argument_modes, attributes)
}


fn build_lowered_signature<'ctx>(
    ctx: ContextRef<'ctx>,
    return_mode: ReturnMode<'ctx>,
    argument_modes: Vec<ArgumentMode<'ctx>>,
    attributes: Vec<AbiAttribute<'ctx>>,
) -> LoweredSignature<'ctx> {
    let phys_ret = match &return_mode {
        ReturnMode::Ignore { .. } => *ctx.void(),
        ReturnMode::Direct { passed_as, .. } => *passed_as,
        ReturnMode::Indirect { .. } => *ctx.void(),
    };

    let mut phys_params = Vec::new();
    if let ReturnMode::Indirect { .. } = &return_mode {
        phys_params.push(*ctx.ptr());
    }

    for arg in &argument_modes {
        match arg {
            ArgumentMode::Ignore { .. } => {}
            ArgumentMode::Direct { passed_as, .. } => {
                phys_params.push(*passed_as);
            }
            ArgumentMode::Expand { parts, .. } => {
                for part in parts {
                    phys_params.push(part.ty);
                }
            }
            ArgumentMode::Indirect { .. } => {
                phys_params.push(*ctx.ptr());
            }
        }
    }

    let function_type = phys_ret.fn_ty(ctx.arena, &phys_params, false);

    LoweredSignature {
        function_type,
        return_mode,
        arguments: argument_modes,
        attributes,
    }
}

// -----------------------------------------------------------------------------
// C ABI lowering and calls
// -----------------------------------------------------------------------------

pub(crate) fn lower_callable<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    ret: SemanticType,
    args: &[(SemanticType, bool)],
) -> LoweredSignature<'ctx> {
    let boundary_ret = boundary_type(conv, ret);
    let mut source_args = std::vec::Vec::with_capacity(args.len());
    for &(arg, inout) in args {
        let ty = if inout {
            *conv.llvm_context().ptr()
        } else {
            boundary_type(conv, arg)
        };
        source_args.push(ty);
    }

    lower_signature(
        conv.target(),
        conv.llvm_context(),
        conv.llvm_module(),
        boundary_ret,
        &source_args,
    )
}

fn boundary_type<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    ty: SemanticType,
) -> Type<'ctx> {
    let symbols = conv.semantic_symbols();
    let ty = ty.resolve(&[], symbols);
    let hash = ty.hash(symbols);
    if let Some(&mapped) = conv.abi_types.c.get(&hash) {
        return mapped;
    }

    let logical = conv.logical_llvm_type(ty);
    let boundary = if ty == SemanticType::BOOL {
        *conv.llvm_context().bool()
    } else if logical.kind() != TypeKind::Struct {
        logical
    } else {
        let symbols = conv.semantic_symbols();
        let symbol = ty.sym(symbols).unwrap();
        let kind = symbols.sym(symbol).kind();
        let gens_id = ty.gens(symbols);
        let gens = symbols.get_gens(gens_id);
        match kind {
            SymbolKind::Container(container)
                if matches!(container.kind(), ContainerKind::Struct | ContainerKind::Tuple) =>
            {
                let fields = container
                    .fields()
                    .iter()
                    .map(|(_, field)| {
                        let field_ty = field.to_ty(gens, conv.semantic_symbols());
                        boundary_type(conv, field_ty)
                    })
                    .collect::<Vec<_>>();
                let logical_struct = logical.as_struct();
                if fields.iter().enumerate().all(|(index, field)| {
                    logical_struct.field_at(index) == Some(*field)
                }) {
                    logical
                } else {
                    *conv.llvm_context().literal_struct(&fields, false)
                }
            }
            _ => logical,
        }
    };
    conv.abi_types.c.insert(hash, boundary);
    boundary
}
pub(crate) fn function_value(function: FunctionPtr<'_>) -> Value<'_> {
    *function
}

// The only source-layout change at the C boundary is bool, recursively inside
// aggregates. Leave identical subtrees untouched, including their ownership.
fn convert_value<'ctx>(
    builder: &Builder<'ctx>,
    value: Value<'ctx>,
    target: Type<'ctx>,
) -> Value<'ctx> {
    let source = value.ty();
    if source == target {
        return value;
    }
    if target.kind() == TypeKind::Integer {
        let tag = builder.field_load(value.as_struct(), 0).as_integer();
        let zero = builder.const_int(tag.ty(), 0, false);
        return *builder.cmp_int(tag, zero, IntCmp::Ne);
    }
    let target = target.as_struct();
    if source.kind() == TypeKind::Integer {
        let tag = builder.int_cast(value.as_integer(), target.field_at(0).unwrap(), false);
        let payload = builder.const_zero(target.field_at(1).unwrap());
        return *builder.struct_instance(target, [tag, payload]);
    }
    let source_type = source.as_struct();
    let source = builder.alloca_store(value);
    *builder.struct_instance(
        target,
        (0..target.fields_count()).map(|index| {
            let ptr = builder.field_ptr(source, source_type, index);
            let value = builder.load(ptr, source_type.field_at(index).unwrap());
            convert_value(builder, value, target.field_at(index).unwrap())
        }),
    )
}

fn to_boundary<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    ty: SemanticType,
    value: Value<'ctx>,
) -> Value<'ctx> {
    convert_value(builder, value, boundary_type(conv, ty))
}

fn from_boundary<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    ty: SemanticType,
    value: Value<'ctx>,
) -> Value<'ctx> {
    convert_value(builder, value, conv.logical_llvm_type(ty))
}

pub(super) fn emit_inout_writeback<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    ty: SemanticType,
    destination: Ptr<'ctx>,
    value: Value<'ctx>,
) {
    let value = to_boundary(conv, builder, ty, value);
    builder.store(destination, value);
}


fn coerce<'ctx>(
    conv: &Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    value: Value<'ctx>,
    source_ty: Type<'ctx>,
    target_ty: Type<'ctx>,
) -> Value<'ctx> {
    if source_ty == target_ty {
        return value;
    }
    let source = builder.alloca_store(value);
    let target = builder.alloca(target_ty);
    builder.store(target, builder.const_zero(target_ty));
    let source_size = source_ty.size_of(conv.llvm_module()).unwrap() as u64;
    let target_size = target_ty.size_of(conv.llvm_module()).unwrap() as u64;
    builder.copy_bytes(target, source, source_size.min(target_size));
    builder.load(target, target_ty)
}

pub(crate) fn configure<'ctx>(
    ctx: ContextRef<'ctx>,
    function: FunctionPtr<'ctx>,
    signature: &LoweredSignature<'ctx>,
) {
    function.set_abi_attributes(ctx, &signature.attributes);
    function.set_c_calling_convention();
}

pub(crate) fn definition_name(
    conv: &Conversion<'_, '_, '_, '_, '_>,
    default: StringIndex,
    decl: DeclId,
) -> StringIndex {
    conv.public_function_name(decl).unwrap_or(default)
}

fn sret_ptr<'ctx>(
    builder: &Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
) -> Option<Ptr<'ctx>> {
    if !matches!(signature.return_mode, ReturnMode::Indirect { .. }) {
        return None;
    }
    Some(builder.local_get(builder.arg(0).unwrap()).as_ptr())
}

fn sret_ptr_is_present(signature: &LoweredSignature<'_>) -> bool {
    matches!(signature.return_mode, ReturnMode::Indirect { .. })
}

pub(crate) fn bind_definition<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    env: &mut Env<'_, 'ctx>,
    builder: &mut Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
    args: &[FunctionArgument<'_>],
    arg_types: &[SemanticType],
) -> DefinitionBinding<'ctx> {
    assert_eq!(args.len(), arg_types.len());

    let binding = DefinitionBinding {
        sret: sret_ptr(builder, signature),
    };

    let mut physical_index = usize::from(sret_ptr_is_present(signature));
    for (i, arg) in args.iter().enumerate() {
        let arg_ty = arg_types[i];
        let llvm_ty = conv.logical_llvm_type(arg_ty);
        let mode = &signature.arguments[i];
        let param = builder.arg(physical_index);

        let (value, borrowed, param) = if arg.is_inout() {
            physical_index += 1;
            let incoming = builder.local_get(param.unwrap()).as_ptr();
            let c_value = builder.load(incoming, boundary_type(conv, arg_ty));
            let local = from_boundary(conv, builder, arg_ty, c_value);
            (local, true, param)
        } else {
            let value = match mode {
                ArgumentMode::Ignore { original } => builder.const_zero(*original),
                ArgumentMode::Direct { original, passed_as } => {
                    let incoming = builder.local_get(param.unwrap());
                    coerce(conv, builder, incoming, *passed_as, *original)
                }
                ArgumentMode::Expand { original, parts } => {
                    let slot = builder.alloca(*original);
                    for part in parts {
                        let incoming = builder.local_get(builder.arg(physical_index).unwrap());
                        let dst = builder.gep(
                            slot,
                            *conv.llvm_context().integer(8),
                            builder.const_int(
                                conv.llvm_context().integer(64),
                                part.byte_offset as i64,
                                false,
                            ),
                        );
                        builder.store(dst, incoming);
                        physical_index += 1;
                    }
                    builder.load(slot, *original)
                }
                ArgumentMode::Indirect { original, .. } => {
                    let incoming = builder.local_get(param.unwrap()).as_ptr();
                    builder.load(incoming, *original)
                }
            };
            if !matches!(mode, ArgumentMode::Ignore { .. })
                && !matches!(mode, ArgumentMode::Expand { .. })
            {
                physical_index += 1;
            }
            let value = from_boundary(conv, builder, arg_ty, value);
            (value, false, None)
        };

        let local = builder.local(llvm_ty);
        let owned = if borrowed {
            value
        } else {
            conv.emit_copy(builder, value, arg_ty)
        };
        builder.local_set(local, owned);
        env.alloc_var(arg.name(), local, arg_ty, borrowed);
        if borrowed {
            env.add_inout(param.unwrap(), local, arg_ty);
        }
    }

    binding
}


pub(crate) fn emit_definition_return<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
    ret: SemanticType,
    binding: &DefinitionBinding<'ctx>,
    value: Value<'ctx>,
) {
    let value = to_boundary(conv, builder, ret, value);
    match &signature.return_mode {
        ReturnMode::Ignore { .. } => builder.ret_void(),
        ReturnMode::Direct { original, passed_as } => {
            let value = coerce(conv, builder, value, *original, *passed_as);
            builder.ret(value);
        }
        ReturnMode::Indirect { .. } => {
            builder.store(binding.sret.unwrap(), value);
            builder.ret_void();
        }
    }
}
fn emit_call_with_inout<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
    ret: SemanticType,
    callee: Value<'ctx>,
    values: &[(Value<'ctx>, SemanticType, bool)],
) -> Value<'ctx> {
    let boundary_ret = boundary_type(conv, ret);
    let sret = if sret_ptr_is_present(signature) {
        Some(builder.alloca(boundary_ret))
    } else {
        None
    };
    let mut physical = std::vec::Vec::new();
    if let Some(sret) = sret {
        physical.push(*sret);
    }

    let mut writebacks = Vec::new();
    for (index, (value, ty, inout)) in values.iter().enumerate() {
        let value = if *inout {
            let logical = conv.logical_llvm_type(*ty);
            let boundary = boundary_type(conv, *ty);
            if logical == boundary {
                *value
            } else {
                let incoming = builder.load(value.as_ptr(), logical);
                let incoming = convert_value(builder, incoming, boundary);
                let slot = builder.alloca_store(incoming);
                writebacks.push((value.as_ptr(), slot, *ty));
                *slot
            }
        } else {
            to_boundary(conv, builder, *ty, *value)
        };
        match &signature.arguments[index] {
            ArgumentMode::Ignore { .. } => {}
            ArgumentMode::Direct { original, passed_as } => {
                if *inout {
                    physical.push(value);
                } else {
                    physical.push(coerce(conv, builder, value, *original, *passed_as));
                }
            }
            ArgumentMode::Expand { parts, .. } => {
                let src = builder.alloca_store(value);
                for part in parts {
                    let ptr = builder.gep(
                        src,
                        *conv.llvm_context().integer(8),
                        builder.const_int(
                            conv.llvm_context().integer(64),
                            part.byte_offset as i64,
                            false,
                        ),
                    );
                    physical.push(builder.load(ptr, part.ty));
                }
            }
            ArgumentMode::Indirect { .. } => {
                if *inout {
                    physical.push(value);
                } else {
                    physical.push(*builder.alloca_store(value));
                }
            }
        }
    }

    let func_ptr = unsafe { FunctionPtr::new(callee) };
    let result = builder.call_c_with_attributes(
        func_ptr,
        signature.function_type,
        &physical,
        &signature.attributes,
    );

    for (destination, slot, ty) in writebacks {
        let boundary = boundary_type(conv, ty);
        let value = builder.load(slot, boundary);
        let value = from_boundary(conv, builder, ty, value);
        builder.store(destination, value);
    }

    let result = match &signature.return_mode {
        ReturnMode::Ignore { .. } => builder.const_zero(boundary_ret),
        ReturnMode::Direct { original, passed_as } => {
            coerce(conv, builder, result, *passed_as, *original)
        }
        ReturnMode::Indirect { original, .. } => builder.load(sret.unwrap(), *original),
    };
    from_boundary(conv, builder, ret, result)
}

pub(crate) fn emit_call<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &mut Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
    ret: SemanticType,
    callee: Value<'ctx>,
    values: &[(Value<'ctx>, SemanticType, Option<ExprId>)],
) -> Value<'ctx> {
    let values = values
        .iter()
        .map(|(value, ty, inout)| (*value, *ty, inout.is_some()))
        .collect::<std::vec::Vec<_>>();
    emit_call_with_inout(conv, builder, signature, ret, callee, &values)
}

pub(crate) fn emit_startup_call<'ctx>(
    conv: &mut Conversion<'_, '_, '_, '_, 'ctx>,
    builder: &Builder<'ctx>,
    signature: &LoweredSignature<'ctx>,
    ret: SemanticType,
    callee: FunctionPtr<'ctx>,
    values: &[StartupArg<'ctx>],
) {
    let values = values
        .iter()
        .map(|value| (value.value, value.ty, value.inout))
        .collect::<std::vec::Vec<_>>();
    emit_call_with_inout(conv, builder, signature, ret, *callee, &values);

}
// -----------------------------------------------------------------------------
// Classifier Unit Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use llvm_api::{ctx::Context, tys::TypeKind};
    use sti::arena::Arena;
    use super::*;

    #[test]
    fn test_darwin_scalars_and_hfa() {
        let arena = Arena::new();
        let ctx = Context::new(&arena, "arm64-apple-darwin");
        let module = ctx.module("test");
        let ctx = ctx.as_ctx_ref();

        // 1. bool scalar
        let lowered = lower_signature(
            CompilationTarget::Arm64AppleDarwin,
            ctx,
            module,
            *ctx.bool(),
            &[*ctx.bool()],
        );
        assert!(matches!(lowered.return_mode, ReturnMode::Direct { .. }));
        assert_eq!(lowered.attributes.len(), 2);
        assert_eq!(lowered.attributes[0].location, AttributeLocation::Return);
        assert_eq!(lowered.attributes[1].location, AttributeLocation::Parameter(0));

        let hfa2 = *ctx.literal_struct(&[*ctx.f64(), *ctx.f64()], false);
        let lowered = lower_signature(
            CompilationTarget::Arm64AppleDarwin,
            ctx,
            module,
            hfa2,
            &[hfa2],
        );
        // HFA returns directly as original
        assert_eq!(lowered.return_mode, ReturnMode::Direct { original: hfa2, passed_as: hfa2 });
        // HFA argument coerced to [2 x double]
        assert!(matches!(lowered.arguments[0], ArgumentMode::Direct { passed_as, .. } if passed_as.kind() == TypeKind::Array));
        assert_eq!(lowered.attributes.len(), 0); // No alignstack on Darwin
    }

    #[test]
    fn test_linux_aapcs_hfa_alignstack() {
        let arena = Arena::new();
        let ctx = Context::new(&arena, "aarch64-unknown-linux-gnu");
        let module = ctx.module("test");
        let ctx = ctx.as_ctx_ref();
        let hfa3 = *ctx.literal_struct(&[*ctx.f64(), *ctx.f64(), *ctx.f64()], false);
        let lowered = lower_signature(
            CompilationTarget::Aarch64UnknownLinuxGnu,
            ctx,
            module,
            hfa3,
            &[hfa3],
        );
        assert_eq!(lowered.return_mode, ReturnMode::Direct { original: hfa3, passed_as: hfa3 });
        // Linux AAPCS attaches alignstack(8) to HFA argument
        assert_eq!(lowered.attributes.len(), 1);
        assert_eq!(lowered.attributes[0].location, AttributeLocation::Parameter(0));
        assert_eq!(
            lowered.attributes[0].kind,
            AttributeKind::Enum { name: "alignstack", value: 8 }
        );
    }

    #[test]
    fn test_sysv_aggregates_and_exhaustion() {
        let arena = Arena::new();
        let ctx = Context::new(&arena, "x86_64-unknown-linux-gnu");
        let module = ctx.module("test");
        let ctx = ctx.as_ctx_ref();

        let s1 = *ctx.literal_struct(&[*ctx.integer(64)], false);
        let s2 = *ctx.literal_struct(&[*ctx.integer(64), *ctx.integer(64)], false);
        let s3 = *ctx.literal_struct(&[*ctx.integer(64), *ctx.integer(64), *ctx.integer(64)], false);
        // s1 return -> i64, arg -> Expand([i64])
        let lowered1 = lower_signature(CompilationTarget::X86_64UnknownLinuxGnu, ctx, module, s1, &[s1]);
        assert_eq!(lowered1.return_mode, ReturnMode::Direct { original: s1, passed_as: *ctx.integer(64) });
        assert!(matches!(lowered1.arguments[0], ArgumentMode::Expand { .. }));

        // s2 return -> { i64, i64 }, arg -> Expand([i64, i64])
        let lowered2 = lower_signature(CompilationTarget::X86_64UnknownLinuxGnu, ctx, module, s2, &[s2]);
        assert!(matches!(lowered2.return_mode, ReturnMode::Direct { passed_as, .. } if passed_as.kind() == TypeKind::Struct));
        if let ArgumentMode::Expand { parts, .. } = &lowered2.arguments[0] {
            assert_eq!(parts.len(), 2);
            assert_eq!(parts[0].byte_offset, 0);
            assert_eq!(parts[1].byte_offset, 8);
        } else {
            panic!("expected expand");
        }

        // s3 (> 16 bytes) -> Indirect sret / byval
        let lowered3 = lower_signature(CompilationTarget::X86_64UnknownLinuxGnu, ctx, module, s3, &[s3]);
        assert!(matches!(lowered3.return_mode, ReturnMode::Indirect { .. }));
        assert!(matches!(lowered3.arguments[0], ArgumentMode::Indirect { byval: true, .. }));

        // Register exhaustion: 6 int args followed by s2 (needs 2 int regs) -> byval
        let args = [*ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), s2];
        let lowered_ex = lower_signature(CompilationTarget::X86_64UnknownLinuxGnu, ctx, module, *ctx.void(), &args);
        assert!(matches!(lowered_ex.arguments[6], ArgumentMode::Indirect { byval: true, .. }));

        // Register exhaustion exception rule: 6 int args followed by s1 (size <= 8, align <= 8) -> direct stack carrier
        let args_ex_s1 = [*ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), *ctx.integer(64), s1];
        let lowered_ex_s1 = lower_signature(CompilationTarget::X86_64UnknownLinuxGnu, ctx, module, *ctx.void(), &args_ex_s1);
        assert!(matches!(lowered_ex_s1.arguments[6], ArgumentMode::Direct { passed_as, .. } if passed_as.kind() == TypeKind::Integer));
    }

    #[test]
    fn test_sysv_stack_aggregate_alignment() {
        let arena = Arena::new();
        let ctx = Context::new(&arena, "x86_64-unknown-linux-gnu");
        let module = ctx.module("test");
        let ctx = ctx.as_ctx_ref();
        let byte_aligned_17 = *ctx.array(*ctx.integer(8), 17);

        let lowered = lower_signature(
            CompilationTarget::X86_64UnknownLinuxGnu,
            ctx,
            module,
            *ctx.void(),
            &[byte_aligned_17],
        );

        assert_eq!(
            lowered.attributes,
            vec![
                AbiAttribute::new(
                    AttributeLocation::Parameter(0),
                    AttributeKind::Type { name: "byval", ty: byte_aligned_17 },
                ),
                AbiAttribute::new(
                    AttributeLocation::Parameter(0),
                    AttributeKind::Enum { name: "align", value: 8 },
                ),
            ],
        );
        assert_eq!(
            lowered.arguments,
            vec![ArgumentMode::Indirect {
                original: byte_aligned_17,
                byval: true,
                alignment: 8,
            }],
        );
    }

    #[test]
    fn test_wasm32_unwrapping() {
        let arena = Arena::new();
        let ctx = Context::new(&arena, "wasm32-unknown-unknown");
        let module = ctx.module("test");
        let ctx = ctx.as_ctx_ref();

        let s1 = *ctx.literal_struct(&[*ctx.integer(64)], false);
        let lowered1 = lower_signature(CompilationTarget::Wasm32UnknownUnknown, ctx, module, s1, &[s1]);
        assert_eq!(lowered1.return_mode, ReturnMode::Direct { original: s1, passed_as: *ctx.integer(64) });
        assert_eq!(lowered1.arguments[0], ArgumentMode::Direct { original: s1, passed_as: *ctx.integer(64) });

        let s_bool = *ctx.literal_struct(&[*ctx.bool()], false);
        let lowered_b = lower_signature(CompilationTarget::Wasm32UnknownUnknown, ctx, module, s_bool, &[s_bool]);
        assert_eq!(lowered_b.return_mode, ReturnMode::Direct { original: s_bool, passed_as: *ctx.bool() });
        assert_eq!(lowered_b.arguments[0], ArgumentMode::Direct { original: s_bool, passed_as: *ctx.bool() });
        assert_eq!(lowered_b.attributes.len(), 2);
        assert_eq!(lowered_b.attributes[0].location, AttributeLocation::Return);
        assert_eq!(lowered_b.attributes[1].location, AttributeLocation::Parameter(0));

        let s2 = *ctx.literal_struct(&[*ctx.integer(32), *ctx.integer(32)], false);
        let lowered2 = lower_signature(CompilationTarget::Wasm32UnknownUnknown, ctx, module, s2, &[s2]);
        assert!(matches!(lowered2.return_mode, ReturnMode::Indirect { .. }));
        assert!(matches!(lowered2.arguments[0], ArgumentMode::Indirect { byval: true, .. }));
    }

    #[test]
    fn test_unsigned_byte_extension_attributes() {
        let targets = [
            (CompilationTarget::Arm64AppleDarwin, 2),
            (CompilationTarget::X86_64UnknownLinuxGnu, 2),
            (CompilationTarget::Aarch64UnknownLinuxGnu, 0),
            (CompilationTarget::Wasm32UnknownUnknown, 2),
        ];

        for (target, expected_attributes) in targets {
            let arena = Arena::new();
            let triple = target.llvm_target_triple();
            let ctx = Context::new(&arena, &triple);
            let module = ctx.module("test");
            let ctx = ctx.as_ctx_ref();
            let byte = *ctx.integer(8);
            let lowered = lower_signature(target, ctx, module, byte, &[byte]);

            assert_eq!(lowered.attributes.len(), expected_attributes);
            assert!(lowered.attributes.iter().all(|attribute| {
                attribute.kind == AttributeKind::Enum { name: "zeroext", value: 0 }
            }));
        }
    }
}
