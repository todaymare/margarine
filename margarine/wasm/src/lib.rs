pub mod low_level;

use std::{fmt::Write, collections::HashMap};

use common::string_map::{StringIndex, StringMap};
use errors::ErrorId;
use sti::{write, vec::Vec, string::String, arena::Arena};

#[derive(Debug, Clone, Copy)]
pub enum WasmType {
    I8,
    I16,
    U8,
    U16,
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
    pub const fn param_ty_name(self) -> &'static str {
        match self {
            WasmType::I8  => "i32",
            WasmType::I16 => "i32",
            WasmType::U8  => "i32",
            WasmType::U16 => "i32",
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


#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
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
pub struct StringAddress {
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
    pub externs: HashMap<StringIndex, Vec<(StringIndex, FunctionId)>>,
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
            externs: HashMap::new(),
        }
    }

    
    pub fn extern_func(&mut self, path: StringIndex, name: StringIndex) -> FunctionId {
        let fid = self.function_id();
        self.externs.entry(path).or_insert(Vec::new()).push((name, fid));
        fid
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
        self.stack_size += 1024; // For assuming the last KB is not used
        self.memory = self.memory.max((self.stack_size / (64 * 1024)) + 1);
    }


    pub fn add_string(&mut self, str: &'strs str) -> StringAddress {
        self.strs.push(str);
        let ptr = StringAddress {
            address: self.text_sec_size,
            size: str.len(),
        };
        self.text_sec_size += str.len();
        ptr
    }


