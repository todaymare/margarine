use pc::{ProgramCounter, Code};
use register_stack::RegisterStack;

pub mod pc;
pub mod register_stack;


pub struct VM<'consts> {
    pub constants: &'consts [Data],
    pub pc: ProgramCounter,
    pub register_stack: RegisterStack,
}

impl<'consts> VM<'consts> {
    pub fn new(
        constants: &'consts [Data], 
        register_stack: RegisterStack
    ) -> Self { 
        Self { 
            constants, 
            pc: ProgramCounter::new(unsafe { Code::from_raw(core::ptr::null(), core::ptr::null()) }), 
            register_stack 
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



pub union Data {
    pub int  : i64,
    pub float: f64,
    pub uint : u64,
    pub unit : (),
    pub bool : bool,
    
    pub obj  : *mut Data,
    pub frame: *const StackFrame,
}


#[derive(Clone, Debug)]
pub struct StackFrame {
    pub pc: ProgramCounter,
}