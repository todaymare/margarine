#![feature(try_trait_v2)]

#[cfg(not(debug_assertions))]
use std::marker::PhantomData;

use std::{collections::HashMap, convert::Infallible, ffi::{CStr, CString}, mem::ManuallyDrop, ops::{Deref, DerefMut, FromResidual, Index, IndexMut}};

//use crate::jit::JIT;

pub mod runtime;
pub mod opcode;
pub mod alloc;
//pub mod jit;


#[derive(Clone, Copy)]
pub struct Reg {
    kind: u64,
    data: RegData,
}


#[derive(Clone, Copy)]
union RegData {
    as_int: i64,
    as_float: f64,
    as_bool: bool,
    as_obj: u64,
    as_unit: (),
}


pub struct Stack {
    pub values: Buffer<Reg>,
    pub bottom: usize,
    curr      : usize,
}


pub struct VM<'src> {
    pub stack: Stack,
    callstack: Callstack<'src>,
    curr: CallFrame<'src>,
    pub funcs: Vec<Function<'src>>,
    error_table: &'src [u8],
    pub objs: Vec<Object>,
    //jit: JIT,
}


#[derive(Debug)]
pub enum Object {
    Struct {
        fields: Vec<Reg>,
    },


    List(Vec<Reg>),


    String(Box<str>),


    Dict(HashMap<Reg, Reg>),


    FuncRef {
        func: u32,
        captures: Vec<Reg>,
    }
}


#[derive(Debug)]
pub struct Function<'src> {
    name: &'src str,
    argc: u8,
    args: &'src [u8],
    ret: u32,
    kind: FunctionKind<'src>,

    cache: Option<HashMap<&'static [Reg], Reg>>,
}


#[derive(Debug)]
pub enum FunctionKind<'src> {
    Code {
        byte_offset: usize,
        byte_size: usize,
    },


    Host(unsafe extern "C" fn(&mut VM<'src>, &mut Reg, &mut Status)),
}


#[repr(C)]
pub struct Status {
    status: u64,
    kind: StatusKind,
}


#[repr(C)]
union StatusKind {
    ok: (),
    err: ManuallyDrop<FatalError>,
}


struct Callstack<'me> {
    stack: Vec<CallFrame<'me>>,
    src: &'me [u8],
}




///
/// A program error that can not be recovered from.
/// This error could be caused by a variety of runtime issues 
///
#[derive(Debug)]
#[repr(C)]
pub struct FatalError {
    pub msg: &'static CStr,
}


///
/// A stack frame
///
struct CallFrame<'me> {
    reader: Reader<'me>,
    
    /// The bottom of the previous frame's stack.
    previous_offset: usize,

    func: u32,

    argc: u8,

}


#[derive(Clone)]
pub struct Reader<'me> {
    /// The source code
    /// Note: This pointer is only valid in the range given at creation
    src: *const u8,

    bounds: &'me [u8],

    #[cfg(not(debug_assertions))]
    _phantom: PhantomData<&'me ()>
}


