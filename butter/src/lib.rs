use std::{fs, process::Command};

use colourful::ColourBrush;
use margarine::{nodes::{decl::Decl, AST}, Codegen, DropTimer, FileData, SourceRange, StringMap};
use sti::arena::Arena;

pub fn run(string_map: &mut StringMap<'_>, files: Vec<FileData>) -> Result<(), &'static str> {
    let arena = Arena::new();
    let mut global = AST::new();
    let mut modules = vec![];
    let mut lex_errors = vec![];
    let mut parse_errors = vec![];

    let mut source_offset = 0;
    for (i, f) in files.iter().enumerate() {
        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = margarine::lex(&f, string_map, source_offset);
            tokens
        });

        let (body, pe) = DropTimer::with_timer("parsing", || {
            margarine::parse(tokens, i.try_into().unwrap(), &arena, string_map, &mut global)
        });

        lex_errors.push(le);
        parse_errors.push(pe);

        modules.push(global.add_decl(
            Decl::Module {
                name: f.name(),
                body,
                header: SourceRange::new(0, 0),
           },

           SourceRange::new(source_offset, source_offset + f.read().len() as u32),
        ).into());

       source_offset += f.read().len() as u32;
    }

    assert_eq!(lex_errors.len(), files.len());
    assert_eq!(parse_errors.len(), files.len());


    let sema_arena = Arena::new();
    let _scopes = Arena::new();
    let mut sema = {
        let _1 = DropTimer::new("semantic analysis");
        margarine::TyChecker::run(&sema_arena, &mut global, &*modules, string_map)
    };

    dbg!(&sema);

    // todo: find a way to comrpess these errors into vecs
    let mut lex_error_files = Vec::with_capacity(lex_errors.len());
    for l in lex_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = margarine::display(e.1, &sema.string_map, &files, &mut ());
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
            let report = margarine::display(e.1, &sema.string_map, &files, &mut ());
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(sema.errors.len());
    for s in sema.errors.iter() {
        let report = margarine::display(s.1, &sema.string_map, &files, &mut sema.syms);
        #[cfg(not(feature = "fuzzer"))]
        println!("{report}");
        sema_errors.push(report);
    } 

    let codegen = Codegen::run(&mut sema);
    let module = codegen.0.module(codegen.1);

    let str = module.dump_to_str();
    fs::write("out.ll", str).unwrap();

    if let Err(e) = module.validate() {
        println!("COMPILER ERROR! {}", e.red().bold());
        return Err("fatal compiler error");
    }


    Command::new("clang")
        .arg("engine.c")
        .arg("out.ll")
        .arg("-o")
        .arg("out")
        .arg("-Wall")
        .arg("-Wno-override-module")
        .spawn().unwrap().wait().unwrap();


    Ok(())
}
