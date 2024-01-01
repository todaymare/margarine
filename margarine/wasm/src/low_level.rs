use std::fmt::Write as _;

use sti::{write, format_in, arena_pool::ArenaPool};

use crate::{WasmFunctionBuilder, MemoryAddress, LocalId, GlobalId, FunctionId, StackPointer, LoopId, BlockId, WasmType};

impl WasmFunctionBuilder<'_> { 
    ///
    /// Inserts a `local.set` at a specific offset
    ///
    pub fn insert_local_set(&mut self, offset: usize, local: LocalId) -> usize {
        self.insert(format_in!(&*ArenaPool::tls_get_temp(), "local.set {}", local.0).as_str(), offset)
    }

    
    /// 
    /// Inserts a `drop` at a specified offset
    ///
    pub fn insert_drop(&mut self, offset: usize) -> usize {
        self.insert("drop ", offset)
    }


    ///
    /// Inserts `str` at a specific offset
    ///
    pub fn insert(&mut self, str: &str, offset: usize) -> usize {
        let vec = unsafe { self.body.inner_mut() };
        vec.insert_from_slice(offset, str.as_bytes());
        debug_assert!(std::str::from_utf8(vec).is_ok());
        str.len()
    } 

    ///
    /// Pushes a boolean constant on the stack
    /// () -> `bool`
    ///
    #[inline(always)]
    pub fn bool_const(&mut self, v: bool) { self.i32_const(v as i32); }

    ///
    /// Inverts a boolean
    /// `bool` -> `bool`
    ///
    #[inline(always)]
    pub fn bool_not(&mut self) {
        self.i64_const(-1);
        self.i64_bw_xor();
    }

    ///
    /// Pushes a unit value on the stack
    /// () -> `unit`
    ///
    #[inline(always)]
    pub fn unit(&mut self) { self.i64_const(0); }
}


impl WasmFunctionBuilder<'_> {
    ///
    /// Pushes the value of the specified local
    /// to the stack
    /// () -> `$local`
    ///
    #[inline(always)]
    pub fn local_get(&mut self, index: LocalId) { write!(self.body, "local.get {} ", index.0); }
    

    ///
    /// Sets the value of the specified local
    /// `$local` -> ()
    ///
    #[inline(always)]
    pub fn local_set(&mut self, index: LocalId) { write!(self.body, "local.set {} ", index.0); }
    
    /// 
    /// Sets & gets the value of the specified local
    /// `$local` -> `$local`
    ///
    #[inline(always)]
    pub fn local_tee(&mut self, index: LocalId) { write!(self.body, "local.tee {} ", index.0); }
    
    ///
    /// Pushes the memory size to the stack
    /// () -> `i32`
    /// 
    #[inline(always)]
    pub fn memory_size(&mut self) { write!(self.body, "memory.size "); }

    
    ///
    /// Pushes the specified global's value to the stack
    /// () -> `$global`
    ///
    #[inline(always)]
    pub fn global_get(&mut self, index: GlobalId) { write!(self.body, "global.get {}", index.0); }

    ///
    /// Sets the value of the specified global
    /// `$global` -> ()
    ///
    #[inline(always)]
    pub fn global_set(&mut self, index: GlobalId) { write!(self.body, "global.set {}", index.0); }

    ///
    /// Calls a function
    /// for `$arg` in `0..argc` -> `$ret`
    ///
    #[inline(always)]
    pub fn call(&mut self, func: FunctionId) {
        write!(self.body, "global.get $s_{}", self.function_id.0);
        self.call_template("push");

        write!(self.body, "call $_{} ", func.0);

        write!(self.body, "global.get $s_{}", self.function_id.0);
        self.call_template("pop");
    }

    ///
    /// Removes a value from the stack
    /// `$T` -> ()
    ///
    #[inline(always)]
    pub fn pop(&mut self) { write!(self.body, "drop "); }

    ///
    /// Marks this branch as unreachable
    /// () -> ()
    ///
    #[inline(always)]
    pub fn unreachable(&mut self) { write!(self.body, "unreachable "); }

    ///
    /// Allocates space on the stack with
    /// the given size and returns a sptr to it
    /// () -> ()
    ///
    #[inline(always)]
    pub fn alloc_stack(&mut self, size: usize) -> StackPointer {
        let ptr = StackPointer(self.stack_size);
        self.stack_size += size; 
        ptr
    } 


    ///
    /// Puts a stack pointer on the stack as a raw memory address
    /// () -> `ptr`
    ///
    #[inline(always)]
    pub fn sptr_const(&mut self, ptr: StackPointer) {
        if ptr.0 == 0 { write!(self.body, "(global.get $stack_pointer) "); return };
        write!(self.body, "(i32.add (global.get $stack_pointer) (i32.const {})) ", ptr.0);
    } 


    ///
    /// Calls a raw template
    /// `?` -> `?`
    ///
    /// Please don't use this.
    ///
    #[inline(always)]
    pub fn call_template(&mut self, name: &str) {
        write!(self.body, "(call ${name}) ");
    }


    ///
    /// Breaks out of a specific loop
    ///
    #[inline(always)]
    pub fn break_loop(&mut self, loop_id: LoopId) { self.break_block(loop_id.break_id); }

    ///
    /// Continues a loop from the start 
    ///
    #[inline(always)]
    pub fn continue_loop(&mut self, loop_id: LoopId) { write!(self.body, "br $l{} ", loop_id.continue_id); }

    ///
    /// Appends a raw string
    ///
    #[inline(always)]
    pub fn raw(&mut self, str: &str) { self.body.push(str); }

    ///
    /// Breaks out of a specific block
    /// 
    #[inline(always)]
    pub fn break_block(&mut self, block: BlockId) { write!(self.body, "br $b{} ", block.0); }

    ///
    /// Returns from the current function
    /// `$ret` -> `unreachable`
    ///
    #[inline(always)]
    pub fn ret(&mut self) {
        if self.ret.is_some() { write!(self.body, "local.set $_ret "); }
        write!(self.body, "br $_ret ");
    }
}