impl<'src> VM<'src> {
    pub fn new(hosts: HashMap<String, unsafe extern "C" fn(&mut VM<'src>, &mut Reg, &mut Status)>, src: &'src [u8]) -> Result<Self, FatalError> {
        let mut reader = Reader::new(src);
        let Some(b"BUTTERY") = reader.try_next_n().as_ref()
        else { return Err(FatalError::new("invalid header")) };

        let mut objs = vec![];


        // table sizes
        let Some(funcs_size) = reader.try_next_u32()
        else { return Err(FatalError::new("invalid func block size")) };

        let Some(errs_size) = reader.try_next_u32()
        else { return Err(FatalError::new("invalid err block size")) };

        let Some(strs_size) = reader.try_next_u32()
        else { return Err(FatalError::new("invalid str block size")) };

        let Some(errs) = reader.try_next_slice(errs_size as usize)
        else { return Err(FatalError::new("invalid errs block")) };


        let Some(strs) = reader.try_next_slice(strs_size as usize)
        else { return Err(FatalError::new("invalid strs block")) };

        {
            let mut reader = Reader::new(strs);
            let Some(count) = reader.try_next_u32()
            else { return Err(FatalError::new("str count is invalid")) };

            for _ in 0..count {
                let Some(str) = reader.try_next_str()
                else { return Err(FatalError::new("invalid str")) };

                objs.push(Object::String(str.to_string().into_boxed_str()));
            }
        }


        // func metadata
        let Some(funcs_data) = reader.try_next_slice(funcs_size as usize)
        else { return Err(FatalError::new("invalid funcs metadata block")) };

        let mut funcs_reader = Reader::new(funcs_data);

        let mut funcs : Vec<Function> = vec![];

        unsafe {
        loop {
            match funcs_reader.next() {
                opcode::func::consts::Terminate => {
                    break;
                }


                opcode::func::consts::Func => {
                    let name = funcs_reader.next_str();
                    let argc = funcs_reader.next();
                    let ret = funcs_reader.next_u32();
                    let is_cached = funcs_reader.next();

                    let args = funcs_reader.next_slice(argc as usize * 4);

                    let kind = funcs_reader.next();
                    if is_cached != 0 {
                        println!("oh my god its cached");
                    }

                    let cache =
                    if is_cached == 1 { Some(HashMap::new()) }
                    else { None };

                    match kind {
                        0 => {
                            let code = funcs_reader.next_u32();
                            let size = funcs_reader.next_u32();

                            funcs.push(Function {
                                name,
                                kind: FunctionKind::Code {
                                    byte_offset: code as usize,
                                    byte_size: size as usize,
                                },
                                argc,
                                ret,
                                args,
                                cache,
                            });
                        }
                        1 => {
                            let path = funcs_reader.next_str();
                            let Some(host_fn) = hosts.get(path)
                            else {
                                return Err(FatalError::new(&format!("invalid host function: '{path}'")));
                            };

                            funcs.push(Function {
                                name,
                                kind: FunctionKind::Host(*host_fn),
                                argc,
                                ret,
                                args,
                                cache,
                            });
                        }
                        _ => return Err(FatalError::new("invalid func kind"))
                    }

                }

                _ => unreachable!()
            }
        }
        }


        let offset = unsafe { reader.src.offset_from(src.as_ptr()) } as usize;
        let code_section = &src[offset..];

        Ok(Self {
            stack: Stack::new(1024),
            callstack: Callstack::new(256, code_section),
            curr: CallFrame::new(code_section, 0, 0, 0),
            funcs,
            error_table: errs,
            objs,
            //jit: JIT::default(),
        })
    }


    pub fn reset(&mut self) {
        while let Some(_) = self.callstack.pop() {}
        self.stack.curr = 0;
        self.stack.bottom = 0;
    }


    pub fn new_obj(&mut self, obj: Object) -> Reg {
        let id = self.objs.len();
        self.objs.push(obj);
        Reg { kind: Reg::TAG_OBJ, data: RegData { as_obj: id as u64 } }
    }
}


impl<'me> CallFrame<'me> {
    pub fn new(
        code: &'me [u8],
        previous_offset: usize,
        argc: u8,
        func: u32,
    ) -> Self {

        Self {
            reader: Reader::new(code),
            previous_offset,
            argc,
            func,
        }
    }
}


impl<'me> Reader<'me> {
    pub fn new(code: &'me [u8]) -> Self {
        Self {
            src: code.as_ptr(),

            bounds: code,

            #[cfg(not(debug_assertions))]
            _phantom: PhantomData,
        }
    }


    pub fn offset_from_start(&self) -> usize {
        let offset = unsafe { self.src.offset_from(self.bounds.as_ptr()) };
        offset as usize
    }


    ///
    /// Offsets the current cursor by `offset`
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn offset(&mut self, offset: i32) {
        unsafe {
            self.src = self.src.offset(offset as isize);
            debug_assert!(self.bounds.as_ptr_range().contains(&self.src));
        }
    }


    ///
    /// Returns the next value in the source code whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next(&mut self) -> u8 {
        unsafe {
            self.next_n::<1>()[0]
        }
    }


