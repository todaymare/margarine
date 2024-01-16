use std::{env, fs::{self}};

use game_runtime::encode;
use margarine::{FileData, StringMap, DropTimer};
use sti::prelude::Arena;
use wasmtime::{Config, Engine};

const GAME_RUNTIME : &[u8] = include_bytes!("../../target/release/game-runtime");

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

         println!("{ast:#?}");

         let ns_arena = Arena::new();
         let _scopes = Arena::new();
         let mut sema = {
             let _1 = DropTimer::new("semantic analysis");
             margarine::Analyzer::run(&ns_arena, &mut string_map, &ast)
         };

         // println!("{sema:#?}");

         dbg!(&sema);

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
         

         let code = sema.module_builder.build(&mut sema.string_map);

         /*
         println!("symbol map arena {:?} ns_arena: {ns_arena:?}, arena: {arena:?}", string_map.arena_stats());
         println!("{:?}", &*ArenaPool::tls_get_temp());
         println!("{:?}", &*ArenaPool::tls_get_rec());
         */
         
         fs::write("out.wat", &*code).unwrap();
         // Run
         {
            let data = &*code;
            let mut game = GAME_RUNTIME.to_vec();

            let mut config = Config::new();
            config.strategy(wasmtime::Strategy::Cranelift);
            config.wasm_bulk_memory(true);
            let engine = Engine::new(&config).unwrap();
            let data = engine.precompile_module(&data).unwrap();
                   
            let imports = {
                let mut vec = Vec::with_capacity(sema.module_builder.externs.len());
                for (path, value) in sema.module_builder.externs.iter() {
                    let mut values = Vec::with_capacity(value.len());
                    for i in value { values.push(sema.string_map.get(i.0)) }
                    vec.push((sema.string_map.get(*path), values));
                }
                vec
            };

            encode(&mut game, &*data, &*imports);
            fs::write("out", &*game).unwrap();
             
         }

         Ok(())
     })?;
 

     Ok(())

}

