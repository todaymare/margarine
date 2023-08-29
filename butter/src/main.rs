use std::{collections::HashMap, time::Instant};

use margarine::{FileData, SymbolMap, DropTimer};

fn main() -> Result<(), &'static str> {
    DropTimer::with_timer("compilation", || {
        let mut symbol_map = SymbolMap::new();
        let file = DropTimer::with_timer(
            "opening file", 
            || FileData::open("text.txt", &mut symbol_map).unwrap()
        );

        let tokens = DropTimer::with_timer("tokenisation", || {
            let tokens = margarine::lex(&file, &mut symbol_map);
            match tokens {
                Ok(v)  => Ok(v),
                Err(e) => {
                    let report = e.build(&HashMap::from([(file.name(), file.clone())]), &symbol_map);
                    println!("{report}");
                    return Err("failed to compile because of the previous errors")
                },
            }
        })?;


        let mut instructions = DropTimer::with_timer("parsing", || {
            // drop_timer!("parsing");
            let instructions = margarine::parse(tokens, &mut symbol_map);
            match instructions {
                Ok(v)  => Ok(v),
                Err(e) => {
                    let report = e.build(&HashMap::from([(file.name(), file.clone())]), &symbol_map);
                    println!("{report}");
                    return Err("failed to compile because of the previous errors")
                },
            }
        })?;

        let state = {
            let _1 = DropTimer::new("semantic analysis");
            let state = margarine::semantic_analysis(&mut symbol_map, &mut instructions);
            match state {
                Ok(v) => v,
                Err(e) => {
                    let report = e.build(&HashMap::from([(file.name(), file.clone())]), &symbol_map);
                    println!("{report}");
                    return Err("failed to compile because of the previous errors")
                }
            }
        };

        println!("{:?}", symbol_map.arena_stats());
    
        Ok(())
    })
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