    ///
    /// Reads the next 4 bytes as a `u32` in little endian order whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_u32(&mut self) -> u32 {
        unsafe {
            u32::from_le_bytes(self.next_n::<4>())
        }
    }


    ///
    /// Reads the next 8 bytes as a `u64` in little endian order whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_u64(&mut self) -> u64 {
        unsafe {
            u64::from_le_bytes(self.next_n::<8>())
        }
    }


    ///
    /// Reads the next 8 bytes as a `f64` in little endian order whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_f64(&mut self) -> f64 {
        unsafe {
            f64::from_le_bytes(self.next_n::<8>())
        }
    }


    ///
    /// Reads the next 8 bytes as a `i64` in little endian order whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_i64(&mut self) -> i64 {
        unsafe {
            self.next_u64() as i64
        }
    }



    ///
    /// Reads the next 4 bytes as a `i32` in little endian order whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_i32(&mut self) -> i32 {
        unsafe {
            self.next_u32() as i32
        }
    }


    ///
    /// Reads the next 4 bytes as a `u32` in little endian order and interprets it as the lenght
    /// `len` of the string.
    /// Then, reads the next `len` bytes as a UTF-8 string and returns it.
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds that the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_str(&mut self) -> &'me str {
        unsafe {
            let len = self.next_u32();
            let str = self.next_slice(len as usize);
            core::str::from_utf8(str).unwrap()
        }
    }


    ///
    /// Reads the next 4 bytes as a `u32` in little endian order and interprets it as the lenght
    /// `len` of the string.
    /// Then, reads the next `len` bytes as a UTF-8 string and returns it.
    ///
    pub fn try_next_str(&mut self) -> Option<&'me str> {
        let len = self.try_next_u32()?;
        let str = self.try_next_slice(len as usize)?;
        Some(core::str::from_utf8(str).unwrap())
    }


    ///
    /// Reads the next 4 bytes as a `u32` in little endian order whilst moving the cursor
    ///
    pub fn try_next_u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.try_next_n::<4>()?))
    }


    ///
    /// Returns the next `N` values in the source code whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds of the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_n<const N: usize>(&mut self) -> [u8; N] {
        unsafe {
            debug_assert!(self.bounds.as_ptr_range().contains(&self.src));
            debug_assert!(self.bounds.as_ptr_range().contains(&self.src.add(N-1)));

            let value = *self.src.cast::<[u8; N]>();
            self.src = self.src.add(N);
            value
        }
    }


    ///
    /// Returns the next `n` values in the source code as a slice whilst moving the cursor
    ///
    pub fn try_next_slice(&mut self, n: usize) -> Option<&'me [u8]> {
        unsafe {
            if !self.bounds.as_ptr_range().contains(&self.src)
                || !self.bounds.as_ptr_range().contains(&self.src.add(n-1)) {
                return None
            }

            Some(self.next_slice(n))
        }
    }


    ///
    /// Returns the next `n` values in the source code as a slice whilst moving the cursor
    ///
    /// # Undefined Behaviour
    /// If this function exceeds the initial bounds of the source code it \
    /// will cause undefined behaviour.
    ///
    pub unsafe fn next_slice(&mut self, n: usize) -> &'me [u8] {
        unsafe {
            if n == 0 { return &[] }
            debug_assert!(self.bounds.as_ptr_range().contains(&self.src));
            debug_assert!(self.bounds.as_ptr_range().contains(&self.src.add(n-1)));

            let value = std::slice::from_raw_parts(self.src, n);
            self.src = self.src.add(n);
            value
        }
    }

    ///
    /// Returns the next `N` values in the source code whilst moving the cursor
    ///
    pub fn try_next_n<const N: usize>(&mut self) -> Option<[u8; N]> {
        unsafe {
            if !self.bounds.as_ptr_range().contains(&self.src)
                || !self.bounds.as_ptr_range().contains(&self.src.add(N-1)) {
                return None
            }

            Some(self.next_n())
        }
    }
}


impl Stack {
    pub fn new(cap: usize) -> Self {
        let data = Vec::from_iter((0..cap).map(|_| Reg::new_unit()));
        Self {
            values: Buffer::new(data),
            bottom: 0,
            curr: 0,
        }
    }



    /// 
    /// Returns the value at `bottom + reg`
    ///
    /// # Undefined Behaviour
    /// Accessing a register above the `top` of the stack is undefined behaviour
    ///
    #[inline(always)]
    #[must_use]
    pub unsafe fn reg(&self, reg: u8) -> Reg {
        self.values[reg as usize + self.bottom]
    }


    ///
    /// Sets the value at `bottom + reg` to the given value
    ///
    /// # Undefined Behaviour
    /// Accessing a register above the `top` of the stack is undefined behaviour
    ///
    #[inline(always)]
    pub unsafe fn set_reg(&mut self, reg: u8, data: Reg) {
        /*
        let abs_offset = self.bottom + reg as usize;
        debug_assert!(
            abs_offset < self.top, 
            "failed to set reg '{reg}' relative to offset '{}' to '{data:?}' \
            because it overflows the specified top of the stack top '{}'",
            self.bottom,
            self.top
        );
        */

        self.values[reg as usize + self.bottom] = data;
    }


