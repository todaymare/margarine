use pc::{ProgramCounter, Code};
use register_stack::RegisterStack;
use data_stack::{DataStack, Data, StackFrame, StackFramePointer};

pub mod pc;
pub mod register_stack;
pub mod data_stack;


pub struct VM<'consts> {
    pub const_strs: &'consts [&'consts str],
    pub pc: ProgramCounter,
    pub register_stack: RegisterStack,
    pub data_stack: DataStack,


    ///
    /// A stack of `StackFramePointer`s used for debug assertions
    ///
    #[cfg(debug_assertions)]
    stackframe_stack: Vec<StackFramePointer>,
}

impl<'consts> VM<'consts> {
    pub fn new(
        const_strs: &'consts [&'consts str], 
        register_stack: RegisterStack
    ) -> Self { 
        Self { 
            const_strs, 
            pc: ProgramCounter::new(unsafe { 
                Code::from_raw(core::ptr::null(), #[cfg(debug_assertions)] core::ptr::null()) }), 
            register_stack,
            data_stack: DataStack::new(),

            #[cfg(debug_assertions)]
            stackframe_stack: Vec::new(), 
        } 
    }
}



#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);


impl TypeId {
    pub const UNIT_TAG : TypeId = TypeId(0);
    pub const INT_TAG  : TypeId = TypeId(1);
    pub const FLOAT_TAG: TypeId = TypeId(2);
    pub const UINT_TAG : TypeId = TypeId(3);
    pub const BOOL_TAG : TypeId = TypeId(4);
    pub const STR_TAG  : TypeId = TypeId(5);
}
