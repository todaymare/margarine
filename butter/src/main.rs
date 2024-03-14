use std::{env, fs, process::Command};

use game_runtime::encode;
use margarine::{nodes::{decl::{Declaration, DeclarationNode}, Node}, DropTimer, FileData, SourceRange, StringMap};
use sti::prelude::Arena;

// const GAME_RUNTIME : &[u8] = include_bytes!("../../target/debug/game-runtime");

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
        for (i, f) in files.iter().enumerate() {
            let (tokens, le) = DropTimer::with_timer("tokenisation", || {
                let tokens = margarine::lex(&f, &mut string_map, source_offset);
                tokens
            });

            let (ast, pe) = DropTimer::with_timer("parsing", || {
                let ast = margarine::parse(tokens, i.try_into().unwrap(), &arena, &mut string_map);
                ast
            });

            {
                for n in ast.iter() {
                    if matches!(n, Node::Declaration(_)) { continue }
                    if let Node::Attribute(attr) = n {
                        if matches!(attr.node(), Node::Declaration(_)) { continue }
                    }

                    ()
                }
            }

            lex_errors.push(le);
            parse_errors.push(pe);

            global.push(DeclarationNode::new(
                Declaration::Module {
                   name: f.name(),
                   body: ast.inner(),
               },

               SourceRange::new(source_offset, source_offset + f.read().len() as u32),
            ).into());

           source_offset += f.read().len() as u32;
        }

        assert_eq!(lex_errors.len(), files.len());
        assert_eq!(parse_errors.len(), files.len());


        let ns_arena = Arena::new();
        let _scopes = Arena::new();
        let mut sema = {
            let _1 = DropTimer::new("semantic analysis");
            margarine::Analyzer::run(&ns_arena, &mut string_map, &*global)
        };

        println!("{sema:#?}");


        // todo: find a way to comrpess these errors into vecs
        let mut lex_error_files = Vec::with_capacity(lex_errors.len());
        for l in lex_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = margarine::display(e.1, &sema.string_map, &files, &());
                println!("{report}");
                file.push(report);
            }

            lex_error_files.push(file);
        }

        let mut parse_error_files = Vec::with_capacity(parse_errors.len());
        for l in parse_errors {
            let mut file = Vec::with_capacity(l.len());
            for e in l.iter() {
                let report = margarine::display(e.1, &sema.string_map, &files, &());
                println!("{report}");
                file.push(report);
            }

            parse_error_files.push(file);
        }

        let mut sema_errors = Vec::with_capacity(sema.errors.len());
        for s in sema.errors.iter() {
            let report = margarine::display(s.1, &sema.string_map, &files, &sema.types);
            println!("{report}");
            sema_errors.push(report);
         } 

        let code = sema.module_builder.build(&mut sema.string_map);

        /*
        println!("symbol map arena {:?} ns_arena: {ns_arena:?}, arena: {arena:?}", string_map.arena_stats());
        println!("{:?}", &*ArenaPool::tls_get_temp());
        println!("{:?}", &*ArenaPool::tls_get_rec());
        */

        // ------ Prepare data for transmission ------
        let data_imports = {
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

        let data_funcs = {
            let mut vec = Vec::with_capacity(sema.funcs.len() + 1);

            let mut funcs = sema.funcs.iter().collect::<Vec<_>>();
            funcs.sort_unstable_by_key(|x| x.wasm_id);
            for f in funcs.iter() {
                vec.push(sema.string_map.get(f.path));
            }

            vec
        };

        // ------------------------------------------
         
        fs::write("out.wat", &*code).unwrap();
        assert!(Command::new("wat2wasm")
            .arg("out.wat")
            .arg("-o")
            .arg("out.wasm")
            .output().unwrap().status.success());

        let wasm = fs::read("out.wasm").unwrap();

        let result = encode(&wasm, &data_imports, &data_funcs, [&lex_error_files, &parse_error_files, &[sema_errors]]);
        fs::write("out", result).unwrap();

         Ok(())
     })?;
 

     Ok(())

}

