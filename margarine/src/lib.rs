use std::collections::HashMap;

pub use lexer::lex;
use parser::nodes::decl::Decl;
use parser::nodes::AST;
pub use parser::parse;
pub use parser::nodes;
pub use common::source::{FileData, Extension};
pub use common::string_map::StringMap;
pub use common::{DropTimer, source::SourceRange};
use semantic_analysis::syms::sym_map::SymbolId;
pub use semantic_analysis::{TyChecker};
pub use errors::display;
pub use runtime::{VM, opcode, Status, FatalError, Reg};
use sti::arena::Arena;


pub fn run(string_map: &mut StringMap<'_>, files: Vec<FileData>) -> Vec<u8> {
    let arena = Arena::new();
    let mut global = AST::new();
    let mut modules = vec![];
    let mut lex_errors = vec![];
    let mut parse_errors = vec![];

    let mut source_offset = 0;
    for (i, f) in files.iter().enumerate() {
        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = lex(&f, string_map, source_offset);
            tokens
        });

        let (body, pe) = DropTimer::with_timer("parsing", || {
            parse(tokens, i.try_into().unwrap(), &arena, string_map, &mut global)
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
        TyChecker::run(&sema_arena, &temp, &mut global, &*modules, string_map)
    };

    // todo: find a way to comrpess these errors into vecs
    let mut lex_error_files = Vec::with_capacity(lex_errors.len());
    for l in lex_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = display(e.1, &sema.string_map, &files, &mut ());
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
            let report = display(e.1, &sema.string_map, &files, &mut ());
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(sema.errors.len());
    for s in sema.errors.iter() {
        let report = display(s.1, &sema.string_map, &files, &mut sema.syms);
        #[cfg(not(feature = "fuzzer"))]
        println!("{report}");
        sema_errors.push(report);
    } 


    let src = semantic_analysis::codegen::run(&mut sema, [lex_error_files, parse_error_files, vec![sema_errors]]);
    src
}


pub fn stdlib(hosts: &mut HashMap<String, fn(&mut VM) -> Reg>) {
    hosts.insert("print_raw".to_string(), |vm| {
        let val = unsafe { vm.stack.reg(0) };
        let ty_id = unsafe { vm.stack.reg(1).as_int() };

        unsafe {
        match SymbolId::new_unck(ty_id as u32) {
            SymbolId::I64 => println!("{}", val.as_int()),
            SymbolId::F64 => println!("{}", val.as_float()),
            SymbolId::BOOL => println!("{}", val.as_bool()),
            SymbolId::STR => println!("{}", vm.objs[val.as_obj() as usize].as_str()),
            SymbolId::LIST => println!("{:?}", vm.objs[val.as_obj() as usize].as_list()),

            //@todo
            _ => println!("{:?}", vm.objs[val.as_obj() as usize].as_fields())
        }
        }
        Reg::new_unit()
    });


    hosts.insert("new_any".to_string(), |vm| {
        let value = unsafe { vm.stack.reg(0) };
        let type_id = unsafe { vm.stack.reg(1) };
        dbg!(value, type_id);

        let obj = vm.new_obj(runtime::Object::Struct { fields: vec![type_id, value] });
        obj
    });


    hosts.insert("downcast_any".to_string(), |vm| {
        let any_value = *unsafe { &vm.stack.reg(0) };
        let target_ty = *unsafe { &vm.stack.reg(1) };

        let obj = unsafe { any_value.as_obj() };
        let obj = vm.objs[obj as usize].as_fields();

        unsafe {
            if obj[0].as_int() == target_ty.as_int() {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(0), obj[1]] })
            } else {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
            }
        }
    });


}