impl WasmFunctionBuilder<'_> {
    ///
    /// Pushes a raw memory address to the stack
    /// () -> `ptr($t)`
    #[inline(always)]
    pub fn ptr_const(&mut self, ptr: MemoryAddress) {
        self.i32_const(ptr.address.try_into().unwrap());
    }


    ///
    /// Compares the equality of two pointer **addresses**
    /// `ptr($ty)`, `ptr($ty)` -> `bool`
    ///
    #[inline(always)]
    pub fn ptr_eq(&mut self) {
        self.i32_eq()
    }


    ///
    /// Compares the value of two pointers
    /// `ptr($ty)`, `ptr($ty)` -> `bool`
    ///
    #[inline(always)]
    pub fn ptr_veq(&mut self, size: usize) {
        self.i32_const(size.try_into().unwrap());
        self.call_template("bcmp");
    }
}


impl WasmFunctionBuilder<'_> {
    ///
    /// Pushes an `i32` constant to the stack
    /// () -> `i32`
    ///
    #[inline(always)]
    pub fn i32_const(&mut self, num: i32) { write!(self.body, "i32.const {num} "); }

    #[inline(always)]
    pub fn i32_eq(&mut self) { write!(self.body, "i32.eq "); }

    #[inline(always)]
    pub fn i32_ne(&mut self) { write!(self.body, "i32.ne "); }

    /// Checks if the value at the top of the stack is 0
    #[inline(always)]
    pub fn i32_eqz(&mut self) { write!(self.body, "i32.eqz"); }

    #[inline(always)]
    pub fn i32_gt(&mut self) { write!(self.body, "i32.gt_s "); }

    #[inline(always)]
    pub fn i32_ge(&mut self) { write!(self.body, "i32.ge_s "); }

    #[inline(always)]
    pub fn i32_lt(&mut self) { write!(self.body, "i32.lt_s "); }

    #[inline(always)]
    pub fn i32_le(&mut self) { write!(self.body, "i32.le_s "); }

    #[inline(always)]
    pub fn i32_add(&mut self) { write!(self.body, "i32.add "); }

    #[inline(always)]
    pub fn i32_sub(&mut self) { write!(self.body, "i32.sub "); }

    #[inline(always)]
    pub fn i32_mul(&mut self) { write!(self.body, "i32.mul "); }

    #[inline(always)]
    pub fn i32_div(&mut self) { write!(self.body, "i32.div_s "); }

    #[inline(always)]
    pub fn i32_rem(&mut self) { write!(self.body, "i32.rem_s "); }
    
    #[inline(always)]
    pub fn i32_as_i64(&mut self) { write!(self.body, "i64.extend_i32_s "); }

    #[inline(always)]
    pub fn i32_as_f32(&mut self) { write!(self.body, "f32.convert_i32_s "); }
    
    #[inline(always)]
    pub fn i32_as_f64(&mut self) { write!(self.body, "f64.convert_i32_s "); }

    #[inline(always)]
    pub fn i32_reinterp_f32(&mut self) { write!(self.body, "f32.reinterpret_i32 "); }

    #[inline(always)]
    pub fn i32_reinterp_f64(&mut self) { write!(self.body, "f64.reinterpret_i32 "); }

    #[inline(always)]
    pub fn i32_bw_and(&mut self) { write!(self.body, "i32.and"); }

    #[inline(always)]
    pub fn i32_bw_or(&mut self) { write!(self.body, "i32.or"); }

    #[inline(always)]
    pub fn i32_bw_xor(&mut self) { write!(self.body, "i32.xor"); }

    #[inline(always)]
    pub fn i32_bw_left_shift(&mut self) { write!(self.body, "i32.shl"); }

    #[inline(always)]
    pub fn i32_bw_right_shift(&mut self) { write!(self.body, "i32.shr_s"); }

    #[inline(always)]
    pub fn i32_bw_rotate_left(&mut self) { write!(self.body, "i32.rotl"); }

    #[inline(always)]
    pub fn i32_bw_rotate_right(&mut self) { write!(self.body, "i32.rotr"); }

    #[inline(always)]
    pub fn i32_bw_clz(&mut self) { write!(self.body, "i32.clz"); }

    #[inline(always)]
    pub fn i32_bw_ctz(&mut self) { write!(self.body, "i32.ctz"); }

    #[inline(always)]
    pub fn i32_bw_popcunt(&mut self) { write!(self.body, "i32.popcnt"); }
 
    ///
    /// Reads an `i32` at a pointer
    /// `ptr(i32)` -> `i32`
    ///
    #[inline(always)]
    pub fn i32_read(&mut self) { write!(self.body, "i32.load "); }


    /// 
    /// Writes an `i32` to the given pointer
    /// `i32`, `ptr(i32)` -> ()
    ///
    #[inline(always)]
    pub fn i32_write(&mut self) {
        self.call_template("write_i32_to_stack")
    }

}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn u32_gt(&mut self) { write!(self.body, "i32.gt_u "); }

    #[inline(always)]
    pub fn u32_ge(&mut self) { write!(self.body, "i32.ge_u "); }

    #[inline(always)]
    pub fn u32_lt(&mut self) { write!(self.body, "i32.lt_u "); }

    #[inline(always)]
    pub fn u32_le(&mut self) { write!(self.body, "i32.le_u "); }

    #[inline(always)]
    pub fn u32_div(&mut self) { write!(self.body, "i32.div_u "); }

    #[inline(always)]
    pub fn u32_rem(&mut self) { write!(self.body, "i32.rem_u "); }
 
    #[inline(always)]
    pub fn u32_as_i64(&mut self) { write!(self.body, "i64.extend_i32_s "); }

    #[inline(always)]
    pub fn u32_as_f32(&mut self) { write!(self.body, "f32.convert_i32_u "); }

    #[inline(always)]
    pub fn u32_as_f64(&mut self) { write!(self.body, "f32.convert_i64_u "); }

    #[inline(always)]
    pub fn u32_bw_right_shift(&mut self) { write!(self.body, "i32.shr_u"); }

    ///
    /// Reads an `u32` at a pointer
    /// `ptr(u32)` -> `u32`
    ///
    #[inline(always)]
    pub fn u32_read(&mut self) { self.i32_read(); }

    /// 
    /// Writes a `u32` to the given pointer
    /// `u32`, `ptr(u32)` -> ()
    ///
    #[inline(always)]
    pub fn u32_write(&mut self) { self.i32_write() }
}


