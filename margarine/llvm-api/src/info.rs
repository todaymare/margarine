use std::{ffi::CStr, ptr::NonNull};

use llvm_sys::core::LLVMDisposeMessage;

/// An LLVM message 
pub struct Message {
    msg: NonNull<i8>,
}


impl Message {

    /// # Safety
    /// Undefined behaviour if the string is not from LLVM
    pub unsafe fn new(msg: NonNull<i8>) -> Self {
        Self { msg }
    }


    pub fn as_str(&'_ self) -> &'_ str {
        let slice = unsafe { CStr::from_ptr(self.msg.as_ptr()) };
        slice.to_str().unwrap()
    }
}


impl Drop for Message {
    fn drop(&mut self) {
        unsafe { LLVMDisposeMessage(self.msg.as_ptr()) }
    }
}


impl core::fmt::Display for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
