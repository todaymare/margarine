use margarine::{FileData, StringMap, DropTimer, Extension};
use sti::prelude::Arena;

 fn main() -> Result<(), &'static str> {
     DropTimer::with_timer("compilation", || {
         let string_map_arena = Arena::new();
         let mut string_map = StringMap::new(&string_map_arena);
         let file = [
             FileData::new(
                 include_str!("../../text0.txt").to_string(), 
                 string_map.insert("test"), 
                 Extension::Mar
             )
         ];

         let (tokens, lex_errors) = DropTimer::with_timer("tokenisation", || {
             let tokens = margarine::lex(&file[0], &mut string_map);
             tokens
         });

         println!("{tokens:#?}");

         let mut arena = Arena::new();
         let (ast, parse_errors) = DropTimer::with_timer("parsing", || {
             let ast = margarine::parse(tokens, &mut arena, &mut string_map);
             ast
         });

         println!("{ast:#?}");

         let ns_arena = Arena::new();
         let _scopes = Arena::new();
         let sema = {
             let _1 = DropTimer::new("semantic analysis");
             margarine::Analyzer::run(&ns_arena, &mut string_map, &ast)
        };

         println!("{sema:#?}");

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
         

         let code = sema.module_builder.build(&mut string_map);
         std::fs::write("out.wat", &*code).unwrap();

         // let typed_ast = match typed_ast {
         //     Ok(v)  => v,
         //     Err(e) => {
         //         let report = e.display(&symbol_map, &file);
         //         println!("{report}");
         //         return Err("failed to compile because of the previous errors")
         //     },
         // };

         // dbg!(&typed_ast);


         // println!("scopes {:?}", scopes.stats());
         // drop(scopes);

         // println!("typed ast arena {:?}", ns_arena.stats());
        

         println!("symbol map arena {:?}", string_map.arena_stats());

         Ok(())
     })?;
 

     Ok(())
 }


// fn main() {
//     use bytecode_consts::*;
//     let mut vm = VM::new(
//         Stack::with_capacity(128),
//         CompilerMetadata {
//             num_of_functions: 1,
//             num_of_structs: 1,
//         },
//         vec![
//             Data::new_float(1.0),
//             Data::new_float(2.0),
//             Data::new_float(5.0),

//         ].into_boxed_slice()
//     );
//     let slice = [
//         Func,
//             // Function Metadata
//                 // Name: 3 characters, "fib"
//                 3, 0, 0, 0,
//                 b'f', b'i', b'b',


//                 // Is NOT System
//                 0,

//                 // Size (50 bytes)
//                 59, 0, 0, 0,

//                 // Return type (Type Id 2 == float)
//                 2, 0, 0, 0,

//                 // Args len 1
//                 1, 0,

//                 // Arguments
//                     // Arg1
//                         // Name: 1 characters, "n"
//                         1, 0, 0, 0,
//                         b'n',

//                         // Is NOT Inout
//                         0,

//                         // Type Id 2 == float
//                         2, 0, 0, 0,
    

//             // Function Body
//                 Push, 100,
//                 // n <= 1
//                 Lit, 2, 0, 0, 0, 0,
//                 LeF, 2, 1, 2,
//                 Jif, 2, 14, 20,

//                 Copy, 0, 1,
//                 Pop, 100,
//                 Ret,

//                 // fib(n-1)
//                 Lit, 2, 0, 0, 0, 0,
//                 SubF, 2, 1, 2,
//                 Call, 3, 0, 1, 2,
                
//                 // fib(n-2)
//                 Lit, 2, 1, 0, 0, 0,
//                 SubF, 2, 1, 2,
//                 Call, 4, 0, 1, 2,

//                 // fib(n-1) + fib(n-2)
//                 AddF, 0, 3, 4,
                
//                 Pop, 100,
//                 Ret,

//         // Main Fn
//         Func,
//             // Name: main, 4 chars
//             4, 0, 0, 0,
//             b'm', b'a', b'i', b'n',

//             // Not system
//             0,

//             // Size 9
//             12, 0, 0, 0,


//             // Ret unit
//             0, 0, 0, 0,

//             // no args
//             0, 0,

//             // Body
//             Lit, 0, 2, 0, 0, 0,
//             Call, 0, 0, 1, 0,
//             Ret,

//         Ret,
//     ];

//     vm.register_declarations(Code::new(slice.as_ptr(), slice.last().unwrap() as *const u8));
//     let id = vm.query_function("main").unwrap();
//     let block = vm.get_function_block(id);

//     vm.stack.push(1);
//     vm.run_bytecode(block);

//     dbg!(vm);
// }

/*
fn main() {
    let arena = Arena::new();
    let mut string_map = StringMap::new(&arena);
    
    let func_arena = Arena::new();
    let mut module_builder = WasmModuleBuilder::new();
    let func1 = module_builder.function_id();
    let mut builder = WasmFunctionBuilder::new(&func_arena, func1);
    builder.do_loop(
        |f, id| {
            f.i32_const(69);
            f.ite(
                |f| { f.i32_const(1); f.pop(); f.continue_loop(id) },
                |_| {},
            );
        }
    );
    module_builder.register(builder);
   

    println!("{}", String::from_utf8(module_builder.build(&mut string_map).to_vec()).unwrap());
}

*/

