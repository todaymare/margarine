pub mod tys;

use core::{alloc::Layout, ffi::CStr, ptr::null, fmt::Write};
use std::marker::PhantomData;

use semantic_analysis::syms::sym_map::SymbolId;

// Import from the binary
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MultiFileError {
    errs_count: u32,
    errs: *const *const i8,
}


unsafe extern "C" {
    static fileCount : u32;

    static lexerErrors  : MultiFileError;
    static parserErrors : MultiFileError;

    static semaErrors : *const i8;
    static semaErrorsLen : u32;
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineAlloc(size: u64) -> *mut u8 {
    //println!("malloc {size}");
    unsafe { std::alloc::alloc(Layout::from_size_align(size as _, 8).unwrap()) }
}


#[unsafe(no_mangle)]
pub extern "C" fn print_int(size: i32) {
    println!("{size}");
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineAbort() -> ! {
    println!("margarine abort");
    std::process::abort();
}


#[unsafe(no_mangle)]
unsafe extern "C" fn print_raw(value: Any) {
    match SymbolId(value.ty as u32) {
        SymbolId::I64 => print!("{}", unsafe { *value.ptr.cast::<i64>() }),
        SymbolId::F64 => print!("{}", unsafe { *value.ptr.cast::<f64>() }),
        SymbolId::BOOL => print!("{}", unsafe { *value.ptr.cast::<Enum>() }.tag != 0),
        SymbolId::STR => {
            let s = unsafe { *value.ptr.cast::<Str>() };
            print!("{}", s.read());
        }


        _ => todo!(),
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn int_to_str(value: i64) -> Str {
    let mut buf = itoa::Buffer::new();
    let str = buf.format(value);
    Str::new(str)
}


#[unsafe(no_mangle)]
unsafe extern "C" fn float_to_str(value: f64) -> Str {
    let mut buf = ryu::Buffer::new();
    let str = buf.format(value);
    Str::new(str)
}


#[unsafe(no_mangle)]
unsafe extern "C" fn io_read_file(path: Str) -> Enum {
    let str = std::fs::read_to_string(path.read());

    match str {
        Ok(v) => Enum::result_ok(Str::new(&v)),
        Err(e) => Enum::result_err(Str::new(&e.to_string())),
    }
}



#[unsafe(no_mangle)]
unsafe extern "C" fn io_read_line() -> Enum {
    let mut str = String::new();
    let result = std::io::stdin().read_line(&mut str);

    if let Err(e) = result {
        Enum::result_err(Str::new(&e.to_string()))
    } else {
        Enum::result_ok(Str::new(&str))
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn now_secs() -> i64 {
    let Ok(time) = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
    else { panic!("failed to get the epoch") };

    let secs = time.as_secs();
    secs as i64
}


#[unsafe(no_mangle)]
unsafe extern "C" fn now_nanos() -> i64 {
    let Ok(time) = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
    else { panic!("failed to get the epoch") };

    let secs = time.subsec_nanos();
    secs as i64
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_len(s: Str) -> i64 {
    s.len() as i64
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_nth(s: Str, n: i64) -> Str {
    let ch = s.read().chars().nth(n as usize).unwrap();
    Str::new(&ch.to_string())
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_lines_iter(s: Str) -> *mut Lines {
    alloc(Lines {
        str: s,
        offset: 0,
    })
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_lines_iter_next(s: *mut Lines) -> Enum {
    let lines = unsafe { &mut *s };
    if lines.offset >= lines.str.len() as usize {
        return Enum::option_none()
    }

    let str = lines.str.read();
    let str = &str[lines.offset as usize..];
    let str = str.lines().next();
    lines.offset += str.unwrap_or("").len() + 1;

    if let Some(line) = str {
        Enum::option_some(Str::new(line))
    } else {
        Enum::option_none()
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_split_at(s: Str, idx: i64) -> Tuple2<Str, Str>{
    let idx = idx as u64;
    if idx as u32 >= s.len() as u32 {
        panic!("index '{idx}' is out of bounds");
    }

    let (s1, s2) = s.read().split_at(idx as usize);

    Tuple2::new(Str::new(s1), Str::new(s2))
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_hash(s: Str, hasher: *const ()) {
    let func_ref = unsafe { *hasher.cast::<FuncRef>() };
    let func = unsafe {
        core::mem::transmute::<_, extern "C" fn(*const (), i64, *const u8)>(func_ref.ptr)
    };

    let bytes = s.read().as_bytes();
    for i in 0..=(bytes.len() / 8) {
        let i = i * 8;
        let b = i64::from_ne_bytes([
            bytes.get(i).copied().unwrap_or(0),
            bytes.get(i+1).copied().unwrap_or(0),
            bytes.get(i+2).copied().unwrap_or(0),
            bytes.get(i+3).copied().unwrap_or(0),
            bytes.get(i+4).copied().unwrap_or(0),
            bytes.get(i+5).copied().unwrap_or(0),
            bytes.get(i+6).copied().unwrap_or(0),
            bytes.get(i+7).copied().unwrap_or(0),
        ]);

        func(hasher, b, func_ref.captures);
    };

}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_parse(s: Str, ty: i64) -> Enum {
    let s = s.read().trim();
    match SymbolId(ty as u32) {
       SymbolId::I64 => {
            let Ok(data) = s.parse::<i64>()
            else { return Enum::option_none() };

            Enum::option_some(Any::new(data, SymbolId::I64.0))
        }

        SymbolId::F64 => {
            let Ok(data) = s.parse::<f64>()
            else { return Enum::option_none() };

            Enum::option_some(Any::new(data, SymbolId::F64.0))
        }

        _ => {
            Enum::option_none()
        },
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_split_once(s: Str, delimeter: Str) -> Enum {
    let res = s.read().split_once(delimeter.read());

    match res {
        Some((a, b)) => {
            Enum::option_some(Tuple2::new(Str::new(a), Str::new(b)))
        },


        None => Enum::option_none(),
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_slice(s: Str, min: i64, max: i64) -> Str {
    let sliced = s.read()
        .chars()
        .skip(min as usize)
        .take((max - min) as usize)
        .collect::<String>();

    Str::new(&sliced)
}



#[unsafe(no_mangle)]
unsafe extern "C" fn str_cmp(a: Str, b: Str) -> Enum {
    let result = 
        a.len() == b.len()
        && a.read() == b.read();

    Enum {
        tag: result as u32,
        data: null(),
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn random_int() -> i64 {
    rand::random()
}


#[unsafe(no_mangle)]
unsafe extern "C" fn random_float() -> f64 {
    rand::random()
}


#[unsafe(no_mangle)]
unsafe extern "C" fn list_push(list: *mut List, elem: Any, elem_size: u64) {
    let list = unsafe { &mut *list };


    if list.len == list.cap {
        let ptr = margarineAlloc(
            (list.cap as usize * 2 * elem_size as usize) as u64);

        unsafe {
        core::ptr::copy(list.data, ptr, list.len as usize * elem_size as usize);
        }

        list.cap *= 2;
        list.cap = list.cap.max(1);

        list.data = ptr;
    }

    /*
    println!(
        "PUSH: 
        list.buf_ptr: {:?}, 
        list.len: {:?}, 
        list.cap: {:?}, 
        elem.ptr={:?}, 
        first8={:016x}, 
        elem_size: {:x}",
        list.data,
        list.len,
        list.cap,
        elem.ptr,
        unsafe { *(elem.ptr as *const u64) },
        elem_size,
    );
    */


    let ptr = elem.ptr.cast::<u8>();
    let buf = unsafe { list.data.add((list.len as u64 * elem_size) as usize) };

    for i in 0..elem_size as usize {
        unsafe { *buf.add(i) = *ptr.add(i) };
    }

    list.len += 1;
}


#[unsafe(no_mangle)]
unsafe extern "C" fn list_pop(list: *mut List, elem_size: u64) -> Enum {
    let list = unsafe { &mut *list };

    if list.len == 0 {
        return Enum::option_none();
    }

    list.len -= 1;

    let ptr = unsafe { list.data.add((list.len as u64 * elem_size) as usize) };
    let buf = margarineAlloc(elem_size);

    for i in 0..elem_size as usize {
        unsafe { *buf.add(i) = *ptr.add(i) };
    }

    Enum {
        data: buf,
        tag: 0,
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn list_clear(list: *mut List) {
    let list = unsafe { &mut *list };
    list.len = 0;
}


#[unsafe(no_mangle)]
unsafe extern "C" fn list_len(list: *const List) -> i64 {
    unsafe { *list }.len as i64
}


#[unsafe(no_mangle)]
unsafe extern "C" fn test(list: FuncRef) {
    unsafe {
        let func = core::mem::transmute::<_, unsafe extern "C" fn(*const u8)>(list.ptr);
        func(list.captures)
    }

}


#[derive(Clone, Copy, Debug)]
#[repr(C)]
struct FuncRef {
    ptr: unsafe extern "C" fn(),
    captures: *const u8,
}


#[derive(Clone, Copy)]
struct Lines {
    str: Str,
    offset: usize,
}


#[derive(Clone, Copy)]
#[repr(C)]
struct Tuple2<A, B> {
    data: *mut InnerTuple2<A, B>,
    _marker: PhantomData<(A, B)>,
}


#[derive(Clone, Copy)]
#[repr(C)]
struct InnerTuple2<A, B> {
    a: A,
    b: B,
}

impl<A, B> Tuple2<A, B> {
    fn new(a: A, b: B) -> Self {
        let data = InnerTuple2 { a, b };
        let data = alloc(data);
        Self {
            data,
            _marker: PhantomData,
        }
    }
}



fn alloc<T>(value: T) -> *mut T {
    let ptr = margarineAlloc(size_of::<T>() as _);
    let ptr = ptr.cast::<T>();
    unsafe { *ptr = value; }
    ptr
}


#[derive(Clone, Copy)]
#[repr(C)]
struct Any {
    ptr: *mut (),
    ty: u32,
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Enum {
    data: *const u8,
    tag: u32,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Str {
    data: *const u8,
}


#[derive(Clone, Copy)]
#[repr(C)]
struct List {
    len: u32,
    cap: u32,
    data: *mut u8,
}


macro_rules! test {
    ($($e: expr),* ; $($f: expr),*) => {
        $(
            println!("arg: {}", $e);
        )*
        $(
            println!("field: {}", $f);
        )*
    };
}


impl Str {
    pub fn new(s: &str) -> Str {
        let buf = margarineAlloc((4 + s.len()) as u64);
        unsafe {
            *buf.cast::<u32>() = s.len() as u32;
            let data = buf.cast::<u32>().add(1).cast::<u8>();
            let slice = core::slice::from_raw_parts_mut(data, s.len());

            slice.copy_from_slice(s.as_bytes());
        }

        Str { data: buf }
    }


    pub fn len(&self) -> u32 {
        unsafe { *self.data.cast::<u32>() }
    }


    pub fn read(&self) -> &str {
        let len = self.len();
        unsafe {
        let data = self.data.cast::<u32>().add(1).cast::<u8>();
        let slice = core::slice::from_raw_parts(data, len as usize);

        let result = core::str::from_utf8(slice).unwrap();
        result

        }
    }
}


impl Enum {
    pub fn option_some<T>(value: T) -> Enum {
        let ptr = alloc(value);
        Enum {
            tag: 0,
            data: ptr.cast(),
        }
    }


    pub fn option_none() -> Enum {
        Enum {
            tag: 1,
            data: null(),
        }
    }


    pub fn result_ok<T>(value: T) -> Enum {
        let ptr = alloc(value);
        Enum {
            tag: 0,
            data: ptr.cast(),
        }
    }


    pub fn result_err<T>(value: T) -> Enum {
        let ptr = alloc(value);
        Enum {
            tag: 1,
            data: ptr.cast(),
        }
    }

}



impl Any {
    pub fn new<T>(data: T, ty: u32) -> Any {
        Any {
            ptr: alloc(data).cast(),
            ty,
        }
    }
}



#[unsafe(no_mangle)]
pub extern "C" fn margarineError(error_type: u32, error_file: u32, error_id: u32) -> ! {
    println!("aborting processes: encountered a compiler error");

    match error_type {
        0 => {
            assert!(error_file < unsafe { fileCount });

            let file = unsafe { *(((&lexerErrors) as *const MultiFileError).add(error_file as usize)) };
            assert!(error_id < file.errs_count);

            let err = unsafe { *file.errs.add(error_id as usize) };
            let cstr = unsafe { CStr::from_ptr(err) };
            println!("{}", cstr.to_str().unwrap());
        }

        1 => {
            assert!(error_file < unsafe { fileCount });

            let file = unsafe { *(((&parserErrors) as *const MultiFileError).add(error_file as usize)) };
            assert!(error_id < file.errs_count);

            let err = unsafe { *file.errs.add(error_id as usize) };
            let cstr = unsafe { CStr::from_ptr(err) };
            println!("{}", cstr.to_str().unwrap());
        },

        2 => {
            assert!(error_id < unsafe { semaErrorsLen });

            let err = unsafe { *((&semaErrors) as *const *const i8).add(error_id as usize) };
            let cstr = unsafe { CStr::from_ptr(err) };
            println!("{}", cstr.to_str().unwrap());
        }
        _ => println!("invalid error type id"),
    }

    std::process::abort();
}



/*

// API stuff
#[no_mangle]
pub extern "C" fn print(str: Str) {
    println!("{}", str.read())
}

#[no_mangle]
pub extern "C" fn print_cstr(str: Rc<*const i8>) {
    println!("{}", unsafe { CStr::from_ptr(str.read()).to_string_lossy() });
}


#[no_mangle]
pub extern "C" fn print_i64(str: i64) {
    println!("{}", str)
}


#[no_mangle]
pub extern "C" fn str_to_cstr(str: Str) -> Rc<*const i8> {
    let str = str.read();
    let str = CString::new(str).unwrap().into_boxed_c_str();
    let str = Box::leak(str);
    let str = str.as_ptr();
    let rc = Rc::new(str);

    rc
}


*/
