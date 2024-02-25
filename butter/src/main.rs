use std::{env, fs, ops::Deref};

use game_runtime::encode;
use margarine::{nodes::{decl::{Declaration, DeclarationNode}, Node}, DropTimer, FileData, SourceRange, StringMap};
use sti::prelude::Arena;
use wasmtime::{Config, Engine};

const GAME_RUNTIME : &[u8] = include_bytes!("../../target/debug/game-runtime");

fn main() -> Result<(), &'static str> {
     DropTimer::with_timer("compilation", || {
        let string_map_arena = Arena::new();
        let mut string_map = StringMap::new(&string_map_arena);
        let files = {
            let mut files = Vec::new();
            for i in env::args().skip(1) {
                files.push(FileData::open(i, &mut string_map).unwrap());
            }

            files
        };

        let mut global = vec![];
        let mut lex_errors = vec![];
        let mut parse_errors = vec![];

        let arena = Arena::new();
        let mut source_offset = 0;
        for f in &files {
            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = margarine::lex(&f, &mut string_map, source_offset);
                tokens
            });

            let (ast, pe) = DropTimer::with_timer("parsing", || {
                let ast = margarine::parse(tokens, &arena, &mut string_map);
                ast
            });

            {
                for n in ast.iter() {
                    if matches!(n, Node::Declaration(_)) { continue }
                    if let Node::Attribute(attr) = n {
                        if matches!(attr.node(), Node::Declaration(_)) { continue }
                    }

                    unreachable!();
                }
            }

            lex_errors.push(le);
            parse_errors.push(pe);

            dbg!(string_map.get(f.name()));
            global.push(DeclarationNode::new(
                Declaration::Module {
                    name: f.name(),
                    body: ast.inner(),
                },

                SourceRange::new(source_offset, source_offset + f.read().len() as u32),
             ).into());

            source_offset += f.read().len() as u32;
         }


         let ns_arena = Arena::new();
         let _scopes = Arena::new();
         let mut sema = {
             let _1 = DropTimer::new("semantic analysis");
             margarine::Analyzer::run(&ns_arena, &mut string_map, &*global)
         };

         println!("{sema:#?}");


         for l in lex_errors {
             if !l.is_empty() {
                 let report = margarine::display(l.as_slice().inner(), &sema.string_map, &files, &());
                 println!("{report}");
             }
         }

         for l in parse_errors {
             if !l.is_empty() {
                 let report = margarine::display(l.as_slice().inner(), &sema.string_map, &files, &());
                 println!("{report}");
             }
         }

         if !sema.errors.is_empty() {
             let report = margarine::display(sema.errors.as_slice().inner(), &sema.string_map, &files, &sema.types);
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
                    let mut values : Vec<&str> = Vec::with_capacity(value.len());
                    for i in value {
                        let str = sema.string_map.get(i.0);
                        if values.contains(&str) { continue }
                        values.push(str)
                    }
                    vec.push((sema.string_map.get(*path), values));
                }
                vec
            };

            let funcs = {
                let mut vec = Vec::with_capacity(sema.funcs.len() + 1);

                let mut funcs = sema.funcs.iter().collect::<Vec<_>>();
                funcs.sort_unstable_by_key(|x| x.wasm_id);
                for f in funcs.iter() {
                    vec.push(sema.string_map.get(f.path));
                }

                vec
            };

            encode(&mut game, &*data, &*imports, &*funcs);
            fs::write("out", &*game).unwrap();
             
         }

         Ok(())
     })?;
 

     Ok(())

}

