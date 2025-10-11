use std::{collections::HashMap, mem::offset_of};

use cranelift::prelude::*;
use cranelift_jit::{JITBuilder, JITModule};
use cranelift_module::{DataDescription, Linkage, Module};
use crate::{opcode::runtime::{Decoded, OpCode}, Function, FunctionKind, Reader, Reg, Stack, VM};


#[derive(Debug)]
pub enum JITResult<'src> {
    Success,
    HostFunction,
    UnsupportedInstr(usize, Decoded<'src>),
    UnsupportedArgType(u32),
}


pub fn attempt_jit<'src>(vm: &mut VM<'src>, func: usize) -> JITResult<'src> {
    let result = attempt_jit_internal(vm, func);
    vm.jit.module.clear_context(&mut vm.jit.ctx);

    if !matches!(result, JITResult::Success) {
        vm.jit.builder_context = FunctionBuilderContext::new();
    }

    result
}


fn attempt_jit_internal<'src>(vm: &mut VM<'src>, index: usize) -> JITResult<'src> {
    let func = &vm.funcs[index];
    let FunctionKind::Code { byte_offset, byte_size } = func.kind
    else { return JITResult::HostFunction };

    let src = &vm.callstack.src[byte_offset..byte_offset+byte_size];

    let ptr_ty = vm.jit.module.target_config().pointer_type();

    // fn(*mut VM, *mut Reg)
    vm.jit.ctx.func.signature = Signature {
        params: vec![
            AbiParam::new(ptr_ty),
            AbiParam::new(ptr_ty),
        ],
        returns: vec![],
        call_conv: vm.jit.module.target_config().default_call_conv,
    };

    let mut builder = FunctionBuilder::new(
        &mut vm.jit.ctx.func,
        &mut vm.jit.builder_context
    );

    // infer basic block offsets
    let bbs = {
        let mut reader = Reader::new(src);
        let mut block_start = 0;
        let mut bbs = HashMap::new();

        while let Some(opcode) = OpCode::decode(&mut reader) {
            match opcode.1 {
                  crate::opcode::runtime::Decoded::Ret { .. }
                | crate::opcode::runtime::Decoded::Jump { .. }
                | crate::opcode::runtime::Decoded::SwitchOn { .. }
                | crate::opcode::runtime::Decoded::Err { .. }
                | crate::opcode::runtime::Decoded::Switch { .. } => {
                    let offset = reader.offset_from_start();
                    let bb = builder.create_block();
                    bbs.insert(block_start, bb);
                    block_start = offset;
                },

                _ => (),
            }

        }

        bbs.insert(reader.offset_from_start(), builder.create_block());

        bbs
    };


    // transcode
    builder.switch_to_block(bbs[&0]);
    builder.append_block_params_for_function_params(bbs[&0]);

    // unwrap the arguments
    if func.argc != 0 {
        let param = builder.block_params(bbs[&0])[0];

        let stack_offset = offset_of!(VM, stack);
        let bottom_offset = offset_of!(Stack, bottom);
        let values_offset = offset_of!(Stack, values);

        let bottom = builder.ins().load(ptr_ty, MemFlags::new(), param, (stack_offset + bottom_offset) as i32);
        let values = builder.ins().load(ptr_ty, MemFlags::new(), param, (stack_offset + values_offset) as i32);

        let offset = builder.ins().imul_imm(bottom, size_of::<Reg>() as i64);
        let offset = builder.ins().iadd(values, offset);

        let mut args = Reader::new(func.args);
        for i in 0..func.argc {
            let elem_offset = builder.ins().iadd_imm(offset, (i as usize * size_of::<Reg>()) as i64);
            let ty = unsafe { args.next_u32() }; 
            let ty = match ty as u64 {
                Reg::TAG_INT => ptr_ty,
                Reg::TAG_FLOAT => types::F64,
                Reg::TAG_UNIT => types::I64,
                Reg::TAG_BOOL => types::I64,

                _ => return JITResult::UnsupportedArgType(ty),
            };

            let elem_offset = builder.ins().load(
                ty,
                MemFlags::new(),
                elem_offset,
                offset_of!(Reg, data) as i32
            );



            let var = builder.declare_var(ty);
            builder.def_var(var, elem_offset);
        }
    }

    let mut reader = Reader::new(&src);
    let mut stack = vec![];

    while let Some(opcode) = OpCode::decode(&mut reader) {
        let offset = reader.offset_from_start();
        let result = decode_instr(
            offset, opcode.1, &bbs, &mut builder,
            &mut stack, ptr_ty, func
        );

        if !result {
            return JITResult::UnsupportedInstr(offset, opcode.1);
        }
    }


    for block in bbs {
        builder.seal_block(block.1);
    }

    dbg!(&builder.func);
    builder.finalize();


    let id = vm.jit
        .module
        .declare_function(&func.name, Linkage::Export, &vm.jit.ctx.func.signature)
        .map_err(|e| e.to_string()).unwrap();


    vm.jit.module
        .define_function(id, &mut vm.jit.ctx)
        .map_err(|e| e.to_string()).unwrap();

    vm.jit.module.clear_context(&mut vm.jit.ctx);

    vm.jit.module.finalize_definitions().unwrap();
    let code = vm.jit.module.get_finalized_function(id);

    let func: unsafe extern "C" fn(&mut VM, &mut Reg) = unsafe { std::mem::transmute(code) };
    dbg!(func);

    vm.funcs[index].kind = FunctionKind::Host(func);

    JITResult::Success
}


