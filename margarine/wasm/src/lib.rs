use std::{fmt::{Write, write}, ops::{DerefMut, Deref}};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use sti::{write, vec::Vec, string::String, arena::Arena};

#[derive(Debug, Clone, Copy)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    Ptr(usize),
}


#[derive(Debug, Clone, Copy)]
pub enum WasmConstant {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}


impl WasmType {
    pub const fn name(self) -> &'static str {
        match self {
            WasmType::I32 => "i32",
            WasmType::I64 => "i64",
            WasmType::F32 => "f32",
            WasmType::F64 => "f64",
            WasmType::Ptr(_) => "i32",
        }
    }

    pub const fn max_size_of_name() -> usize { 3 }

    pub const fn stack_size(self) -> usize {
        match self {
            WasmType::Ptr(v) => v,
            _ => 0,
        }
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FunctionId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalId(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GlobalId(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LoopId(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StackPointer(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Pointer(usize);

impl StackPointer {
    pub fn add(self, inc: usize) -> StackPointer {
        StackPointer(self.0 + inc)
    }
}


#[derive(Debug)]
pub struct WasmModuleBuilder<'a, 'strs> {
    pub arena: &'a Arena,
    functions: std::vec::Vec<WasmFunctionBuilder<'a>>,
    globals: Vec<WasmConstant>,
    memory: usize,
    stack_size: usize,

    strs: Vec<&'strs str>,
    text_sec_size: usize,

    function_id_counter: u32,
}


impl<'a, 'strs> WasmModuleBuilder<'a, 'strs> {
    pub fn new(arena: &'a Arena) -> Self { 
        Self {
            functions: std::vec::Vec::new(),
            function_id_counter: 0,
            globals: Vec::new(),
            memory: 0,
            stack_size: 0,
            arena,
            strs: Vec::new(),
            text_sec_size: 0,
        }
    }

    
    pub fn function_id(&mut self) -> FunctionId {
        self.function_id_counter += 1;
        FunctionId(self.function_id_counter - 1)
    }


    pub fn register(&mut self, function: WasmFunctionBuilder<'a>) {
        self.functions.push(function)
    }


    pub fn global(&mut self, constant: WasmConstant) -> GlobalId {
        self.globals.push(constant);
        GlobalId(self.globals.len() as u32 - 1)
    }


    pub fn memory(&mut self, init: usize) {
        self.memory = init;
    }


    pub fn stack_size(&mut self, size: usize) {
        self.stack_size = size;
        self.memory = self.memory.max(self.stack_size);
    }


    pub fn add_string(&mut self, str: &'strs str) -> Pointer {
        self.strs.push(str);
        let ptr = Pointer(self.text_sec_size);
        self.text_sec_size += str.len();
        ptr
    }


    pub fn build(mut self, string_map: &mut StringMap<'strs>) -> Vec<u8> {
        self.functions.sort_unstable_by_key(|x| x.function_id.0);

        let mut buffer = String::new();
        write!(buffer, "(module ");
        
        assert!(self.memory >= self.stack_size);
        write!(buffer, "(memory (export \"memory\") {})", self.memory);
        write!(buffer, "(global $stack_pointer (export \"stack_pointer\") (mut i32) (i32.const {}))", 
               self.stack_size);

        let _ = writeln!(buffer);
        let _ = writeln!(buffer, ";;");
        let _ = writeln!(buffer, ";; TEMPLATE START");
        let _ = writeln!(buffer, ";;");
        let _ = writeln!(buffer, include_str!("../template.wat"));
        let _ = writeln!(buffer, ";;");
        let _ = writeln!(buffer, ";; TEMPLATE OVER");
        let _ = writeln!(buffer, ";;");


        for g in self.globals.iter() {
            write!(buffer, "(global");
            match g {
                WasmConstant::I32(v) => write!(buffer, "i32 (i32.const {v})"),
                WasmConstant::I64(v) => write!(buffer, "i64 (i64.const {v})"),
                WasmConstant::F32(v) => write!(buffer, "f32 (f32.const {v})"),
                WasmConstant::F64(v) => write!(buffer, "f64 (f64.const {v})"),
            }
        }

        for f in self.functions.into_iter() {
            f.build(string_map, &mut buffer)
        }

        buffer.push_char(')');
        buffer.into_inner()
    }
}


#[derive(Debug)]
pub struct WasmFunctionBuilder<'a> {
    function_id: FunctionId,
    export: Option<StringIndex>,
    ret: Option<WasmType>,
    ret_offsets: Vec<usize, &'a Arena>,
    body: String<&'a Arena>,
    stack_size: usize,

    loop_nest: usize,
    block_nest: usize,

    locals: Vec<WasmType, &'a Arena>,
    params: Vec<WasmType, &'a Arena>,
}


impl<'a> WasmFunctionBuilder<'a> {
    pub fn new(arena: &'a Arena, id: FunctionId) -> Self { 
        Self { 
            ret: None, 
            export: None, 
            body: String::new_in(arena), 
            locals: Vec::new_in(arena), 
            params: Vec::new_in(arena), 
            ret_offsets: Vec::new_in(arena),
            function_id: id,
            loop_nest: 0,
            block_nest: 0,
            stack_size: 0,
        }
    }

    #[inline(always)]
    pub fn param(&mut self, ty: WasmType) -> LocalId {
        assert!(self.locals.is_empty());
        self.params.push(ty);
        LocalId(self.params.len() as u32 - 1)
    }

    
    #[inline(always)]
    pub fn local(&mut self, ty: WasmType) -> LocalId {
        self.locals.push(ty);
        LocalId(self.params.len() as u32 + self.locals.len() as u32 - 1)
    }


    #[inline(always)]
    pub fn return_value(&mut self, ty: WasmType) -> StackPointer {
        assert!(self.ret.is_none());

        self.ret.replace(ty);
        self.alloc_stack(ty.stack_size())
    }


    #[inline(always)]
    pub fn export(&mut self, string_index: StringIndex) {
        self.export.replace(string_index);
    }


    #[inline(always)]
    pub fn error(&mut self, err: ErrorId) {
        self.body.push(&format!("unreachable (; {err:?} ;)"));
    }
}


impl WasmFunctionBuilder<'_> {
    pub fn build(mut self, string_map: &StringMap, buffer: &mut String) {
        self.ret();
        

        write!(buffer, "(func $_{}", self.function_id.0);

        if let Some(export) = self.export {
            write!(buffer, "(export \"{}\") ", string_map.get(export));
        }

        for p in &self.params {
            write!(buffer, "(param {}) ", p.name());
        }

        for l in &self.locals {
            write!(buffer, "(local {}) ", l.name());
        }

        let mut ret_stack_size = 0; 
        if let Some(ret) = self.ret {
            write!(buffer, "(result {})", ret.name());
            ret_stack_size = ret.stack_size();
        }

        write!(buffer, "(call $push (i32.const {}))", self.stack_size - ret_stack_size);

        if let Some(WasmType::Ptr(_)) = self.ret {
            write!(buffer, "(global.get $stack_pointer)");   
        }

        let fmt = format!("(call $pop (i32.const {}))", self.stack_size);
        for r in &self.ret_offsets {
            unsafe { self.body.inner_mut() }.insert_from_slice(*r, fmt.as_bytes());
        }

        buffer.reserve_exact(self.body.len() + 1);
        for i in self.body.trim_end().chars() {
            buffer.push_char(i)
        }

        buffer.push_char(')');

   }
}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn local_get(&mut self, index: LocalId) { write!(self.body, "local.get {} ", index.0); }
    
    #[inline(always)]
    pub fn local_set(&mut self, index: LocalId) { write!(self.body, "local.set {} ", index.0); }
    
    #[inline(always)]
    pub fn local_tee(&mut self, index: LocalId) { write!(self.body, "local.tee {} ", index.0); }
    
    #[inline(always)]
    pub fn memory_size(&mut self) { write!(self.body, "memory.size "); }

    #[inline(always)]
    pub fn read_i32(&mut self) { write!(self.body, "i32.load "); }

    #[inline(always)]
    pub fn read_f32(&mut self) { write!(self.body, "f32.load "); }

    #[inline(always)]
    pub fn read_i64(&mut self) { write!(self.body, "i64.load "); }

    #[inline(always)]
    pub fn read_f64(&mut self) { write!(self.body, "f64.load "); }
}


impl<'a> WasmFunctionBuilder<'a> {
    #[inline(always)]
    pub fn global_get(&mut self, index: GlobalId) { write!(self.body, "global.get {}", index.0); }

    #[inline(always)]
    pub fn global_set(&mut self, index: GlobalId) { write!(self.body, "global.set {}", index.0); }

    #[inline(always)]
    pub fn call(&mut self, func: FunctionId) { write!(self.body, "call $_{} ", func.0); }

    #[inline(always)]
    pub fn pop(&mut self) { write!(self.body, "drop "); }

    #[inline(always)]
    pub fn unreachable(&mut self) { write!(self.body, "unreachable "); }

    #[inline(always)]
    pub fn alloc_stack(&mut self, size: usize) -> StackPointer {
        let ptr = StackPointer(size);
        self.stack_size += size; 
        ptr
    } 


    #[inline(always)]
    pub fn stack_offset(&mut self, ptr: Pointer) {
        write!(self.body, "(i32.sub (global.get $stack_pointer) (i32.const {})) ", ptr.0);
    }


    #[inline(always)]
    pub fn stack_to_global(&mut self, ptr: StackPointer) {
        write!(self.body, "(i32.add (global.get $stack_pointer) (i32.const {})) ", ptr.0);
    }


    #[inline(always)]
    pub fn write_i32_to_stack(&mut self, ptr: StackPointer) {
        self.stack_to_global(ptr);
        self.call_template("write_i32_to_stack")
    }


    #[inline(always)]
    pub fn write_i64_to_stack(&mut self, ptr: StackPointer) {
        self.stack_to_global(ptr);
        self.call_template("write_i64_to_stack")
    }


    #[inline(always)]
    pub fn write_f32_to_stack(&mut self, ptr: StackPointer) {
        self.stack_to_global(ptr);
        self.call_template("write_f32_to_stack")
    }


    #[inline(always)]
    pub fn write_f64_to_stack(&mut self, ptr: StackPointer) {
        self.stack_to_global(ptr);
        self.call_template("write_f64_to_stack")
    }


    #[inline(always)]
    pub fn memcpy(&mut self, ptr: StackPointer, ty: WasmType) {
        match ty {
            WasmType::I32 => self.write_i32_to_stack(ptr),
            WasmType::I64 => self.write_i64_to_stack(ptr),
            WasmType::F32 => self.write_f32_to_stack(ptr),
            WasmType::F64 => self.write_f64_to_stack(ptr),
            WasmType::Ptr(v) => {
                self.stack_to_global(ptr);
                self.i32_const(v.try_into().unwrap());
                self.call_template("memcpy");
            },
        }
    }


    
    #[inline(always)]
    pub fn call_template(&mut self, name: &str) {
        write!(self.body, "(call ${name}) ");
    }


    #[inline(always)]
    pub fn ite(
        &mut self,
        then_body: impl FnOnce(&mut WasmFunctionBuilder),
        else_body: impl FnOnce(&mut WasmFunctionBuilder),
    ) {
        write!(self.body, "(if (then ");
        then_body(self);
        write!(self.body, ")(else ");
        else_body(self);
        write!(self.body, "))");
    }


    #[inline(always)]
    pub fn do_loop(
        &mut self,
        body: impl FnOnce(&mut Self, LoopId),
    ) {
        write!(self.body, "(loop $l{} ", self.loop_nest);
        self.loop_nest += 1;

        body(self, LoopId(self.loop_nest-1));

        self.loop_nest += 1;
        write!(self.body, ")");
    }


    #[inline(always)]
    pub fn block(
        &mut self,
        body: impl FnOnce(&mut Self, BlockId),
    ) { 
        write!(self.body, "(block $b{} ", self.block_nest);
        self.block_nest += 1;

        body(self, BlockId(self.block_nest-1));

        self.block_nest -= 1;
    }


    #[inline(always)]
    pub fn break_block(&mut self, block: BlockId) { write!(self.body, "br $b{} ", block.0); }


    #[inline(always)]
    pub fn continue_loop(&mut self, loop_id: LoopId) { write!(self.body, "br $l{} ", loop_id.0); }


    #[inline(always)]
    pub fn ret(&mut self) { self.ret_offsets.push(self.body.len() - 1); write!(self.body, "return "); }
}



impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn ptr_const(&mut self, ptr: Pointer) { write!(self.body, "i32.const {} ", ptr.0); }

    #[inline(always)]
    pub fn bool_const(&mut self, v: bool) { write!(self.body, "i32.const {} ", v as i32); }
}


impl WasmFunctionBuilder<'_> {
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
}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn i64_const(&mut self, num: i64) { write!(self.body, "i64.const {num} "); }

    #[inline(always)]
    pub fn i64_eq(&mut self) { write!(self.body, "i64.eq "); }

    #[inline(always)]
    pub fn i64_ne(&mut self) { write!(self.body, "i64.ne "); }

    /// Checks if the value at the top of the stack is 0
    #[inline(always)]
    pub fn i64_eqz(&mut self) { write!(self.body, "i64.eqz"); }

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
}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn f32_const(&mut self, val: f32) { write!(self.body, "f32.const {val}"); }

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
}


impl WasmFunctionBuilder<'_> {
    #[inline(always)]
    pub fn f64_const(&mut self, val: f64) { write!(self.body, "f64.const {val}"); }

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
}



