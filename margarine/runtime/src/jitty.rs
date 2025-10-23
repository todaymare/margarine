use std::{collections::HashMap, mem::offset_of};

use cranelift::{codegen::{self, ir::FuncRef}, prelude::{settings, types::{I64, I64X2}, AbiParam, Block, Configurable, FunctionBuilder, FunctionBuilderContext, InstBuilder, MemFlags, Signature, Type, Value}};
use cranelift_jit::{JITBuilder, JITMemoryProvider, JITModule};
use cranelift_module::{DataDescription, FuncId, Linkage, Module};

use crate::{opcode::runtime::{Decoded, OpCode}, runtime::ret_instr, Function, FunctionKind, Reader, Reg, Stack, VM};

pub struct JIT {
    builder_ctx: FunctionBuilderContext,
    ctx: codegen::Context,
    data_description: DataDescription,
    module: JITModule,

    ret_instr: FuncId,
}


impl JIT {
    pub fn default() -> Self {
        let mut flag_builder = settings::builder();
        flag_builder.set("use_colocated_libcalls", "false").unwrap();
        flag_builder.set("is_pic", "false").unwrap();
        let isa_builder = cranelift_native::builder().unwrap_or_else(|msg| {
            panic!("host machine is not supported: {}", msg);
        });

        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .unwrap();

        let ptr = isa.pointer_type();
        let conv = isa.default_call_conv();

        let mut builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        builder.symbol("ret_instr", ret_instr as _);

        let mut module = JITModule::new(builder);


        let sig = Signature {
            params: vec![AbiParam::new(ptr)],
            returns: vec![],
            call_conv: conv,
        };

        let ret_instr = module.declare_function("ret_instr", cranelift_module::Linkage::Import, &sig).unwrap();

        Self {
            builder_ctx: FunctionBuilderContext::new(),
            ctx: module.make_context(),
            data_description: DataDescription::new(),
            module,

            ret_instr,
        }
    }
}


#[derive(Debug)]
pub enum JITResult<'src> {
    Success,
    HostFunction,
    UnsupportedInstr(usize, Decoded<'src>),
    UnsupportedArgType(u32),
}


pub fn attempt_jit<'src>(vm: &mut VM<'src>, idx: usize) -> JITResult<'src> {
    let result = attempt_jit_ex(vm, idx);

    if !matches!(result, JITResult::Success) {
        vm.jit.builder_ctx = FunctionBuilderContext::new();
        vm.jit.module.clear_context(&mut vm.jit.ctx);
    }

    result
}


fn attempt_jit_ex<'src>(vm: &mut VM<'src>, idx: usize) -> JITResult<'src> {
    let func = &vm.funcs[idx];
    println!("attemptiing to jit {}", func.name);
    let FunctionKind::Code { byte_offset, byte_size } = func.kind
    else { return JITResult::HostFunction };

    let src = &vm.callstack.src[byte_offset..byte_offset+byte_size];

    let ptr_ty = vm.jit.module.target_config().pointer_type();

    // fn(*mut VM, *mut Reg)
    vm.jit.ctx.func.signature = Signature {
        params: vec![
            AbiParam::new(ptr_ty),
            AbiParam::new(I64),
        ],
        returns: vec![],
        call_conv: vm.jit.module.target_config().default_call_conv,
    };


    let ret_instr = vm.jit.module.declare_func_in_func(vm.jit.ret_instr, &mut vm.jit.ctx.func);


    let mut builder = FunctionBuilder::new(
        &mut vm.jit.ctx.func,
        &mut vm.jit.builder_ctx
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

    builder.switch_to_block(bbs[&0]);
    builder.append_block_params_for_function_params(bbs[&0]);


    let vm_ptr = builder.block_params(bbs[&0])[0];

    let curr_offset = offset_of!(VM, stack.curr);
    let values_offset = offset_of!(VM, stack.values);

    let values_ptr = builder.ins().iadd_imm(vm_ptr, values_offset as i64);
    let curr_ptr   = builder.ins().iadd_imm(vm_ptr, curr_offset as i64);


    let mut reader = Reader::new(&src);

    while let Some(opcode) = OpCode::decode(&mut reader) {
        let offset = reader.offset_from_start();
        let result = decode_instr(
            offset, opcode.1, &bbs, &mut builder, ptr_ty, func,
            ret_instr, vm_ptr, values_ptr, curr_ptr,
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

    //vm.funcs[idx].kind = FunctionKind::Host(func);



    JITResult::Success
}


fn decode_instr(
    offset: usize,
    instr: Decoded,
    bbs: &HashMap<usize, Block>,
    builder: &mut FunctionBuilder,
    ptr_ty: Type,
    func: &Function,

    ret_instr: FuncRef,

    vm_ptr      : Value,
    stack_bottom: Value,
    stack_curr  : Value,
) -> bool {
    macro_rules! push_const {
        ($reg: expr) => {
            let stack_offset = builder.ins().load(I64, MemFlags::new(), stack_curr, 0);
            let offset = builder.ins().imul_imm(stack_offset, size_of::<Reg>() as i64);
            let offset = builder.ins().iadd(stack_bottom, offset);
            
            let tag = builder.ins().iconst(I64, $reg.kind as i64);
            let data = builder.ins().iconst(I64, unsafe { $reg.data.as_int });

            builder.ins().store(MemFlags::new(), tag , offset, offset_of!(Reg, kind) as i32);
            builder.ins().store(MemFlags::new(), data, offset, offset_of!(Reg, data) as i32);

            let curr = builder.ins().iadd_imm(stack_offset, 1);
            builder.ins().store(MemFlags::new(), curr, stack_curr, 0);
        };
    }

    match instr {
        Decoded::PushLocalSpace { amount } => {
            let curr = builder.ins().load(I64, MemFlags::new(), stack_curr, 0);
            let curr = builder.ins().iadd_imm(curr, amount as i64);
            builder.ins().store(MemFlags::new(), curr, stack_curr, 0);
        }


        Decoded::Ret { local_count } => {
            let imm = builder.ins().iconst(I64, local_count as i64);
            let value = builder.create_sized_stack_slot(codegen::ir::StackSlotData { kind: codegen::ir::StackSlotKind::ExplicitSlot, size: 8, align_shift: 3 });

            let zero = builder.ins().iconst(I64, 0);
            builder.ins().stack_store(zero, value, 0);

            let should_ret = builder.ins().stack_addr(I64, value, 0);

            //builder.ins().call(ret_instr, &[vm_ptr, imm, should_ret]);
            builder.ins().return_(&[]);
        }


        Decoded::Unit {  } => {
            push_const!(Reg::new_unit());
        },


        Decoded::ConstInt { val } => {
            push_const!(Reg::new_int(val));
        },


        Decoded::ConstFloat { val } => {
            push_const!(Reg::new_float(val));
        },


        Decoded::ConstBool { val } => {
            push_const!(Reg::new_bool(val != 0));
        },

        _ => return false,
    };

    return true;
}


