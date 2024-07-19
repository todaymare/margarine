pub mod tys;

use std::ffi::{c_char, c_int, c_void, CStr, CString};

use tys::{Rc, Str};

// Import from the binary
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MultiFileError {
    errs_count: u32,
    errs: *const *const i8,
}


extern "C" {
    fn __initStartupSystems__();

    static fileCount : u32;

    static lexerErrors  : MultiFileError;
    static parserErrors : MultiFileError;

    static semaErrors : *const i8;
    static semaErrorsLen : u32;
}

#[no_mangle]
pub extern "C" fn main() {
    unsafe { __initStartupSystems__() };
}


// API stuff
#[no_mangle]
pub extern "C" fn print(str: Str) {
    println!("{:?}", str.read())
}


#[no_mangle]
pub extern "C" fn print_cstr(str: *const i8) {
    println!("{}", unsafe { CStr::from_ptr(str).to_string_lossy() });
}


#[no_mangle]
pub extern "C" fn print_i64(str: i64) {
    println!("{}", str)
}


#[no_mangle]
pub extern "C" fn str_to_cstr(str: Str) -> *const i8 {
    let str = str.read();
    let str = CString::new(str).unwrap();
    let str = CString::into_raw(str);
    str
}


#[no_mangle]
pub extern "C" fn free_cstr(str: *mut i8) {
    core::mem::drop(unsafe { CString::from_raw(str) });
}


#[no_mangle]
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


#[derive(Debug)]
pub enum GLFWwindow {}
pub type GLFWglproc = *const c_void;
pub type GLFWframebuffersizefun = extern "C" fn(*mut GLFWwindow, c_int, c_int);

extern "C" {
    pub fn glfwInit() -> c_int;
    pub fn glfwGetCurrentContext() -> *mut GLFWwindow;
    pub fn glfwMakeContextCurrent(window: *mut GLFWwindow);
    pub fn glfwGetProcAddress(procname: *const c_char) -> GLFWglproc;
    pub fn glfwSetFramebufferSizeCallback(
        window: *mut GLFWwindow,
        cbfun: Option<GLFWframebuffersizefun>,
    ) -> Option<GLFWframebuffersizefun>;

}


#[no_mangle]
pub extern "C" fn loadOpenGlToGLFW() {
    unsafe { 
        gl::load_with(|s| {
            let str = CString::new(s).unwrap();
            let result = glfwGetProcAddress(str.as_ptr());
            result
        });
    } 
}


#[no_mangle]
pub extern "C" fn setFrameBufferCallback(window: *mut GLFWwindow, width: i32, height: i32) {
    unsafe { 
        let prev_ctx = glfwGetCurrentContext();
        glfwMakeContextCurrent(window);

        gl::Viewport(0, 0, width, height);
        glfwSetFramebufferSizeCallback(window, Some(framebuffer_size_callback));

        glfwMakeContextCurrent(prev_ctx);
    } 
}


extern "C" fn framebuffer_size_callback(window: *mut GLFWwindow, width: i32, height: i32) {
    unsafe {
        let prev_ctx = glfwGetCurrentContext();
        glfwMakeContextCurrent(window);
        gl::Viewport(0, 0, width, height);
        glfwMakeContextCurrent(prev_ctx);
    }
}


