use std::{env, fs, io::{Read, Write}, process::{Command, ExitCode, Stdio}};

use game_runtime::encode;
use margarine::{nodes::{decl::{Declaration, DeclarationNode}, Node}, DropTimer, FileData, SourceRange, StringMap};
use sti::prelude::Arena;

pub fn run(string_map: &mut StringMap<'_>, files: Vec<FileData>) -> Result<(), &'static str> {
    let mut global = vec![];
    let mut lex_errors = vec![];
    let mut parse_errors = vec![];

    let arena = Arena::new();
    let mut source_offset = 0;
    for (i, f) in files.iter().enumerate() {
        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = margarine::lex(&f, string_map, source_offset);
            tokens
        });

        let (ast, pe) = DropTimer::with_timer("parsing", || {
            let ast = margarine::parse(tokens, i.try_into().unwrap(), &arena, string_map);
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
        margarine::Analyzer::run(&ns_arena, string_map, &*global)
    };

    // todo: find a way to comrpess these errors into vecs
    let mut lex_error_files = Vec::with_capacity(lex_errors.len());
    for l in lex_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = margarine::display(e.1, &sema.string_map, &files, &());
            #[cfg(not(feature = "fuzzer"))]
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
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(sema.errors.len());
    for s in sema.errors.iter() {
        let report = margarine::display(s.1, &sema.string_map, &files, &sema.types);
        #[cfg(not(feature = "fuzzer"))]
        println!("{report}");
        sema_errors.push(report);
    } 

    dbg!(&sema);

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
     
    #[cfg(not(feature = "fuzzer"))]
    {
        fs::write("out.wat", &*code).unwrap();

        let mut wat2wasm = Command::new("wat2wasm")
            .arg("-")
            .arg("--output=-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn().unwrap();

        wat2wasm.stdin.as_mut().unwrap().write_all(&code).unwrap();
        assert!(wat2wasm.wait().unwrap().success());

        let mut wasmopt = Command::new("wasm-opt")
            .arg("-O4")
            .arg("--enable-bulk-memory")
            // .arg("-iit")
            .arg("-aimfs=128")
            .arg("-fimfs=512")
            // .arg("-lmu")
            .arg("-pii=64")
            .arg("-ifwl")
            .arg("-o=-")
            .stdin(Stdio::from(wat2wasm.stdout.unwrap()))
            .stdout(Stdio::piped())
            .spawn().unwrap();

        assert!(wasmopt.wait().unwrap().success());
        let mut wasm = Vec::new();
        wasmopt.stdout.unwrap().read_to_end(&mut wasm).unwrap();

        let wasm = wasm;
        fs::write("out.wasm", &*wasm).unwrap();

        let result = encode(&wasm, &data_imports, &data_funcs, [&lex_error_files, &parse_error_files, &[sema_errors]]);
        fs::write("out", result).unwrap();
    }
    

     Ok(())
}