    ///
    /// Sets the bottom of the stack to the given value.
    ///
    /// # Undefined Behaviour
    /// It is undefined behaviour to set the `amount` to a value which is \
    /// greater than or equal to the `top` of the stack.
    ///
    #[inline(always)]
    unsafe fn set_bottom(&mut self, amount: usize) {
        /*
        debug_assert!(
            amount < self.top,
            "failed to set the bottom of the stack to '{amount}' as it \
            overflows the specified top '{}'", self.top,
        );
        */

        self.bottom = amount;
    }
    

    ///
    /// Increments the `top` of the stack by the given `amount`
    ///
    /// # Returns
    /// - This method will return `Status::Err` with a stack overflow message \
    ///   if the requested `top` of the stack exceeds the capacity of the stack.
    /// - Else, it will return `Status::Ok`
    ///
    #[inline(always)]
    fn push(&mut self, value: Reg) {
        self.values[self.curr] = value;
        self.curr += 1;
    }


    ///
    /// Decrements the `top` of the stack by the given `amount`
    ///
    /// # Undefined Behaviour
    /// If the `top` of the stack is less than or equal to `amount` it will be \
    /// considered undefined behaviour. The `top` of the stack must always be at least 1
    ///
    #[inline(always)]
    unsafe fn pop(&mut self) -> Reg {
        self.curr -= 1;
        let val = self.values[self.curr];
        #[cfg(debug_assertions)]
        {
            self.values[self.curr] = Reg::new_unit();
        }
        val
    }

    ///
    /// Returns the top of the stack
    ///
    /// # Undefined Behaviour
    /// If the `top` of the stack is less than or equal to `amount` it will be \
    /// considered undefined behaviour. The `top` of the stack must always be at least 1
    ///
    #[inline(always)]
    unsafe fn read(&mut self) -> Reg {
        let val = self.values[self.curr-1];
        val
    }
}


#[repr(C)]
pub struct Buffer<T> {
    ptr: *mut T,
    len: usize,
}


impl<T> Buffer<T> {
    pub fn new(vec: Vec<T>) -> Buffer<T> {
        let vec = vec.leak();
        Buffer {
            ptr: vec.as_mut_ptr(),
            len: vec.len(),
        }
    }
}


impl<T> Index<usize> for Buffer<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        debug_assert!(index < self.len);
        unsafe { &*self.ptr.add(index) }
    }
}


impl<T> IndexMut<usize> for Buffer<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        debug_assert!(index < self.len);
        unsafe { &mut *self.ptr.add(index) }
    }
}


impl FatalError {
    pub fn new(msg: &str) -> Self {
        let cstr = ManuallyDrop::new(CString::new(msg).unwrap());
        let cstr = cstr.as_ptr();
        let cstr = unsafe { CStr::from_ptr(cstr) };
        Self { msg: cstr }
    }
}


impl<'me> Callstack<'me> {
    pub fn new(cap: usize, src: &'me [u8]) -> Self {
        Self {
            stack: Vec::with_capacity(cap),
            src,
        }
    }


    pub fn push(&mut self, frame: CallFrame<'me>) {
        self.stack.push(frame);
    }


    pub fn pop(&mut self) -> Option<CallFrame<'me>> {
        self.stack.pop()
    }
}


impl Reg {
    pub unsafe fn as_int(self) -> i64 {
        debug_assert_eq!(self.kind, Self::TAG_INT);
        unsafe { self.data.as_int }
    }


    pub unsafe fn as_bool(self) -> bool {
        debug_assert_eq!(self.kind, Self::TAG_BOOL);
        unsafe { self.data.as_bool }
    }


    pub unsafe fn as_float(self) -> f64 {
        debug_assert_eq!(self.kind, Self::TAG_FLOAT);
        unsafe { self.data.as_float }
    }


    pub unsafe fn as_obj(self) -> u64 {
        debug_assert_eq!(self.kind, Self::TAG_OBJ);
        unsafe { self.data.as_obj }

    }

}


impl Status {
    pub fn err(err: FatalError) -> Self {
        Self {
            status: 1,
            kind: StatusKind { err: ManuallyDrop::new(err) }
        }
    }

