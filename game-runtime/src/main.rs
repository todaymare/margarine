use std::{env, fs, ptr::null};

use butter_runtime_api::{alloc::{free, walloc}, ffi::{Ctx, SendPtr, WasmPtr}};
use game_runtime::decode;
use libloading::{Library, Symbol};
use wasmtime::{Config, Engine, Module, Linker, Store, Val};

type ExternFunction<'a> = Symbol<'a, ExternFunctionRaw>;
type ExternFunctionRaw = unsafe extern "C" fn(*const Ctx, *mut u8);

static mut CTX_PTR : SendPtr<Ctx> = SendPtr(null());

#[cfg(feature="miri")]
const DATA : &[u8] = include_bytes!("../../out");

fn main() {
    #[cfg(not(feature="miri"))]
    let file = env::current_exe().unwrap();
    #[cfg(feature="miri")]
    let file = "out";
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
    let mut libs = Vec::with_capacity(imports_data.len());

    let mut ctx = Ctx::new();

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
            linker.func_wrap(&*path_noext, i, move |param: i32| {
                let ptr = u32::from_ne_bytes(param.to_ne_bytes());
                let ptr = WasmPtr::from_u32(ptr);
                let ptr = ptr.as_mut(unsafe { &*CTX_PTR.0 });

                unsafe { sym(CTX_PTR.0, ptr); };
            }).unwrap();
        }

        if let Ok(f) = unsafe { library.get::<unsafe extern "C" fn(&Ctx)>(b"_init") } {
            unsafe { f(&ctx) };
        }

        libs.push(library);
    }

    {
        linker.func_wrap("::host", "alloc", |size: u32| {
            let ctx = unsafe { &*CTX_PTR.0 };
            walloc(&mut (ctx.mem(), ctx.store()), size as usize).as_u32()
        }).unwrap();

        linker.func_wrap("::host", "free", |ptr: u32| {
            let ctx = unsafe { &*CTX_PTR.0 };
            free(&(ctx.mem(), ctx.store()), WasmPtr::from_u32(ptr))
        }).unwrap();

        linker.func_wrap("::host", "printi32", |ptr: i32| {
            println!("printi32: {ptr}");
        }).unwrap();
    }

    let mut store = Store::new(&engine, ());
    let instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "memory").unwrap();


    ctx.set_mem(&memory);
    ctx.set_store(&mut store);

    let heap_start = instance.get_global(&mut store, "heap_start").unwrap().get(&mut store);
    let Val::I32(heap_start) = heap_start else { unreachable!() };
    butter_runtime_api::alloc::set_heap_start(WasmPtr::from_u32(u32::from_ne_bytes(heap_start.to_ne_bytes())));

    unsafe { CTX_PTR = SendPtr(&ctx) };
   
    let func = instance.get_func(&mut store, "_init").unwrap();
    func.call(&mut store, &[], &mut []).unwrap();

    for l in libs {
        if let Ok(f) = unsafe { l.get::<unsafe extern "C" fn(&Ctx)>(b"_finalise") } {
            unsafe { f(&*CTX_PTR.0) };
        }
    }
}

