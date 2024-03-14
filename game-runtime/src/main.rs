use std::{env, fs, time::Instant};

use butter_runtime_api::{alloc::{free, walloc}, dump_stack_trace, ffi::Ctx, ptr::{SendPtr, WasmPtr}};
use game_runtime::decode;
use libloading::{Library, Symbol};
use wasmtime::{Config, Engine, Linker, Module, Store, Trap};

type ExternFunction<'a> = Symbol<'a, ExternFunctionRaw>;
type ExternFunctionRaw = unsafe extern "C" fn(*mut Ctx, *mut u8);

static mut CTX_PTR : SendPtr<Ctx> = SendPtr::null();

fn main() {
    let file = env::args().skip(1).next().expect("no file provided");
    let file = fs::read(file).unwrap();
    run(&file);
}


fn run(file: &[u8]) {
    let (data, imports_data, funcs, errs) = {
        decode(&file)
    };

    let mut config = Config::new();
    config.wasm_bulk_memory(true);
    config.strategy(wasmtime::Strategy::Cranelift);
    config.cranelift_opt_level(wasmtime::OptLevel::Speed);
    config.profiler(wasmtime::ProfilingStrategy::PerfMap);
    let engine = Engine::new(&config).unwrap();

    let module = Module::new(&engine, data).unwrap();
    let mut linker = Linker::new(&engine);
    let mut libraries = Vec::new();

    let mut ctx = Ctx::new(funcs);

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
                let ptr = ptr.as_mut(unsafe { CTX_PTR.as_ref() });

                unsafe { sym(CTX_PTR.as_mut(), ptr); };
            }).unwrap();
        }

        if let Ok(f) = unsafe { library.get::<unsafe extern "C" fn(&Ctx)>(b"_init") } {
            unsafe { f(&ctx) };
        }

        libraries.push(library);
    }
    let libraries = libraries;

    // -------------------------------------


    // -------- Load host functions --------
    {
        let func = |size: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            let ptr = walloc(ctx, size as usize);
            let ptr = ptr.as_u32();
            ptr
        };

        linker.func_wrap("::host", "alloc", func).unwrap();
    }

    {
        let func = |ptr: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            free(ctx, WasmPtr::from_u32(ptr));
        };

        linker.func_wrap("::host", "free", func).unwrap();
    }

    {
        let func = || {
            let ctx = unsafe { CTX_PTR.as_mut() };
            dump_stack_trace(ctx);
        };

        linker.func_wrap("::host", "dump_stack_trace", func).unwrap();
    }

    {
        let errs = unsafe { core::mem::transmute::<[Vec<Vec<&str>>; 3], [Vec<Vec<&'static str>>; 3]>(errs) };
        let func = move |ty: u32, file: u32, index: u32| -> wasmtime::Result<()> {
            let ctx = unsafe { CTX_PTR.as_mut() };
            println!("{}", errs[ty as usize][file as usize][index as usize]);
            dump_stack_trace(ctx);

            Err(Trap::UnreachableCodeReached.into())
        };

        linker.func_wrap("::host", "panic", func).unwrap();
    }

    {
        let func = |val: i32| {
            println!("(i32) {val}");
        };

        linker.func_wrap("::host", "printi32", func).unwrap();
    }

    {
        let func = |val: i64| {
            println!("(i64) {val}");
        };

        linker.func_wrap("::host", "printi64", func).unwrap();
    }

    {
        let func = |ptr: u32, len: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            let ptr = WasmPtr::from_u32(ptr).as_ptr(ctx);
            let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
            let str = std::str::from_utf8(slice).unwrap();

            print!("{str}: ");
        };

        linker.func_wrap("::host", "printvar", func).unwrap();
    }

    let mut store = Store::new(&engine, ());
    let mut instance = linker.instantiate(&mut store, &module).unwrap();
    let memory = instance.get_memory(&mut store, "program_memory").unwrap();

    ctx.set_mem(&memory);
    ctx.set_store(&mut store);
    ctx.set_instance(&mut instance);

    let heap_start = instance.get_global(&mut store, "heap_start").unwrap().get(&mut store);
    let wasmtime::Val::I32(heap_start) = heap_start else { unreachable!() };
    butter_runtime_api::alloc::set_heap_start(WasmPtr::from_u32(u32::from_ne_bytes(heap_start.to_ne_bytes())));

    unsafe { CTX_PTR = SendPtr::new(&ctx) };
   
    let func = instance.get_func(&mut store, "::init").unwrap();
    let time = Instant::now();
    
    let result = func.call(&mut store, &[], &mut []);

    if result.is_err() {
        println!("program aborted prematurely")
    }

    println!("took {}ms to complete startup systems", time.elapsed().as_millis());


    for l in libraries {
        if let Ok(f) = unsafe { l.get::<unsafe extern "C" fn(&Ctx)>(b"_finalise") } {
            unsafe { f(CTX_PTR.as_ref()) };
        }
    }
}