fn decode_instr(
    offset: usize,
    instr: Decoded,
    bbs: &HashMap<usize, Block>,
    builder: &mut FunctionBuilder,
    stack: &mut Vec<Value>,
    ptr_ty: Type,
    func: &Function,
) -> bool {
    let value = match instr {
        Decoded::Ret { .. } => {
            let param = builder.block_params(bbs[&0])[1];

            let imm = builder.ins().iconst(ptr_ty, func.ret as i64);
            let value = stack.pop().unwrap();
            builder.ins().store(MemFlags::new(), imm, param, offset_of!(Reg, kind) as i32);
            builder.ins().store(MemFlags::new(), value, param, offset_of!(Reg, data) as i32);

            builder.ins().return_(&[]);
            builder.switch_to_block(bbs[&offset]);

            return true;
        },


        Decoded::Unit {  } => {
            builder.ins().iconst(ptr_ty, 0)
        },


        Decoded::PushLocalSpace { amount } => {
            for _ in 0..amount {
                builder.declare_var(ptr_ty);
            }

            return true;
        },


        Decoded::PopLocalSpace { amount } => return true,


        Decoded::Err { ty, file, index } => return false,


        Decoded::ConstInt { val } => {
            builder.ins().iconst(ptr_ty, val)
        },


        Decoded::ConstFloat { val } => {
            builder.ins().f64const(val)
        },


        Decoded::ConstBool { val } => {
            builder.ins().iconst(ptr_ty, val as i64)
        },


        Decoded::ConstStr { val } => return false,


        Decoded::Call { func, argc } => {
            return false;
        },


        Decoded::Pop {  } => {
            stack.pop().unwrap();
            return true;
        },


        Decoded::Copy {  } => {
            *stack.last().unwrap()
        },


        Decoded::CreateList { elem_count } => return false,
        Decoded::CreateStruct { field_count } => return false,
        Decoded::LoadField { field_index } => return false,
        Decoded::IndexList {  } => return false,
        Decoded::StoreList {  } => return false,
        Decoded::StoreField { field_index } => return false,
        Decoded::LoadEnumField { enum_index } => return false,
        Decoded::CreateFuncRef { capture_count } => return false,
        Decoded::CallFuncRef { argc } => return false,
        Decoded::Unwrap {  } => return false,
        Decoded::UnwrapFail {  } => return false,
        Decoded::CastIntToFloat {  } => {
            return false
        },


        Decoded::CastFloatToInt {  } => return false,
        Decoded::CastBoolToInt {  } => return false,


        Decoded::NegInt {  } => {
            let value = stack.pop().unwrap();
            builder.ins().imul_imm(value, -1)
        },


        Decoded::AddInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().iadd(lhs, rhs)
        },


        Decoded::SubInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().isub(lhs, rhs)
        },


        Decoded::MulInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().imul(lhs, rhs)
        },


        Decoded::DivInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().udiv(lhs, rhs)
        },


        Decoded::RemInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().urem(lhs, rhs)
        },


        Decoded::EqInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::Equal, lhs, rhs)
        },


        Decoded::NeInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::NotEqual, lhs, rhs)
        },


        Decoded::GtInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::SignedGreaterThan, lhs, rhs)
        },


        Decoded::GeInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, lhs, rhs)
        },


        Decoded::LtInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::SignedLessThan, lhs, rhs)
        },


        Decoded::LeInt {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().icmp(IntCC::SignedLessThanOrEqual, lhs, rhs)
        },


        Decoded::BitwiseOr {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().bor(lhs, rhs)
        },


        Decoded::BitwiseAnd {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().band(lhs, rhs)
        },


        Decoded::BitwiseXor {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().bxor(lhs, rhs)
        },


        Decoded::BitshiftLeft {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().ishl(lhs, rhs)
        },


        Decoded::BitshiftRight {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().sshr(lhs, rhs)
        },

        Decoded::NegFloat {  } => {
            let rhs = stack.pop().unwrap();
            let neg1 = builder.ins().f64const(-1.0);
            builder.ins().fmul(rhs, neg1)
        },


        Decoded::AddFloat {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().fadd(lhs, rhs)
        },


        Decoded::SubFloat {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().fsub(lhs, rhs)
        },


        Decoded::MulFloat {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().fmul(lhs, rhs)
        },


        Decoded::DivFloat {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();
            builder.ins().fdiv(lhs, rhs)
        },


        Decoded::RemFloat {  } => {
            let rhs = stack.pop().unwrap();
            let lhs = stack.pop().unwrap();

            let div = builder.ins().fdiv(lhs, rhs);
            let div_floor = builder.ins().floor(div);
            let y_mul_floor = builder.ins().fmul(rhs, div_floor);
            let rem = builder.ins().fsub(lhs, y_mul_floor);
            rem
        },


        Decoded::EqFloat {  } => return false,
        Decoded::NeFloat {  } => return false,
        Decoded::GtFloat {  } => return false,
        Decoded::GeFloat {  } => return false,
        Decoded::LtFloat {  } => return false,
        Decoded::LeFloat {  } => return false,
        Decoded::EqBool {  } => return false,
        Decoded::NeBool {  } => return false,


        Decoded::NotBool {  } => {
            builder.ins().bnot(stack.pop().unwrap())
        },


        Decoded::Load { index } => {
            builder.use_var(Variable::new(index as usize))
        },


        Decoded::Store { index } => {
            builder.def_var(Variable::new(index as usize), stack.pop().unwrap());
            return true;
        },


        Decoded::UnwrapStore {  } => return false,


        Decoded::Jump { offset: jmp_offset } => {
            let bb = bbs[&((offset as i32 + jmp_offset) as usize)];

            builder.ins().jump(bb, &[]);
            builder.switch_to_block(bbs[&offset]);
            return true;
        },


        Decoded::SwitchOn { true_offset, false_offset } => {
            let true_bb = bbs[&((offset as i32 + true_offset) as usize)];
            let false_bb = bbs[&((offset as i32 + false_offset) as usize)];

            let cond = stack.pop().unwrap();
            builder.ins().brif(cond, true_bb, &[], false_bb, &[]);
            builder.switch_to_block(bbs[&offset]);
            return true;
        },


        Decoded::Switch { offsets } => return false,
    };

    stack.push(value);
    return true;
}


/// The basic JIT class.
pub struct JIT {
    /// The function builder context, which is reused across multiple
    /// FunctionBuilder instances.
    builder_context: FunctionBuilderContext,

    /// The main Cranelift context, which holds the state for codegen. Cranelift
    /// separates this from `Module` to allow for parallel compilation, with a
    /// context per thread, though this isn't in the simple demo here.
    ctx: codegen::Context,

    /// The data description, which is to data objects what `ctx` is to functions.
    data_description: DataDescription,

    /// The module, with the jit backend, which manages the JIT'd
    /// functions.
    module: JITModule,
}




impl Default for JIT {
    fn default() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());

        let module = JITModule::new(builder);
        Self {
            builder_context: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,
        }
    }
}

