use std::{env, fs};

use colourful::ColourBrush;
use margarine::{FileData, StringMap, DropTimer, Extension};
use sti::{prelude::Arena, arena_pool::ArenaPool};
use wasmer::{Store, Module, imports, Instance, RuntimeError};
use wasmer_compiler_cranelift::Cranelift;

 fn main() -> Result<(), &'static str> {
     DropTimer::with_timer("compilation", || {
         let string_map_arena = Arena::new();
         let mut string_map = StringMap::new(&string_map_arena);
         let file = [
             FileData::open(env::args().nth(1).expect("expected a file"), &mut string_map).unwrap()
         ];

         let (tokens, lex_errors) = DropTimer::with_timer("tokenisation", || {
             let tokens = margarine::lex(&file[0], &mut string_map);
             tokens
         });

         // println!("{tokens:#?}");

         let mut arena = Arena::new();
         let (ast, parse_errors) = DropTimer::with_timer("parsing", || {
             let ast = margarine::parse(tokens, &mut arena, &mut string_map);
             ast
         });

         // println!("{ast:#?}");

         let ns_arena = Arena::new();
         let _scopes = Arena::new();
         let sema = {
             let _1 = DropTimer::new("semantic analysis");
             margarine::Analyzer::run(&ns_arena, &mut string_map, &ast)
         };

         // println!("{sema:#?}");


         if !lex_errors.is_empty() {
             let report = margarine::display(lex_errors.as_slice().inner(), &sema.string_map, &file, &());
             println!("{report}");
         }

         if !parse_errors.is_empty() {
             let report = margarine::display(parse_errors.as_slice().inner(), &sema.string_map, &file, &());
             println!("{report}");
         }

         if !sema.errors.is_empty() {
             let report = margarine::display(sema.errors.as_slice().inner(), &sema.string_map, &file, &sema.types);
             println!("{report}");
         }
         

         dbg!(&sema);
         let code = sema.module_builder.build(&mut string_map);

         /*
         println!("symbol map arena {:?} ns_arena: {ns_arena:?}, arena: {arena:?}", string_map.arena_stats());
         println!("{:?}", &*ArenaPool::tls_get_temp());
         println!("{:?}", &*ArenaPool::tls_get_rec());
         */

         {
             fs::write("out.wat", &*code).unwrap();
         }

         // Run
         {
             let cranelift = Cranelift::new();
             let mut store = Store::new(cranelift);
             let module = Module::new(&store, &*code).unwrap();

             let imports_object = imports! {};

             let instance = Instance::new(&mut store, &module, &imports_object).unwrap();
             let func = instance.exports.get_function("main").unwrap();
             let result = DropTimer::with_timer("wasm runtime", || {
                 func.call(&mut store, &[])
             });

             match result {
                Ok(v) => println!("result is {v:?}"),
                Err(v) => {
                    println!("{}: {}", "critical runtime error".red().bold(), print_wasm_error(v).white().bold());
                }
            }
         }

         Ok(())
     })?;
 

     Ok(())

}


fn print_wasm_error(e: RuntimeError) -> String {
    use std::fmt::Write;

    let mut string = String::new();

    if let Some(val) = e.clone().to_trap() {
        let _ = writeln!(string, "{}", val.message().to_string());
    }

    for (i, f) in e.trace().iter().enumerate() {
        let _ = writeln!(string, "{i} - {}", f.function_name().unwrap_or("<unknown>"));
    }

    string
}
