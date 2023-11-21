use std::{fmt::Write, ops::{DerefMut, Deref}};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use sti::{write, vec::Vec, string::String, arena::Arena};

#[derive(Debug, Clone, Copy)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
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
        }
    }

    pub const fn max_size_of_name() -> usize { 3 }
}


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FunctionId(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LocalId(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct GlobalId(u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct LoopId(usize);


#[derive(Debug)]
pub struct WasmModuleBuilder<'a> {
    functions: std::vec::Vec<WasmFunctionBuilder<'a>>,
    globals: Vec<WasmConstant>,
    memory: usize,

    function_id_counter: u32,
}


impl<'a> WasmModuleBuilder<'a> {
    pub fn new() -> Self { 
        Self {
            functions: std::vec::Vec::new(),
            function_id_counter: 0,
            globals: Vec::new(),
            memory: 0,
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


    pub fn build(mut self, string_map: &mut StringMap) -> Vec<u8> {
        self.functions.sort_unstable_by_key(|x| x.function_id.0);

        let mut buffer = String::new();
        write!(buffer, "(module ");
        
        write!(buffer, "(memory {})", self.memory);

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
    body: String<&'a Arena>,

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
            function_id: id,
            loop_nest: 0,
            block_nest: 0,
        }
    }

    #[inline(always)]
    pub fn param(&mut self, ty: WasmType) -> LocalId {
        assert!(!self.locals.is_empty());
        self.params.push(ty);
        LocalId(self.params.len() as u32 - 1)
    }

    
    #[inline(always)]
    pub fn local(&mut self, ty: WasmType) -> LocalId {
        self.locals.push(ty);
        LocalId(self.params.len() as u32 + self.locals.len() as u32 - 1)
    }


    #[inline(always)]
    pub fn return_value(&mut self, ty: WasmType) {
        self.ret.replace(ty);
    }


    #[inline(always)]
    pub fn export(&mut self, string_index: StringIndex) {
        self.export.replace(string_index);
    }


    #[inline(always)]
    pub fn error(&mut self, err: ErrorId) {
        self.body.push("unreachable ");
    }
}


impl WasmFunctionBuilder<'_> {
    pub fn build(self, string_map: &StringMap, buffer: &mut String) {
        write!(buffer, "(func ");

        if let Some(export) = self.export {
            write!(buffer, "(export \"{}\") ", string_map.get(export));
        }

        for p in &self.params {
            write!(buffer, "(param {}) ", p.name());
        }

        for l in &self.locals {
            write!(buffer, "(local {}) ", l.name());
        }

        if let Some(ret) = self.ret {
            write!(buffer, "(result {})", ret.name());
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

    #[inline(always)]
    pub fn write_i32(&mut self) { write!(self.body, "i32.store "); }

    #[inline(always)]
    pub fn write_f32(&mut self) { write!(self.body, "f32.store "); }

    #[inline(always)]
    pub fn write_i64(&mut self) { write!(self.body, "i64.store "); }

    #[inline(always)]
    pub fn write_f64(&mut self) { write!(self.body, "f64.store "); }
}


impl<'a> WasmFunctionBuilder<'a> {
    #[inline(always)]
    pub fn global_get(&mut self, index: GlobalId) { write!(self.body, "global.get {}", index.0); }

    #[inline(always)]
    pub fn global_set(&mut self, index: GlobalId) { write!(self.body, "global.set {}", index.0); }

    #[inline(always)]
    pub fn call(&mut self, func: FunctionId) {
        write!(self.body, "call {} ", func.0);
    }

    #[inline(always)]
    pub fn pop(&mut self) { write!(self.body, "drop "); }

    #[inline(always)]
    pub fn unreachable(&mut self) { write!(self.body, "unreachable "); }

        
    #[inline(always)]
    pub fn ite(
        &mut self,
        then_body: impl Fn(&mut WasmFunctionBuilder),
        else_body: impl Fn(&mut WasmFunctionBuilder),
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
        body: impl Fn(&mut Self, LoopId),
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
        body: impl Fn(&mut Self, BlockId),
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
    pub fn ret(&mut self) { write!(self.body, "return "); }
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
    pub fn f64_const(&mut self, val: f32) { write!(self.body, "f64.const {val}"); }

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



