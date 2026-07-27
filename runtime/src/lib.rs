pub mod tys;

use core::{alloc::Layout, mem::size_of, ptr::{null, null_mut}, fmt::Write};
use std::{env, io::Write as _};

use common::symbol_id::SymbolId;

#[unsafe(no_mangle)]
pub extern "C" fn margarineAlloc(size: u64) -> *mut u8 {
    unsafe { std::alloc::alloc(Layout::from_size_align(size as _, 8).unwrap()) }
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineDealloc(ptr: *mut u8, size: u64) {
    unsafe { std::alloc::dealloc(ptr, Layout::from_size_align(size as _, 8).unwrap()) }
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineRcAlloc(total_size: u64) -> *mut u8 {
    let ptr = margarineAlloc(total_size);
    unsafe { *(ptr as *mut u64) = 1; }
    ptr
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineRcClone(ptr: *mut u8) -> *mut u8 {
    unsafe {
        let rc = &mut *(ptr as *mut u64);
        *rc += 1;
    }
    ptr
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineRcDrop(ptr: *mut u8, total_size: u64) {
    unsafe {
        let rc = &mut *(ptr as *mut u64);
        *rc -= 1;
        if *rc == 0 {
            margarineDealloc(ptr, total_size);
        }
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineStringFromUtf8(bytes: *const u8, len: u64) -> *mut u8 {
    let buf = margarineRcAlloc(16 + len);
    unsafe {
        *(buf.add(8) as *mut u64) = len;
        core::ptr::copy_nonoverlapping(bytes, buf.add(16), len as usize);
    }
    buf
}


#[unsafe(no_mangle)]
pub extern "C" fn print_int(size: i32) {
    println!("{size}");
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineAbort() -> ! {
    panic_message("margarine abort")
}


#[unsafe(no_mangle)]
pub extern "C" fn margarinePanic(message: Str) -> ! {
    panic_message(message.read())
}


fn panic_message(message: &str) -> ! {
    println!("panic: {message}");
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    std::process::abort();
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineAssertNotNull(ptr: *mut u8) {
    if ptr.is_null() {
        panic_message("null pointer dereference");
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn margarineEnvVariable(value: Str) -> Enum<Str> {
    let name = value.read();

    match std::env::var(name) {
        Ok(v) => Enum { tag: 0, data: Str::new(&v) },
        Err(e) => Enum { tag: 1, data: Str::new(&e.to_string()) },
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn margarineEnvArgs() -> *mut List {
    let args = env::args()
        .map(|arg| Str::new(&arg))
        .collect::<std::vec::Vec<_>>();
    List::from_values(&args)
}


#[unsafe(no_mangle)]
unsafe extern "C" fn print_raw(value: Any) {
    let buf_ptr = unsafe { *(value.inner.add(24) as *const *mut u8) };
    match SymbolId(value.ty as u32) {
        SymbolId::I64 => print!("{}", unsafe { *buf_ptr.cast::<i64>() }),
        SymbolId::F64 => print!("{}", unsafe { *buf_ptr.cast::<f64>() }),
        SymbolId::BOOL => print!("{}", unsafe { *buf_ptr.cast::<Enum<()>>() }.tag != 0),
        SymbolId::STR => {
            let s = unsafe { *buf_ptr.cast::<Str>() };
            print!("{}", s.read());
        }


        _ => todo!(),
    }
    let _ = std::io::stdout().flush();
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
unsafe extern "C" fn io_read_file(path: Str) -> Enum<Str> {
    let str = std::fs::read_to_string(path.read());

    match str {
        Ok(v) => Enum { tag: 0, data: Str::new(&v) },
        Err(e) => Enum { tag: 1, data: Str::new(&e.to_string()) },
    }
}



#[unsafe(no_mangle)]
unsafe extern "C" fn io_read_line() -> Enum<Str> {
    let mut str = String::new();
    let result = std::io::stdin().read_line(&mut str);

    if let Err(e) = result {
        Enum { tag: 1, data: Str::new(&e.to_string()) }
    } else {
        Enum { tag: 0, data: Str::new(&str) }
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
unsafe extern "C" fn str_contains(s: Str, value: Str) -> Enum<()> {
    Enum {
        tag: s.read().contains(value.read()) as u32,
        data: (),
    }
}



#[unsafe(no_mangle)]
unsafe extern "C" fn format(s: Str, values: *const List) -> Str {
    let template = s.read();
    let values = unsafe { &*values };
    let values = unsafe {
        core::slice::from_raw_parts(values.data.cast::<Str>(), values.len as usize)
    };

    let mut result = String::with_capacity(template.len());
    let mut offset = 0;
    let mut value_index = 0;

    while let Some(relative) = template[offset..].find("{}") {
        let start = offset + relative;
        result.push_str(&template[offset..start]);

        if let Some(value) = values.get(value_index) {
            result.push_str(value.read());
            value_index += 1;
        } else {
            result.push_str("{}");
        }

        offset = start + 2;
    }

    result.push_str(&template[offset..]);
    Str::new(&result)
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_nth(s: Str, n: i64) -> Str {
    let ch = s.read().chars().nth(n as usize).unwrap();
    Str::new(&ch.to_string())
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_lines_iter(s: Str) -> *mut Lines {
    alloc(Lines {
        str: s.clone(),
        offset: 0,
    })
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_lines_iter_next(s: *mut Lines) -> Enum<Str> {
    let lines = unsafe { &mut *s };
    if lines.offset >= lines.str.len() as usize {
        return Enum { tag: 1, data: Str { data: null() } }
    }

    let str = lines.str.read();
    let str = &str[lines.offset as usize..];
    let str = str.lines().next();
    lines.offset += str.unwrap_or("").len() + 1;

    if let Some(line) = str {
        Enum { tag: 0, data: Str::new(line) }
    } else {
        Enum { tag: 1, data: Str { data: null() } }
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_split_at(s: Str, idx: i64) -> Pair<Str, Str> {
    let idx = idx as u64;
    if idx >= s.len() {
        panic!("index '{idx}' is out of bounds");
    }

    let (s1, s2) = s.read().split_at(idx as usize);

    Pair { a: Str::new(s1), b: Str::new(s2) }
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
unsafe extern "C" fn str_split_once(s: Str, delimeter: Str) -> Enum<Pair<Str, Str>> {
    let res = s.read().split_once(delimeter.read());

    match res {
        Some((a, b)) => {
            Enum { tag: 0, data: Pair { a: Str::new(a), b: Str::new(b) } }
        },


        None => Enum { tag: 1, data: Pair { a: Str { data: null() }, b: Str { data: null() } } },
    }
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_slice(s: Str, min: i64, max: i64) -> Str {
    Str::new(&s.read()[min as usize..max as usize])
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_concat(a: Str, b: Str) -> Str {
    let a = a.read();
    let b = b.read();
    let mut s = String::with_capacity(a.len() + b.len());
    s.push_str(a);
    s.push_str(b);
    Str::new(&s)
}


#[unsafe(no_mangle)]
unsafe extern "C" fn str_byte_at(s: Str, idx: i64) -> Enum<i64> {
    let str = s.read().as_bytes();
    let idx = idx as usize;
    if str.len() <= idx {
        return Enum { tag: 1, data: 0 }
    }

    let byte = str[idx] as i64;
    Enum { tag: 0, data: byte }
}



#[unsafe(no_mangle)]
unsafe extern "C" fn str_cmp(a: Str, b: Str) -> Enum<()> {
    let result = 
        a.len() == b.len()
        && a.read() == b.read();

    Enum {
        tag: result as u32,
        data: (),
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


// #[unsafe(no_mangle)]
// unsafe extern "C" fn list_push(list: *mut List, elem: Any, elem_size: u64) {
//     let list = unsafe { &mut *list };
// 
// 
//     if list.len == list.cap {
//         let ptr = margarineAlloc(
//             (list.cap as usize * 2 * elem_size as usize) as u64);
// 
//         unsafe {
//         core::ptr::copy(list.data, ptr, list.len as usize * elem_size as usize);
//         }
// 
//         list.cap *= 2;
//         list.cap = list.cap.max(1);
// 
//         list.data = ptr;
//     }
// 
//     let ptr = elem.ptr.cast::<u8>();
//     let buf = unsafe { list.data.add((list.len as u64 * elem_size) as usize) };
// 
//     for i in 0..elem_size as usize {
//         unsafe { *buf.add(i) = *ptr.add(i) };
//     }
// 
//     list.len += 1;
// }
// 
// 
// #[unsafe(no_mangle)]
// unsafe extern "C" fn list_pop(list: *mut List, elem_size: u64) -> Enum {
//     let list = unsafe { &mut *list };
// 
//     if list.len == 0 {
//         return Enum::option_none();
//     }
// 
//     list.len -= 1;
// 
//     let ptr = unsafe { list.data.add((list.len as u64 * elem_size) as usize) };
//     let buf = margarineAlloc(elem_size);
// 
//     for i in 0..elem_size as usize {
//         unsafe { *buf.add(i) = *ptr.add(i) };
//     }
// 
//     Enum {
//         data: buf,
//         tag: 0,
//     }
// }
// 
// 
// #[unsafe(no_mangle)]
// unsafe extern "C" fn list_clear(list: *mut List) {
//     let list = unsafe { &mut *list };
//     list.len = 0;
// }
// 
// 
// #[unsafe(no_mangle)]
// unsafe extern "C" fn list_len(list: *const List) -> i64 {
//     unsafe { *list }.len as i64
// }
// 
// 
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


#[repr(C)]
struct Pair<A, B> {
    a: A,
    b: B,
}



fn alloc<T>(value: T) -> *mut T {
    let ptr = margarineAlloc(size_of::<T>() as _);
    let ptr = ptr.cast::<T>();
    unsafe { *ptr = value; }
    ptr
}


#[repr(C)]
struct Any {
    inner: *mut u8,
    ty: u32,
}


#[repr(C)]
#[derive(Clone, Copy)]
struct Enum<T> {
    tag: u32,
    data: T,
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Str {
    data: *const u8,
}


#[repr(C)]
pub struct List {
    ref_count: u64,
    len: u64,
    cap: u64,
    data: *mut u8,
}

impl List {
    pub fn len(&self) -> u64 {
        self.len
    }

    fn from_values<T: Copy>(values: &[T]) -> *mut Self {
        let len = values.len() as u64;
        let cap = len.max(1);
        let data = margarineAlloc(cap * size_of::<T>() as u64);
        unsafe {
            core::ptr::copy_nonoverlapping(values.as_ptr(), data.cast::<T>(), values.len());
        }

        let list = margarineRcAlloc(size_of::<Self>() as u64).cast::<Self>();
        unsafe {
            list.write(Self { ref_count: 1, len, cap, data });
        }
        list
    }
}


#[unsafe(no_mangle)]
pub extern "C" fn list_len(list: *const List) -> i64 {
    unsafe { (*list).len as i64 }
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
        let buf = margarineRcAlloc(16 + s.len() as u64);
        unsafe {
            *(buf.add(8) as *mut u64) = s.len() as u64;
            let data = buf.add(16);
            let slice = core::slice::from_raw_parts_mut(data, s.len());

            slice.copy_from_slice(s.as_bytes());
        }

        Str { data: buf }
    }


    pub fn len(&self) -> u64 {
        unsafe { *(self.data.add(8) as *const u64) }
    }


    pub fn read(&self) -> &str {
        let len = self.len() as usize;
        unsafe {
        let data = self.data.add(16);
        let slice = core::slice::from_raw_parts(data, len);

        let result = core::str::from_utf8(slice).unwrap();
        result

        }
    }


    pub fn clone(self) -> Str {
        unsafe { Str { data: margarineRcClone(self.data as *mut u8) } }
    }
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
