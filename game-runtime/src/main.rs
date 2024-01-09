use std::{env, fs, error::Error, sync::OnceLock, ptr::null, ops::Add};

use game_runtime::decode;
use libloading::{Library, Symbol};
use wasmtime::{Config, Engine, Module, Linker, Store, Val};

type ExternFunction<'a> = Symbol<'a, ExternFunctionRaw>;
type ExternFunctionRaw = unsafe extern "C" fn(*mut u8);

static mut PTR : *const u8 = null();

fn main() {
    let file = env::current_exe().unwrap();
    let file = fs::read(file).unwrap();
    let (imports_data, data) = {
        decode(&file)
    };

    let mut config = Config::new();
    config.strategy(wasmtime::Strategy::Cranelift);
    config.wasm_bulk_memory(true);
    let engine = Engine::new(&config).unwrap();

    let module = unsafe { Module::deserialize(&engine, data) }.unwrap();
    let mut linker = Linker::new(&engine);

    for (path, items) in imports_data {
        let path_noext = path;

        #[cfg(target_os = "windows")]
        let path = format!("{path}.dll");

        #[cfg(target_os = "linux")]
        let path = format!("{path}.so");

        #[cfg(target_os = "macos")]
        let path = format!("{path}.dylib");

        #[cfg(not(any(
            target_os = "windows",
            target_os = "linux",
            target_os = "macos",
        )))]
        compile_error!("this platform is not supported");


        let library = unsafe { Library::new(&*path) }.unwrap();
        for i in items {
            let sym = unsafe { library.get::<ExternFunction<'_>>(i.as_bytes()) }.unwrap();
            let sym = unsafe { sym.into_raw() };
            println!("{path_noext}::{i}");
            linker.func_wrap(&*path_noext, i, move |param: i32| {
                let ptr = unsafe { PTR.add(param as usize).cast_mut() };
                unsafe { sym(ptr); };
            }).unwrap();
        }

        std::mem::forget(library);
    }

    let mut store = Store::new(&engine, 4);
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();

    let ptr = memory.data_ptr(&store);
    unsafe { PTR = ptr };
    
    let func = instance.get_func(&mut store, "main").unwrap();
    let mut slice = [Val::null()];
    func.call(&mut store, &[], &mut slice).unwrap();

    println!("{slice:?}");
}


