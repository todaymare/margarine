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
    let temp = Arena::new();
    let _scopes = Arena::new();
    let mut sema = {
        let _1 = DropTimer::new("semantic analysis");
        margarine::TyChecker::run(&sema_arena, &temp, &mut global, &*modules, string_map)
    };

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

    let (ctx, module) = Codegen::run(&mut sema);

    // transfer errors into the binary
    {
        let u32_ty = ctx.integer(32);
        let ptr_ty = ctx.ptr();
        

        // file count
        {
            let file_count_global = module.add_global(*u32_ty, "fileCount");
            let file_count_value = ctx.const_int(u32_ty, files.len() as i64, false);
            file_count_global.set_initialiser(*file_count_value);
        }


        // lexer errors
        {
            let strct = ctx.structure("lexer_err_ty");
            strct.set_fields(&[*u32_ty, *ptr_ty], false);

            let errs_ty = ctx.array(*strct, parse_error_files.len());
            let mut err_arr_values = Vec::with_capacity(parse_error_files.len());

            for file in lex_error_files {
                let file_err_array_ty = ctx.array(*ptr_ty, file.len());
                let mut file_arr_values = Vec::with_capacity(file.len());

                for s in &file {
                    let global = ctx.const_str(&*s);
                    let ptr = module.add_global(*global.ty(), "");
                    ptr.set_initialiser(*global);
                    file_arr_values.push(*ptr);
                }

                // value arr
                let ptr = module.add_global(*file_err_array_ty, "");
                let arr = ctx.const_array(*ptr_ty, &file_arr_values);
                ptr.set_initialiser(*arr);

                let len = ctx.const_int(u32_ty, file.len() as i64, false);

                let strct = ctx.const_struct(strct, &[*len, *ptr]);
                err_arr_values.push(*strct);
            }

            // value arr
            let ptr = module.add_global(*errs_ty, "lexerErrors");
            let arr = ctx.const_array(*strct, &err_arr_values);
            ptr.set_initialiser(*arr);
        }


        // parser errors
        {
            let strct = ctx.structure("parser_err_ty");
            strct.set_fields(&[*u32_ty, *ptr_ty], false);

            let errs_ty = ctx.array(*strct, parse_error_files.len());
            let mut err_arr_values = Vec::with_capacity(parse_error_files.len());

            for file in parse_error_files {
                let file_err_array_ty = ctx.array(*ptr_ty, file.len());
                let mut file_arr_values = Vec::with_capacity(file.len());

                for s in &file {
                    let global = ctx.const_str(&*s);
                    let ptr = module.add_global(*global.ty(), "");
                    ptr.set_initialiser(*global);
                    file_arr_values.push(*ptr);
                }

                // value arr
                let ptr = module.add_global(*file_err_array_ty, "");
                let arr = ctx.const_array(*ptr_ty, &file_arr_values);
                ptr.set_initialiser(*arr);

                let len = ctx.const_int(u32_ty, file.len() as i64, false);

                let strct = ctx.const_struct(strct, &[*len, *ptr]);
                err_arr_values.push(*strct);
            }

            // value arr
            let ptr = module.add_global(*errs_ty, "parserErrors");
            let arr = ctx.const_array(*strct, &err_arr_values);
            ptr.set_initialiser(*arr);
        }


        // sema errors
        let sema_err_array_ty = ctx.array(*ptr_ty, sema_errors.len());
        let mut arr_values = Vec::with_capacity(sema_errors.len());

        for e in &sema_errors {
            let global = ctx.const_str(&*e);
            let ptr = module.add_global(*global.ty(), "");
            ptr.set_initialiser(*global);
            arr_values.push(*ptr);
        }

        let ptr = module.add_global(*sema_err_array_ty, "semaErrors");
        let array = ctx.const_array(*ptr.ty(), &arr_values);
        ptr.set_initialiser(*array);

        let ptr = module.add_global(*u32_ty, "semaErrorsLen");
        let num = ctx.const_int(u32_ty, sema_errors.len().try_into().unwrap(), false);
        ptr.set_initialiser(*num);
    }


    #[cfg(not(feature="fuzzer"))]
    {
        let str = module.dump_to_str();
        fs::write("out.ll", str.as_str()).unwrap();

        if let Err(e) = module.validate() {
            println!("COMPILER ERROR! {}", e.red().bold());
            return Err("fatal compiler error");
        }

        Command::new("/opt/homebrew/opt/llvm/bin/clang")
            .arg("/opt/homebrew/lib/libglfw.3.4.dylib")
            .arg("target/debug/libcore.a")
            .arg("out.ll")
            .arg("-o")
            .arg("out")
            .arg("-Wall")
            .arg("-Wno-override-module")
            .arg("-framework")
            .arg("OpenGL")
            .spawn().unwrap().wait().unwrap();
    }


    Ok(())
}