    pub fn ok() -> Self {
        Self { status: 0, kind: StatusKind { ok: () } }
    }


    pub fn as_err(&self) -> Option<&CStr> {
        if self.status == 0 {
            return None
        }

        Some(unsafe { self.kind.err.msg })
    }
}


impl FromResidual<std::result::Result<Infallible, FatalError>> for Status {
    fn from_residual(residual: std::result::Result<Infallible, FatalError>) -> Self {
        match residual {
            Ok(_) => Self::ok(),
            Err(e) => Self::err(e),
        }
    }
}


impl<'src> Deref for CallFrame<'src> {
    type Target = Reader<'src>;

    fn deref(&self) -> &Self::Target {
        &self.reader
    }
}

impl<'src> DerefMut for CallFrame<'src> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.reader
    }
}


impl Reg {
    pub const TAG_UNIT  : u64 = 0;
    pub const TAG_INT   : u64 = 1;
    pub const TAG_FLOAT : u64 = 2;
    pub const TAG_BOOL  : u64 = 3;
    pub const TAG_OBJ   : u64 = 4;


    pub fn new_int(data: i64) -> Self { Reg { kind: Self::TAG_INT, data: RegData { as_int: data }} }
    pub fn new_float(data: f64) -> Self { Reg { kind: Self::TAG_FLOAT, data: RegData { as_float: data } } }
    pub fn new_bool(data: bool) -> Self { Reg { kind: Self::TAG_BOOL, data: RegData { as_bool: data } } }
    pub fn new_unit() -> Self { Reg { kind: Self::TAG_UNIT, data: RegData { as_unit: () } } }
    pub fn new_obj(data: u64) -> Self { Reg { kind: Self::TAG_OBJ, data: RegData { as_obj: data } } }
}


impl Object {
    pub fn as_fields(&self) -> &[Reg] {
        match self {
            Object::Struct { fields } => &fields,
            _ => unreachable!(),
        }
    }


    pub fn as_mut_fields(&mut self) -> &mut [Reg] {
        match self {
            Object::Struct { fields } => &mut *fields,
            _ => unreachable!(),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Object::String(str) => &str,
            _ => unreachable!(),
        }
    }

    pub fn as_list(&self) -> &[Reg] {
        match self {
            Object::List(vals) => &vals,
            _ => unreachable!(),
        }
    }

    pub fn as_mut_list(&mut self) -> &mut [Reg] {
        match self {
            Object::List(vals) => vals,
            _ => unreachable!(),
        }
    }

    pub fn as_hm(&mut self) -> &mut HashMap<Reg, Reg> {
        match self {
            Object::Dict(vals) => vals,
            _ => unreachable!(),
        }
    }
}


impl core::fmt::Debug for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut strct = f.debug_struct("Reg");
        match self.kind {
            Self::TAG_INT   => strct.field("tag", &"int"),
            Self::TAG_FLOAT => strct.field("tag", &"float"),
            Self::TAG_BOOL  => strct.field("tag", &"bool"),
            Self::TAG_UNIT  => strct.field("tag", &"unit"),
            Self::TAG_OBJ   => strct.field("tag", &"obj"),
            _ => strct.field("tag", &"unknown".to_string()),
        };

        match self.kind {
            Self::TAG_INT   => strct.field("data", &unsafe { self.data.as_int }),
            Self::TAG_FLOAT => strct.field("data", &unsafe { self.data.as_float }),
            Self::TAG_BOOL  => strct.field("data", &unsafe { self.data.as_bool }),
            Self::TAG_UNIT  => strct.field("data", &unsafe { self.data.as_unit }),
            Self::TAG_OBJ   => strct.field("data", &unsafe { self.data.as_obj }),
            _ => strct.field("data", &"unknown".to_string()),
        };

        strct.finish()
    }
}


impl PartialEq for Reg {
    fn eq(&self, other: &Self) -> bool {
        if self.kind != other.kind { return false };
        unsafe { self.data.as_int == other.data.as_int }
    }
}


impl Eq for Reg {}


impl std::hash::Hash for Reg {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        state.write_u64(self.kind);
        state.write_i64(unsafe { self.data.as_int });
    }
}


impl<T> Deref for Buffer<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        unsafe { std::slice::from_raw_parts(self.ptr, self.len) }
    }
}
