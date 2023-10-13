use sti::{prelude::Arena, arena::ArenaSavePoint, static_assert};
use core::ptr::NonNull;
use std::collections::HashMap;

use crate::{pc::ProgramCounter, VM, TypeId};

pub struct DataStack {
    arena: Arena,
}


impl DataStack {
    pub fn new() -> Self { 
        Self { 
            arena: Arena::new(),
        } 
    }
}


impl VM<'_> {
    
    ///
    /// Creates a `StackFrame` and allocates it onto the stack
    /// returning a pointer to it.
    ///
    #[inline(always)]
    pub fn alloc_stackframe(&mut self) -> StackFramePointer {
        let stackframe = StackFrame {
            pc: self.pc,
            save_point: self.data_stack.arena.save(),
        };

        let allocation = self.data_stack.arena.alloc_new(stackframe);
        let ptr : *const StackFrame = allocation as *const StackFrame;
        let ptr = StackFramePointer(ptr);

        #[cfg(debug_assertions)]
        self.stackframe_stack.push(ptr);

        ptr
    }


    ///
    /// Restores the current state occording to the given `StackFrame`.
    /// 
    /// # Safety
    /// - The pointer must be of the stack that's the most recently created.
    /// - The pointer must've been created by this `VM` instance
    ///
    #[inline(always)]
    pub unsafe fn restore_stackframe(&mut self, stackframe: StackFramePointer) {
        #[cfg(debug_assertions)]
        {
            let Some(last_stackframe) = self.stackframe_stack.pop()
            else { panic!("debug assertion: there are no valid stack frames to restore ") };

            assert_eq!(stackframe, last_stackframe);
        }
        
        let save_point = {
            let deref = unsafe { &*stackframe.0 };

            self.pc = deref.pc;
            deref.save_point.clone()
        };

        unsafe { self.data_stack.arena.restore(save_point) };
    }
}


///
/// A wrapper around `*const StackFrame` ensuring this
/// module is the only one that can create it.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StackFramePointer(*const StackFrame);


#[derive(Clone, Copy)]
pub struct Data {
    #[cfg(debug_assertions)]
    typ: TypeId,
    
    data: DataUnion,
}


static_assert!(core::mem::size_of::<DataUnion>() <= 8);

#[derive(Clone, Copy)]
pub union DataUnion {
    pub int  : i64,
    pub float: f64,
    pub uint : u64,
    pub unit : (),
    pub bool : bool,
    
    pub obj  : *mut Data,
    pub frame: *const StackFrame,
}


impl Data {
    pub fn new_unit() -> Data {
        Data {
            #[cfg(debug_assertions)]
            typ: TypeId::UNIT_TAG,
            data: DataUnion { unit: () },
        }
    }
}


#[derive(Clone)]
pub struct StackFrame {
    pub pc: ProgramCounter,
    pub save_point: ArenaSavePoint,
}


#[cfg(test)]
mod tests {
    use crate::{VM, register_stack::RegisterStack};

    #[test]
    fn test_alloc() {
        let mut vm = VM::new(&[], RegisterStack::new());

        unsafe { 
        let stack_frame = vm.alloc_stackframe();
        vm.data_stack.arena.alloc_new([255; 100]);
        {
            let stack_frame = vm.alloc_stackframe();
            vm.data_stack.arena.alloc_new([255; 100]);
            {
                let stack_frame = vm.alloc_stackframe();
                vm.data_stack.arena.alloc_new([255; 100]);
                {
                    let stack_frame = vm.alloc_stackframe();
                    vm.data_stack.arena.alloc_new([255; 100]);
                    {
            
                    }
                    vm.restore_stackframe(stack_frame);
            
                }
                vm.restore_stackframe(stack_frame);
            }
            vm.restore_stackframe(stack_frame);
        }
        vm.restore_stackframe(stack_frame);
        }
    }
}