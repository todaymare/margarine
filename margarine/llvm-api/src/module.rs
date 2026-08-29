use std::{ffi::{CStr, CString}, ptr::{null_mut, NonNull}};

use llvm_sys::{analysis::{LLVMVerifierFailureAction, LLVMVerifyModule}, core::{LLVMAddFunction, LLVMAddGlobal, LLVMBuildCall2, LLVMCreateBuilderInContext, LLVMDisposeBuilder, LLVMGetBasicBlockTerminator, LLVMGetFirstBasicBlock, LLVMGetFirstFunction, LLVMGetNextBasicBlock, LLVMGetNextFunction, LLVMPositionBuilderBefore, LLVMPrintModuleToString}, error::{LLVMDisposeErrorMessage, LLVMGetErrorMessage}, target::{LLVMGetModuleDataLayout, LLVMPointerSize}, target_machine::LLVMOpaqueTargetMachine, transforms::pass_builder::{LLVMCreatePassBuilderOptions, LLVMDisposePassBuilderOptions, LLVMPassBuilderOptionsSetLoopUnrolling, LLVMPassBuilderOptionsSetVerifyEach, LLVMRunPasses}, LLVMModule};
use sti::arena::Arena;

use crate::{cstr, info::Message, tys::{func::FunctionType, Type}, values::{func::FunctionPtr, global::GlobalPtr, Value}};

#[derive(Clone, Copy)]
pub struct Module<'ctx> {
    pub(crate) ptr: NonNull<LLVMModule>,
    target_machine: NonNull<LLVMOpaqueTargetMachine>,
    arena: &'ctx Arena,
}

impl<'ctx> Module<'ctx> {
    pub fn new(
        ptr: NonNull<LLVMModule>,
        target_machine: NonNull<LLVMOpaqueTargetMachine>,
        arena: &'ctx Arena,
    ) -> Self {
        Self { ptr, target_machine, arena }
    }


    pub fn function(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionPtr<'ctx> {
        assert!(!name.contains('\0'), "the function name can't contain null bytes");

        let pool = self.arena;
        let name = sti::format_in!(&*pool, "{name}\0");

        let func = unsafe { LLVMAddFunction(self.ptr.as_ptr(),
                                            name.as_ptr().cast(),
                                            ty.llvm_ty().as_ptr()) };

        let func = NonNull::new(func).expect("failed to create a function");

        unsafe { FunctionPtr::new(Value::new(func)) }
    }


    pub fn add_global(&self, ty: Type<'ctx>, name: &str) -> GlobalPtr<'ctx> {
        assert!(!name.contains('\0'), "the function name can't contain null bytes");

        let pool = self.arena;
        let name = sti::format_in!(&*pool, "{name}\0");

        let ptr = unsafe { LLVMAddGlobal(self.ptr.as_ptr(), ty.llvm_ty().as_ptr(), name.as_ptr() as *const std::ffi::c_char) };

        unsafe { GlobalPtr::new(Value::new(NonNull::new(ptr).unwrap())) }
    }


    pub fn dump_to_str(&self) -> Message {
        unsafe { Message::new(NonNull::new(LLVMPrintModuleToString(self.ptr.as_ptr())).unwrap()) }
    }


    pub fn validate(&self) -> Result<(), Message> {
        let mut char = std::ptr::null_mut();
        unsafe { LLVMVerifyModule(self.ptr.as_ptr(), LLVMVerifierFailureAction::LLVMReturnStatusAction, &mut char); }

        if char.is_null() { return Ok(()) }
        
        let msg = unsafe { Message::new(NonNull::new(char).unwrap()) };

        if msg.as_str().is_empty() { return Ok(()) }
        Err(msg)
    }


    pub fn ptr_size_in_bytes(&self) -> usize {
        let dt = unsafe { LLVMGetModuleDataLayout(self.ptr.as_ptr()) };
        unsafe { LLVMPointerSize(dt) as usize } 
    }


    /// Inserts a call to `fuel_fn` immediately before the terminator of every
    /// basic block in every defined function in the module.
    pub fn instrument_basic_block_exits(
        &self,
        ctx: crate::ctx::ContextRef<'ctx>,
        fuel_fn: FunctionPtr<'ctx>,
        fuel_fn_ty: FunctionType<'ctx>,
    ) {
        let builder = unsafe { LLVMCreateBuilderInContext(ctx.ptr.as_ptr()) };
        let builder = NonNull::new(builder).expect("failed to create fuel instrumentation builder");

        unsafe {
            let mut function = LLVMGetFirstFunction(self.ptr.as_ptr());
            while !function.is_null() {
                // Declarations have no basic blocks and are skipped naturally.
                let mut block = LLVMGetFirstBasicBlock(function);
                while !block.is_null() {
                    // Keep the next block before inserting into the current one.
                    let next = LLVMGetNextBasicBlock(block);
                    let terminator = LLVMGetBasicBlockTerminator(block);

                    if !terminator.is_null() {
                        LLVMPositionBuilderBefore(builder.as_ptr(), terminator);
                        let call = LLVMBuildCall2(
                            builder.as_ptr(),
                            fuel_fn_ty.llvm_ty().as_ptr(),
                            fuel_fn.llvm_val().as_ptr(),
                            null_mut(),
                            0,
                            cstr!(""),
                        );
                        assert!(!call.is_null(), "failed to build __consume_fuel call");
                    }

                    block = next;
                }

                function = LLVMGetNextFunction(function);
            }

            LLVMDisposeBuilder(builder.as_ptr());
        }
    }


    pub fn optimize(&self) -> Result<(), String> {
        let level = std::env::var("MARGARINE_OPT_LEVEL").unwrap_or_else(|_| "O3".to_string());
        if !matches!(level.as_str(), "O0" | "O1" | "O2" | "O3") {
            return Err(format!(
                "invalid MARGARINE_OPT_LEVEL '{level}'; expected O0, O1, O2, or O3"
            ));
        }
        let pipeline = format!("default<{level}>");
        let pipeline = CString::new(pipeline)
            .expect("optimization pipeline cannot contain a null byte");

        let options = unsafe { LLVMCreatePassBuilderOptions() };
        let Some(options) = NonNull::new(options) else {
            return Err("LLVM failed to create pass builder options".to_string());
        };

        unsafe {
            LLVMPassBuilderOptionsSetVerifyEach(options.as_ptr(), 1);
            LLVMPassBuilderOptionsSetLoopUnrolling(options.as_ptr(), 1);
        }

        let error = unsafe {
            LLVMRunPasses(
                self.ptr.as_ptr(),
                pipeline.as_ptr(),
                self.target_machine.as_ptr(),
                options.as_ptr(),
            )
        };
        unsafe { LLVMDisposePassBuilderOptions(options.as_ptr()) };

        if error.is_null() {
            return Ok(());
        }

        let error_message = unsafe { LLVMGetErrorMessage(error) };
        if error_message.is_null() {
            return Err("LLVM optimization failed".to_string());
        }

        let message = unsafe { CStr::from_ptr(error_message) }.to_string_lossy().into_owned();
        unsafe { LLVMDisposeErrorMessage(error_message) };
        Err(message)
    }
}
