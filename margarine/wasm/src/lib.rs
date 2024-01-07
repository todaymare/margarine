pub mod low_level;

use std::fmt::Write;

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use sti::{write, vec::Vec, string::String, arena::Arena};

#[derive(Debug, Clone, Copy)]
pub enum WasmType {
    I32,
    I64,
    F32,
    F64,
    Ptr { size: usize },
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
            WasmType::Ptr { .. } => "i32",
        }
    }

    pub const fn max_size_of_name() -> usize { 3 }

    pub const fn stack_size(self) -> usize {
        match self {
            WasmType::Ptr { size } => size,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockId(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopId {
    continue_id: usize,
    break_id: BlockId,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct StackPointer(usize);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MemoryAddress {
    address: usize,
    size: usize,
}

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


    pub fn add_string(&mut self, str: &'strs str) -> MemoryAddress {
        self.strs.push(str);
        let ptr = MemoryAddress {
            address: self.text_sec_size,
            size: str.len(),
        };
        self.text_sec_size += str.len();
        ptr
    }


    pub fn build(mut self, string_map: &mut StringMap<'strs>) -> Vec<u8> {
        self.functions.sort_unstable_by_key(|x| x.function_id.0);

        let mut buffer = String::new();
        write!(buffer, "(module ");
        
        assert!(self.memory >= self.stack_size);
        write!(buffer, "(memory (export \"memory\") {})", self.memory);

        let stack_pointer = self.memory;
        write!(buffer, "(global $stack_pointer (export \"stack_pointer\") (mut i32) (i32.const {}))", 
               stack_pointer);
        write!(buffer, "(global $bstack_pointer (export \"bstack_pointer\") i32 (i32.const {}))", 
               stack_pointer);

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
                WasmConstant::I32(v) => write!(buffer, "i32 (i32.const {v}))"),
                WasmConstant::I64(v) => write!(buffer, "i64 (i64.const {v}))"),
                WasmConstant::F32(v) => write!(buffer, "f32 (f32.const {v}))"),
                WasmConstant::F64(v) => write!(buffer, "f64 (f64.const {v}))"),
            }
        }

        for f in self.functions.iter() {
            write!(buffer, "(global $s_{} i32 (i32.const {}))",
                f.function_id.0, f.stack_size);
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
    stack_size: usize,

    loop_nest: usize,
    block_nest: usize,

    locals: Vec<WasmType, &'a Arena>,
    params: Vec<WasmType, &'a Arena>,

    finaliser: String<&'a Arena>,
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
            stack_size: 0,
            finaliser: String::new_in(arena),
        }
    }

    #[inline(always)]
    pub fn param(&mut self, ty: WasmType) -> LocalId {
        assert!(self.locals.is_empty());
        self.params.push(ty);
        LocalId(self.params.len() as u32 - 1)
    }


    #[inline(always)]
    pub fn prepend_param(&mut self, ty: WasmType) -> LocalId {
        assert!(self.locals.is_empty());
        self.params.push(ty);
        LocalId(self.params.len() as u32 - 1)
    }

    
    #[inline(always)]
    pub fn local(&mut self, ty: WasmType) -> LocalId {
        self.locals.push(ty);
        LocalId(self.params.len() as u32 + self.locals.len() as u32)
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


    pub fn offset(&self) -> usize { self.body.len() }


    pub fn set_finaliser(&mut self, string: String<&'a Arena>) { self.finaliser = string } 


    pub fn write_to(
        &mut self, str: &mut String<&'a Arena>,
        func: impl FnOnce(&mut WasmFunctionBuilder)) {
        std::mem::swap(&mut self.body, str);
        func(self);
        std::mem::swap(&mut self.body, str);
    }
}


impl WasmFunctionBuilder<'_> {
    pub fn build(mut self, string_map: &StringMap, buffer: &mut String) {
        self.ret();

        write!(buffer, "(func $_{} ", self.function_id.0);

        if let Some(export) = self.export {
            write!(buffer, "(export \"{}\") ", string_map.get(export));
        } else { write!(buffer, "(export \"{}\")", self.function_id.0) };

        for p in &self.params {
            write!(buffer, "(param {}) ", p.name());
        }


        if let Some(ret) = self.ret {
            if ret.stack_size() == 0 {
                write!(buffer, "(result {})", ret.name());
            }

            write!(buffer, "(local $_ret {})", ret.name());

        }

        for l in &self.locals {
            write!(buffer, "(local {}) ", l.name());
        }

        if let Some(WasmType::Ptr { .. }) = self.ret {
            write!(buffer, "(global.get $stack_pointer)"); 
        }

        write!(buffer, "(block $_ret ");
        buffer.reserve_exact(self.body.len() + 1);
        for i in self.body.trim_end().chars() {
            buffer.push_char(i)
        }
        write!(buffer, ")");

        buffer.push(&self.finaliser);

        if let Some(ret) = self.ret {
            write!(buffer, "local.get $_ret ");
            if ret.stack_size() != 0 {
                dbg!(&self);
                write!(buffer, "local.get {} ", self.params.len() - 1);
                write!(buffer, "i32.const {} ", ret.stack_size());
                write!(buffer, "call $memcpy ");
            }
        }
        write!(buffer, "return");


        buffer.push_char(')');

   }
}


impl<'a> WasmFunctionBuilder<'a> {
    ///
    /// This function expects:
    /// - A pointer to memory with type `ty`
    ///
    #[inline(always)]
    pub fn read(&mut self, ty: WasmType) {
        match ty {
            WasmType::I32 => self.i32_read(),
            WasmType::I64 => self.i64_read(),
            WasmType::F32 => self.f32_read(),
            WasmType::F64 => self.f64_read(),
            WasmType::Ptr { size } => {
                let ptr = self.alloc_stack(size);
                self.sptr_const(ptr);
                self.write(ty);
                self.sptr_const(ptr);
            },
        }
    }


    ///
    /// This function expects:
    /// `$ty`, `ptr($ty)` -> ()
    /// 
    #[inline(always)]
    pub fn write(&mut self, ty: WasmType) {
        match ty {
            WasmType::I32 => self.i32_write(),
            WasmType::I64 => self.i64_write(),
            WasmType::F32 => self.f32_write(),
            WasmType::F64 => self.f64_write(),
            WasmType::Ptr { size } => {
                self.i32_const(size.try_into().unwrap());
                self.call_template("memcpy");
            },
        }
    }


    ///
    /// Compares the equality of two values
    /// `$ty`, `$ty` -> `bool`
    ///
    pub fn eq(&mut self, ty: WasmType) {
        match ty {
            WasmType::I32 => self.i32_eq(),
            WasmType::I64 => self.i64_eq(),
            WasmType::F32 => self.f32_eq(),
            WasmType::F64 => self.f64_eq(),
            WasmType::Ptr { size } => self.ptr_veq(size),
        }
    }

    ///
    /// Compares the inequality of two values
    /// `$ty`, `$ty` -> `bool`
    ///
    pub fn ne(&mut self, ty: WasmType) {
        match ty {
            WasmType::I32 => self.i32_ne(),
            WasmType::I64 => self.i64_ne(),
            WasmType::F32 => self.f32_ne(),
            WasmType::F64 => self.f64_ne(),
            WasmType::Ptr { size } => self.ptr_veq(size),
        }
    }
   

    #[inline(always)]
    pub fn ite<T, A>(
        &mut self,
        value: &mut T,
        then_body: impl FnOnce(&mut T, &mut WasmFunctionBuilder) -> (LocalId, A),
        else_body: impl FnOnce((&mut T, LocalId), &mut WasmFunctionBuilder) -> A,
    ) -> (LocalId, A, A) {
        write!(self.body, "(if (then ");
        let (local, r1) = then_body(value, self);
        write!(self.body, ")(else ");
        let r2 = else_body((value, local), self);
        write!(self.body, "))");
        self.local_get(local);

        (local, r1, r2)
    }


    #[inline(always)]
    pub fn do_loop(
        &mut self,
        body: impl FnOnce(&mut Self, LoopId),
    ) {
        self.block(|wasm, id| {
            write!(wasm.body, "(loop $l{} ", wasm.loop_nest);
            wasm.loop_nest += 1;

            let id = LoopId {
                continue_id: wasm.loop_nest-1,
                break_id: id,
            };

            body(wasm, id);
            wasm.continue_loop(id);

            wasm.loop_nest += 1;
            write!(wasm.body, ")");
        });
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
        write!(self.body, ")");
    }
}