/*

fn run(file: &[u8]) {
    let (imports_data, data, funcs) = {
        decode(&file)
    };

    let mut ctx = Ctx::new(funcs);
    let mut store = Store::new();
    let module = store.load_module(data).unwrap();
    let mut imports = Imports::new(); 

    // ------ Load up dynamic imports ------
    let mut libraries = Vec::with_capacity(imports_data.len());

    for (path, items) in &imports_data {
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
            let sym : ExternFunction = unsafe { library.get(i.as_bytes()) }.unwrap();
            let sym = unsafe { sym.into_raw() };

            let func = store.add_func(move |param: i32| {
                let ptr = param as u32;
                let ptr = WasmPtr::from_u32(ptr);
                let ptr = ptr.as_mut(CTX_PTR.as_ref());

                unsafe { sym(CTX_PTR.as_ref(), ptr) };
            });

            imports.add(path_noext, i, func.func().into());
        }

        unsafe { 
            let init_fn = library.get::<unsafe extern "C" fn(&Ctx)>(b"_init");
            if let Ok(f) = init_fn {
                f(&ctx);
            }
        }

        libraries.push(library);
    }
    let libraries = libraries;

    // -------------------------------------


    // -------- Load host functions --------
    {
        let func = |size: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            let ptr = walloc(ctx, size as usize);
            let ptr = ptr.as_u32();
            ptr
        };

        let func = store.add_func(func);
        imports.add("::host", "alloc", func.func().into());
    }

    {
        let func = |ptr: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            free(ctx, WasmPtr::from_u32(ptr));
        };

        let func = store.add_func(func);
        imports.add("::host", "free", func.func().into());
    }

    {
        let func = || {
            let ctx = unsafe { CTX_PTR.as_mut() };
            dump_stack_trace(ctx);
        };

        let func = store.add_func(func);
        imports.add("::host", "dump_stack_trace", func.func().into());
    }

    {
        let func = |val: i32| {
            println!("(i32) {val}");
        };

        let func = store.add_func(func);
        imports.add("::host", "printi32", func.func().into());
    }

    {
        let func = |val: i64| {
            println!("(i64) {val}");
        };

        let func = store.add_func(func);
        imports.add("::host", "printi64", func.func().into());
    }

    {
        let func = |ptr: u32, len: u32| {
            let ctx = unsafe { CTX_PTR.as_mut() };
            let ptr = WasmPtr::from_u32(ptr).as_ptr(ctx);
            let slice = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
            let str = std::str::from_utf8(slice).unwrap();

            print!("{str}: ");
        };

        let func = store.add_func(func);
        imports.add("::host", "printvar", func.func().into());
    }


    let mut instance = store.instantiate_module(module, &imports).unwrap();
    let memory = store.exported_memory(instance, "program_memory").unwrap();

    ctx.set_mem(&memory);
    ctx.set_store(&mut store);
    ctx.set_instance(&mut instance);

    println!("running");
    let func = store.exported_func::<(), ()>(instance, "::init").unwrap();
    store.call(func, ()).unwrap();

    for library in libraries { 
        unsafe { 
            let init_fn = library.get::<unsafe extern "C" fn(&Ctx)>(b"_finalise");
            if let Ok(f) = init_fn {
                f(&ctx);
            }
        }
    }

}

*/