impl WasmFunctionBuilder<'_> {
    ///
    /// Pushes an i64 constant to the stack
    ///
    #[inline(always)]
    pub fn i64_const(&mut self, num: i64) { write!(self.body, "i64.const {num} "); }


    ///
    /// Negates the i64 on the top of the stack
    ///
    /// This function expects:
    /// - A i64 on the stack
    ///
    #[inline(always)]
    pub fn i64_neg(&mut self) {
        self.i64_const(-1);
        self.i64_mul();
    }


    /// 
    /// Compares the equality of the
    /// top 2 values on the stack
    ///
    /// This function expects:
    /// - 2, i64s on the stack
    ///
    #[inline(always)]
    pub fn i64_eq(&mut self) { write!(self.body, "i64.eq "); }


    /// 
    /// Compares the inequality of the
    /// top 2 values on the stack
    ///
    /// This function expects:
    /// - 2, i64s on the stack
    ///
    #[inline(always)]
    pub fn i64_ne(&mut self) { write!(self.body, "i64.ne "); }


    /// 
    /// Checks if the value at the top of the stack is 0
    ///
    /// This function expects:
    /// - An i64 on the stack
    ///
    #[inline(always)]
    pub fn i64_eqz(&mut self) { write!(self.body, "i64.eqz "); }

    #[inline(always)]
    pub fn i64_gt(&mut self) { write!(self.body, "i64.gt_s "); }

    #[inline(always)]
    pub fn i64_ge(&mut self) { write!(self.body, "i64.ge_s "); }

    #[inline(always)]
    pub fn i64_lt(&mut self) { write!(self.body, "i64.lt_s "); }

    #[inline(always)]
    pub fn i64_le(&mut self) { write!(self.body, "i64.le_s "); }

    #[inline(always)]
    pub fn i64_add(&mut self) { write!(self.body, "i64.add "); }

    #[inline(always)]
    pub fn i64_sub(&mut self) { write!(self.body, "i64.sub "); }

    #[inline(always)]
    pub fn i64_mul(&mut self) { write!(self.body, "i64.mul "); }

    #[inline(always)]
    pub fn i64_div(&mut self) { write!(self.body, "i64.div_s "); }

    #[inline(always)]
    pub fn i64_rem(&mut self) { write!(self.body, "i64.rem_s "); }
    
    #[inline(always)]
    pub fn i64_as_i32(&mut self) { write!(self.body, "i32.wrap_i64 "); }

    #[inline(always)]
    pub fn i64_as_f32(&mut self) { write!(self.body, "f32.convert_i64_s "); }

    #[inline(always)]
    pub fn i64_as_f64(&mut self) { write!(self.body, "f64.convert_i64_s "); }

    #[inline(always)]
    pub fn i64_reinterp_f32(&mut self) { write!(self.body, "f32.reinterpret_i64 "); }

    #[inline(always)]
    pub fn i64_reinterp_f64(&mut self) { write!(self.body, "f64.reinterpret_i64 "); }

    #[inline(always)]
    pub fn i64_bw_and(&mut self) { write!(self.body, "i64.and"); }

    #[inline(always)]
    pub fn i64_bw_or(&mut self) { write!(self.body, "i64.or"); }

    #[inline(always)]
    pub fn i64_bw_xor(&mut self) { write!(self.body, "i64.xor"); }

    #[inline(always)]
    pub fn i64_bw_left_shift(&mut self) { write!(self.body, "i64.shl"); }

    #[inline(always)]
    pub fn i64_bw_right_shift(&mut self) { write!(self.body, "i64.shr_s"); }

    #[inline(always)]
    pub fn i64_bw_rotate_left(&mut self) { write!(self.body, "i64.rotl"); }

    #[inline(always)]
    pub fn i64_bw_rotate_right(&mut self) { write!(self.body, "i64.rotr"); }

    #[inline(always)]
    pub fn i64_bw_clz(&mut self) { write!(self.body, "i64.clz"); }

    #[inline(always)]
    pub fn i64_bw_ctz(&mut self) { write!(self.body, "i64.ctz"); }

    #[inline(always)]
    pub fn i64_bw_popcunt(&mut self) { write!(self.body, "i64.popcnt"); }

    ///
    /// Reads a `i64` at a pointer
    /// `ptr(i64)` -> `i64`
    ///
    #[inline(always)]
    pub fn i64_read(&mut self) { write!(self.body, "i64.load "); }


    /// 
    /// Writes an `i64` to the given pointer
    /// `i64`, `ptr(i64)` -> ()
    ///
    #[inline(always)]
    pub fn i64_write(&mut self) {
        self.call_template("write_i64_to_stack")
    }

}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn u64_gt(&mut self) { write!(self.body, "i64.gt_u "); }

    #[inline(always)]
    pub fn u64_ge(&mut self) { write!(self.body, "i64.ge_u "); }

    #[inline(always)]
    pub fn u64_lt(&mut self) { write!(self.body, "i64.lt_u "); }

    #[inline(always)]
    pub fn u64_le(&mut self) { write!(self.body, "i64.le_u "); }

    #[inline(always)]
    pub fn u64_div(&mut self) { write!(self.body, "i64.div_u "); }

    #[inline(always)]
    pub fn u64_rem(&mut self) { write!(self.body, "i64.rem_u "); }

    #[inline(always)]
    pub fn u64_as_f32(&mut self) { write!(self.body, "f32.convert_i64_u "); }

    #[inline(always)]
    pub fn u64_as_f64(&mut self) { write!(self.body, "f64.convert_i64_u "); }

    #[inline(always)]
    pub fn u64_bw_right_shift(&mut self) { write!(self.body, "i64.shr_u"); }

    ///
    /// Reads a `u64` at a pointer
    /// `ptr(u64)` -> `u64`
    ///
    #[inline(always)]
    pub fn u64_read(&mut self) { self.i64_read(); }

    /// 
    /// Writes an `i64` to the given pointer
    /// `i64`, `ptr(i64)` -> ()
    ///
    #[inline(always)]
    pub fn u64_write(&mut self) { self.i64_write() }

}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn f32_const(&mut self, val: f32) { write!(self.body, "f32.const {val} "); }

    #[inline(always)]
    pub fn f32_eq(&mut self) { write!(self.body, "f32.eq "); }

    #[inline(always)]
    pub fn f32_ne(&mut self) { write!(self.body, "f32.ne "); }

    #[inline(always)]
    pub fn f32_gt(&mut self) { write!(self.body, "f32.gt "); }

    #[inline(always)]
    pub fn f32_ge(&mut self) { write!(self.body, "f32.ge "); }

    #[inline(always)]
    pub fn f32_lt(&mut self) { write!(self.body, "f32.lt "); }

    #[inline(always)]
    pub fn f32_le(&mut self) { write!(self.body, "f32.le "); }

    #[inline(always)]
    pub fn f32_add(&mut self) { write!(self.body, "f32.add "); }

    #[inline(always)]
    pub fn f32_sub(&mut self) { write!(self.body, "f32.sub "); }

    #[inline(always)]
    pub fn f32_mul(&mut self) { write!(self.body, "f32.mul "); }

    #[inline(always)]
    pub fn f32_div(&mut self) { write!(self.body, "f32.div "); }

    #[inline(always)]
    pub fn f32_rem(&mut self) { write!(self.body, "f32.rem "); }
    
    #[inline(always)]
    pub fn f32_as_f64(&mut self) { write!(self.body, "f64.promote_f32 "); }

    #[inline(always)]
    pub fn f32_as_i32(&mut self) { write!(self.body, "i32.trunc_f32_s "); }

    #[inline(always)]
    pub fn f32_as_u32(&mut self) { write!(self.body, "i32.trunc_f32_u "); }

    #[inline(always)]
    pub fn f32_as_i64(&mut self) { write!(self.body, "i64.trunc_f32_u "); }

    #[inline(always)]
    pub fn f32_as_u64(&mut self) { write!(self.body, "i64.trunc_f32_u "); }

    #[inline(always)]
    pub fn f32_reinterp_i32(&mut self) { write!(self.body, "i32.reinterpret_f32 "); }

    #[inline(always)]
    pub fn f32_reinterp_i64(&mut self) { write!(self.body, "i64.reinterpret_f32 "); }

    #[inline(always)]
    pub fn f32_min(&mut self) { write!(self.body, "f32.min "); }

    #[inline(always)]
    pub fn f32_max(&mut self) { write!(self.body, "f32.max "); }

    #[inline(always)]
    pub fn f32_round(&mut self) { write!(self.body, "f32.nearest "); }

    #[inline(always)]
    pub fn f32_ceil(&mut self) { write!(self.body, "f32.ceil "); }

    #[inline(always)]
    pub fn f32_floor(&mut self) { write!(self.body, "f32.floor "); }

    #[inline(always)]
    pub fn f32_truncate(&mut self) { write!(self.body, "f32.trunc "); }

    #[inline(always)]
    pub fn f32_abs(&mut self) { write!(self.body, "f32.abs "); }

    #[inline(always)]
    pub fn f32_neg(&mut self) { write!(self.body, "f32.neg "); }

    #[inline(always)]
    pub fn f32_sqrt(&mut self) { write!(self.body, "f32.sqrt "); }

    #[inline(always)]
    pub fn f32_copysign(&mut self) { write!(self.body, "f32.copysign "); }
   
    /// 
    /// Reads a `f32` at a pointer
    /// `ptr(f32)` -> `f32`
    ///
    #[inline(always)]
    pub fn f32_read(&mut self) { write!(self.body, "f32.load "); }


    /// 
    /// Writes an `f32` to the given pointer
    /// `f32`, `ptr(f32)` -> ()
    ///
    #[inline(always)]
    pub fn f32_write(&mut self) {
        self.call_template("write_f32_to_stack")
    }
}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn f64_const(&mut self, val: f64) { write!(self.body, "f64.const {val} "); }

    #[inline(always)]
    pub fn f64_eq(&mut self) { write!(self.body, "f64.eq "); }

    #[inline(always)]
    pub fn f64_ne(&mut self) { write!(self.body, "f64.ne "); }

    #[inline(always)]
    pub fn f64_gt(&mut self) { write!(self.body, "f64.gt "); }

    #[inline(always)]
    pub fn f64_ge(&mut self) { write!(self.body, "f64.ge "); }

    #[inline(always)]
    pub fn f64_lt(&mut self) { write!(self.body, "f64.lt "); }

    #[inline(always)]
    pub fn f64_le(&mut self) { write!(self.body, "f64.le "); }

    #[inline(always)]
    pub fn f64_add(&mut self) { write!(self.body, "f64.add "); }

    #[inline(always)]
    pub fn f64_sub(&mut self) { write!(self.body, "f64.sub "); }

    #[inline(always)]
    pub fn f64_mul(&mut self) { write!(self.body, "f64.mul "); }

    #[inline(always)]
    pub fn f64_div(&mut self) { write!(self.body, "f64.div "); }

    #[inline(always)]
    pub fn f64_rem(&mut self) { write!(self.body, "f64.rem "); }
    
    #[inline(always)]
    pub fn f64_as_f32(&mut self) { write!(self.body, "f32.demote_f64 "); }

    #[inline(always)]
    pub fn f64_as_i32(&mut self) { write!(self.body, "i32.trunc_f64_s "); }

    #[inline(always)]
    pub fn f64_as_u32(&mut self) { write!(self.body, "i32.trunc_f64_u "); }

    #[inline(always)]
    pub fn f64_as_i64(&mut self) { write!(self.body, "i64.trunc_f64_u "); }

    #[inline(always)]
    pub fn f64_as_u64(&mut self) { write!(self.body, "i64.trunc_f64_u "); }

    #[inline(always)]
    pub fn f64_reinterp_i32(&mut self) { write!(self.body, "i32.reinterpret_f64 "); }

    #[inline(always)]
    pub fn f64_reinterp_i64(&mut self) { write!(self.body, "i64.reinterpret_f64 "); }

    #[inline(always)]
    pub fn f64_min(&mut self) { write!(self.body, "f64.min "); }

    #[inline(always)]
    pub fn f64_max(&mut self) { write!(self.body, "f64.max "); }

    #[inline(always)]
    pub fn f64_round(&mut self) { write!(self.body, "f64.nearest "); }

    #[inline(always)]
    pub fn f64_ceil(&mut self) { write!(self.body, "f64.ceil "); }

    #[inline(always)]
    pub fn f64_floor(&mut self) { write!(self.body, "f64.floor "); }

    #[inline(always)]
    pub fn f64_truncate(&mut self) { write!(self.body, "f64.trunc "); }

    #[inline(always)]
    pub fn f64_abs(&mut self) { write!(self.body, "f64.abs "); }

    #[inline(always)]
    pub fn f64_neg(&mut self) { write!(self.body, "f64.neg "); }

    #[inline(always)]
    pub fn f64_sqrt(&mut self) { write!(self.body, "f64.sqrt "); }

    #[inline(always)]
    pub fn f64_copysign(&mut self) { write!(self.body, "f64.copysign "); }

    ///
    /// Reads a `f64` at a pointer
    /// `ptr(f64)` -> `f64`
    #[inline(always)]
    pub fn f64_read(&mut self) { write!(self.body, "f64.load "); }


    /// 
    /// Writes an `f64` to the given pointer
    /// `f64`, `ptr(f64)` -> ()
    ///
    #[inline(always)]
    pub fn f64_write(&mut self) {
        self.call_template("write_f64_to_stack")
    }

}



