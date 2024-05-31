use std::{ffi::CStr, marker::PhantomData, ptr::{null_mut, NonNull}};

use llvm_sys::{analysis::{LLVMVerifierFailureAction, LLVMVerifyModule}, core::{LLVMAddFunction, LLVMAddGlobal, LLVMPrintModuleToString}, target::{LLVM_InitializeAllAsmParsers, LLVM_InitializeAllAsmPrinters, LLVM_InitializeAllTargetInfos, LLVM_InitializeAllTargetMCs, LLVM_InitializeAllTargets}, target_machine::{LLVMCreateTargetMachine, LLVMGetDefaultTargetTriple, LLVMGetTargetFromTriple}, transforms::pass_builder::{LLVMCreatePassBuilderOptions, LLVMRunPasses}, LLVMModule};
use sti::arena::Arena;

use crate::{cstr, info::Message, tys::{func::FunctionType, Type}, values::{func::FunctionPtr, global::GlobalPtr, Value}};

#[derive(Clone, Copy)]
pub struct Module<'ctx> {
    pub(crate) ptr: NonNull<LLVMModule>,
    phantom: PhantomData<&'ctx ()>
}

impl<'ctx> Module<'ctx> {
    pub fn new(ptr: NonNull<LLVMModule>) -> Self {
        Self { ptr, phantom: PhantomData }
    }


    pub fn function(&self, name: &str, ty: FunctionType<'ctx>) -> FunctionPtr<'ctx> {
        assert!(!name.contains('\0'), "the function name can't contain null bytes");

        let pool = Arena::tls_get_temp();
        let name = sti::format_in!(&*pool, "{name}\0");

        let func = unsafe { LLVMAddFunction(self.ptr.as_ptr(),
                                            name.as_ptr().cast(),
                                            ty.llvm_ty().as_ptr()) };

        let func = NonNull::new(func).expect("failed to create a function");

        unsafe { FunctionPtr::new(Value::new(func)) }
    }


    pub fn add_global(&self, ty: Type<'ctx>, name: &str) -> GlobalPtr<'ctx> {
        assert!(!name.contains('\0'), "the function name can't contain null bytes");

        let pool = Arena::tls_get_temp();
        let name = sti::format_in!(&*pool, "{name}\0");

        let ptr = unsafe { LLVMAddGlobal(self.ptr.as_ptr(), ty.llvm_ty().as_ptr(), name.as_ptr() as *const i8) };

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


    pub fn optimize(&self) {
        unsafe { LLVM_InitializeAllTargets() };
        unsafe { LLVM_InitializeAllTargetInfos() };
        unsafe { LLVM_InitializeAllTargetMCs() };
        unsafe { LLVM_InitializeAllAsmParsers() };
        unsafe { LLVM_InitializeAllAsmPrinters() };

        let pbo = unsafe { LLVMCreatePassBuilderOptions() };
        let tt = unsafe { LLVMGetDefaultTargetTriple() };
        let mut target = null_mut();
        let mut msg = null_mut();
        if unsafe { LLVMGetTargetFromTriple(tt, &mut target, &mut msg) } != 0 {
            let cstr = unsafe { CStr::from_ptr(msg) };
            panic!("{}", cstr.to_str().unwrap());
        }



        let tm = unsafe { LLVMCreateTargetMachine(target, tt, "".as_ptr() as _,
                                                  "".as_ptr() as _,
                                                  llvm_sys::target_machine::LLVMCodeGenOptLevel::LLVMCodeGenLevelAggressive,
                                                  llvm_sys::target_machine::LLVMRelocMode::LLVMRelocDefault,
                                                  llvm_sys::target_machine::LLVMCodeModel::LLVMCodeModelDefault) };

        unsafe { LLVMRunPasses(self.ptr.as_ptr(), cstr!("default<O3>"), tm, pbo); }
    }
}