    pub fn build(&mut self, string_map: &mut StringMap<'strs>) -> Vec<u8> {
        self.functions.sort_unstable_by_key(|x| x.function_id.0);

        let mut buffer = String::new();
        write!(buffer, "(module ");
        
        assert!(self.memory * 64 * 1024 >= self.stack_size);

        write!(buffer, r#"(func $alloc (import "::host" "alloc") (param i32) (result i32))"#);
        write!(buffer, r#"(func $free (import "::host" "free") (param i32))"#);
        write!(buffer, r#"(func $dump_stack_trace (import "::host" "dump_stack_trace"))"#);
        write!(buffer, r#"(func $compiler_error (import "::host" "compiler_error") (param i32) (param i32) (param i32))"#);
        write!(buffer, r#"(func $panic (import "::host" "panic"))"#);
        write!(buffer, r#"(func $printi32 (import "::host" "printi32") (param i32))"#);
        write!(buffer, r#"(func $printi64 (import "::host" "printi64") (param i64))"#);
        write!(buffer, r#"(func $printvar (import "::host" "printvar") (param i32) (param i32))"#);

        for (path, funcs) in &self.externs {
            let path = string_map.get(*path);
            for (name, id) in funcs {
                let name = string_map.get(*name);
                write!(buffer, r#"(func $_{} (import "{}" "{}") (param i32))"#, id.0, path, name);
            }
        }

        write!(buffer, "(memory (export \"program_memory\") {})", self.memory);

        write!(buffer, "(global $panic_reason (export \"panic_reason\") (mut i32) (i32.const {}))", 0);
        write!(buffer, "(global $panic_len (export \"panic_len\") (mut i32) (i32.const {}))", 0);

        let string_pointer = self.stack_size;
        write!(buffer, "(global $string_pointer i32 (i32.const {}))", string_pointer);

        {
            let mut c = string_pointer;
            for f in &self.strs {
                write!(buffer, "(data (i32.const {c}) \"\\01\\00\\00\\00\\00\\00\\00\\00");
                for i in f.chars() {
                    match i {
                        '\n' => buffer.push("\\n"),
                        '\r' => buffer.push("\\r"),
                        '\t' => buffer.push("\\t"),
                        '\\' => buffer.push("\\\\"),
                        '\0' => buffer.push("\\0"),
                        '\"' => buffer.push("\\\""),
                        _ => buffer.push_char(i),
                    }
                }
                write!(buffer, "\")");
                c += f.len() + 8;
            }
        }

        let stack_pointer = self.stack_size;
        write!(buffer, "(global $stack_pointer (export \"stack_pointer\") (mut i32) (i32.const {}))", 
               stack_pointer);
        write!(buffer, "(global $bstack_pointer (export \"bstack_pointer\") i32 (i32.const {}))", 
               stack_pointer);

        write!(buffer, "(global $heap_start (export \"heap_start\") (mut i32) (i32.const {}))", 
               stack_pointer + self.strs.iter().map(|x| x.len() + 8).sum::<usize>());

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

        for f in self.functions.iter_mut() {
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

    temporary_i32: Option<LocalId>,
    temporary_i32_2: Option<LocalId>,
    temporary_i64: Option<LocalId>,
    temporary_i64_2: Option<LocalId>,
    temporary_f32: Option<LocalId>,
    temporary_f64: Option<LocalId>,
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
            stack_size: 8, // prev_sp + func_id
            finaliser: String::new_in(arena),

            temporary_i32: None,
            temporary_i32_2: None,
            temporary_i64: None,
            temporary_i64_2: None,
            temporary_f32: None,
            temporary_f64: None,
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
        let num = match err {
            ErrorId::Lexer(v)  => { self.u32_const(0); self.u32_const(v.0); v.1.inner() },
            ErrorId::Parser(v) => { self.u32_const(1); self.u32_const(v.0); v.1.inner() },
            ErrorId::Sema(v)   => { self.u32_const(2); self.u32_const(0);   v.inner() },
        };

        self.u32_const(num);

        self.call_template("compiler_error");
        self.panic("compiler error encountered");
    }


    pub fn panic(&mut self, str: &str) {
        let len = ((str.len() + 7) / 8) * 8;
        self.u32_const(len as u32);
        self.call_template("alloc");
        write!(self.body, "global.set $panic_reason ");

        {
            let mut iter = str.bytes();

            let mut count = 0;
            loop { 
                let Some(n0) = iter.next()
                else { break };
                let n1 = iter.next().unwrap_or(0);
                let n2 = iter.next().unwrap_or(0);
                let n3 = iter.next().unwrap_or(0);
                let n4 = iter.next().unwrap_or(0);
                let n5 = iter.next().unwrap_or(0);
                let n6 = iter.next().unwrap_or(0);
                let n7 = iter.next().unwrap_or(0);

                let num = u64::from_ne_bytes([n0, n1, n2, n3,
                                              n4, n5, n6, n7]);
                let num = num.to_le();
                write!(self.body, "global.get $panic_reason ");
                write!(self.body, "i32.const {} ", count * 4);
                write!(self.body, "i32.add ");
                write!(self.body, "i64.const {} ", num);
                write!(self.body, "i64.store ");

                count += 1;
            }
        }

        self.u32_const(str.len() as u32);
        write!(self.body, "global.set $panic_len ");

        self.call_template("panic");
        self.unreachable();
    }


    pub fn default(&mut self, ty: WasmType) {
        match ty {
            WasmType::I8  => self.i32_const(0),
            WasmType::I16 => self.i32_const(0),
            WasmType::U8  => self.i32_const(0),
            WasmType::U16 => self.i32_const(0),
            WasmType::I32 => self.i32_const(0),
            WasmType::I64 => self.i64_const(0),
            WasmType::F32 => self.f32_const(0.0),
            WasmType::F64 => self.f64_const(0.0),
            WasmType::Ptr { .. } => self.i32_const(0),
        }
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
    pub fn build(&mut self, string_map: &StringMap, buffer: &mut String) {
        self.ret();

        write!(buffer, "(func $_{} ", self.function_id.0);

        if let Some(export) = self.export {
            write!(buffer, "(export \"{}\") ", string_map.get(export));
        } else { write!(buffer, "(export \"{}\")", self.function_id.0) };

        for (i, p) in self.params.iter().enumerate() {
            write!(buffer, "(param $_{i} {}) ", p.param_ty_name());
        }


        if let Some(ret) = self.ret {
            if ret.stack_size() == 0 {
                write!(buffer, "(result {})", ret.param_ty_name());
            }

            write!(buffer, "(local $_ret {})", ret.param_ty_name());

        }

        for (i, l) in self.locals.iter().enumerate() {
            write!(buffer, "(local $_{} {}) ", self.params.len() + i, l.param_ty_name());
        }


        // push & set the val at current sp to the prev sp
        {
            // load the sp
            write!(buffer, "global.get $stack_pointer ");

            // push
            write!(buffer, "i32.const {} ", self.stack_size);
            write!(buffer, "(call $push) ");

            // write the prev sp at the new sp
            write!(buffer, "global.get $stack_pointer ");
            write!(buffer, "call $write_i32_to_mem ");

        }
        
        // write the func_id
        {
            // func_id
            write!(buffer, "i32.const {} ", self.function_id.0);

            // load the sp
            write!(buffer, "global.get $stack_pointer ");
            write!(buffer, "i32.const 4 ");
            write!(buffer, "i32.add ");

            // write
            write!(buffer, "call $write_i32_to_mem ");
        }

        write!(buffer, "(block $_ret ");
        buffer.reserve(self.body.len() + 1);
        for i in self.body.trim_end().chars() {
            buffer.push_char(i)
        }
        write!(buffer, ")");

        buffer.push(&self.finaliser);

        if let Some(ret) = self.ret {
            write!(buffer, "local.get $_ret ");
            if ret.stack_size() != 0 {
                write!(buffer, "local.get {} ", self.params.len() - 1);
                write!(buffer, "i32.const {} ", ret.stack_size());
                write!(buffer, "call $memcpy ");
            }
        }

        write!(buffer, "i32.const {} ", self.stack_size);
        write!(buffer, "(call $pop) ");
        write!(buffer, "return");


        buffer.push_char(')');

   }
}


impl<'a> WasmFunctionBuilder<'a> {
    ///
    /// This function expects:
    /// `ptr($ty)` -> `$ty`
    ///
    #[inline(always)]
    pub fn read(&mut self, ty: WasmType) {
        match ty {
            WasmType::I8  => self.i8_read(),
            WasmType::I16 => self.i16_read(),
            WasmType::U8  => self.u8_read(),
            WasmType::U16 => self.u16_read(),
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
            WasmType::I8  => self.i8_write(),
            WasmType::I16 => self.i16_write(),
            WasmType::U8  => self.u8_write(),
            WasmType::U16 => self.u16_write(),
            WasmType::I32 => self.i32_write(),
            WasmType::I64 => self.i64_write(),
            WasmType::F32 => self.f32_write(),
            WasmType::F64 => self.f64_write(),
            WasmType::Ptr { size } => {
                self.u32_const(size.try_into().unwrap());
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
            WasmType::I8  => self.i8_eq(),
            | WasmType::I16 
            | WasmType::U8
            | WasmType::U16
            | WasmType::I32 => self.i32_eq(),
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
            WasmType::I8  => self.i8_ne(),
            | WasmType::I16 
            | WasmType::U8
            | WasmType::U16
            | WasmType::I32 => self.i32_ne(),
            WasmType::I64 => self.i64_ne(),
            WasmType::F32 => self.f32_ne(),
            WasmType::F64 => self.f64_ne(),
            WasmType::Ptr { size } => self.ptr_veq(size),
        }
    }


    pub fn ite(
        &mut self,
        then_body: impl FnOnce(&mut Self),
        else_body: impl FnOnce(&mut Self),
    ) {
        write!(self.body, "(if (then ");
        then_body(self);
        write!(self.body, ")(else ");
        else_body(self);
        write!(self.body, "))");
    }
   

    #[inline(always)]
    pub fn lite<T, A>(
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


    pub fn assert_eq(
        &mut self,
        ty: WasmType,
        message: &str,
    ) {
        match ty {
            | WasmType::I8
            | WasmType::I16
            | WasmType::U8
            | WasmType::U16
            | WasmType::I32 => self.i32_ne(),
            WasmType::I64 => self.i64_ne(),
            WasmType::F32 => self.f32_ne(),
            WasmType::F64 => self.f64_ne(),
            WasmType::Ptr { .. } => self.i32_ne(),
        }

        self.ite(
        |wasm| {
            wasm.panic(message);
        },
        |_| {}
        );
    }


    pub fn assert_ne(
        &mut self,
        ty: WasmType,
        message: &str,
    ) {
        match ty {
            | WasmType::I8
            | WasmType::I16
            | WasmType::U8
            | WasmType::U16
            | WasmType::I32 => self.i32_eq(),
            WasmType::I64 => self.i64_eq(),
            WasmType::F32 => self.f32_eq(),
            WasmType::F64 => self.f64_eq(),
            WasmType::Ptr { .. } => self.i32_eq(),
        }

        self.ite(
        |wasm| {
            wasm.panic(message);
        },
        |_| {}
        );
    }
}


