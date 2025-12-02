pub mod raylib;

use std::collections::HashMap;
use std::ffi::CString;
use std::ffi::OsStr;
use std::path::Path;
use std::sync::Mutex;

use common::string_map::StringIndex;
use errors::ParserError;
use git2::Repository;
pub use lexer::lex;
use parser::nodes::decl::Decl;
use parser::nodes::decl::DeclId;
use parser::nodes::expr::Block;
use parser::nodes::NodeId;
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
pub use sti::arena::Arena;
use sti::format_in;


pub fn run<'str>(string_map: &mut StringMap<'str>, files: FileData) -> (Vec<u8>, Vec<String>) {
    let arena = Arena::new();
    let mut global = AST::new(&arena);
    let mut lex_errors = vec![];
    let mut parse_errors = vec![];

    let mut stack = vec![(None, files.clone())];
    let mut source_offset = 0;
    let mut counter = 0;

    let mut root = None;

    let mut files = vec![];
    while let Some((decl, f)) = stack.pop() {
        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = lex(&f, string_map, source_offset);
            tokens
        });

        let (body, imports, mut pe) = DropTimer::with_timer("parsing", || {
            parse(tokens, counter, &arena, string_map, &mut global)
        });


        for (_, i) in imports {
            let source = global.range(i);
            let Decl::ImportFile { name, .. }= global.decl(i)
            else { unreachable!() };

            let path = string_map.get(f.name());
            let path = format_in!(&arena, "{}/{}.mar", path, string_map.get(name));
            let Ok(file) = FileData::open(&*path, string_map)
            else {
                let path_str = string_map.insert(&path);
                let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: path_str });
                global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                
                continue;
            };

            stack.push((Some((i, name)), file));
        }

        if let Some((decl, name)) = decl {
            let offset = global.range(decl);
            global.set_decl(decl, Decl::Module { name, header: offset, body, user_defined: true });
        } else {
            root = Some(body);
        }
        

        lex_errors.push(le);
        parse_errors.push(pe);

        counter += 1;

        source_offset += f.read().len() as u32;
        files.push(f);
    }

    let sema_arena = Arena::new();
    let temp = Arena::new();
    let _scopes = Arena::new();
    let mut sema = {
        let _1 = DropTimer::new("semantic analysis");
        TyChecker::run(&sema_arena, &temp, &mut global, &root.unwrap(), string_map)
    };

    // todo: find a way to comrpess these errors into vecs
    let mut lex_error_files = Vec::with_capacity(lex_errors.len());
    for l in lex_errors {
        let mut file = Vec::with_capacity(l.len());
        for e in l.iter() {
            let report = display(e, &sema.string_map, &files, &mut ());
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
            let report = display(e, &sema.string_map, &files, &mut ());
            #[cfg(not(feature = "fuzzer"))]
            println!("{report}");
            file.push(report);
        }

        parse_error_files.push(file);
    }

    let mut sema_errors = Vec::with_capacity(sema.errors.len());
    for s in sema.errors.iter() {
        let report = display(s, &sema.string_map, &files, &mut sema.syms);

        #[cfg(not(feature = "fuzzer"))]
        println!("{report}");

        sema_errors.push(report);
    } 


    let src = semantic_analysis::codegen::run(&mut sema, [lex_error_files, parse_error_files, vec![sema_errors]]);
    let mut tests = Vec::with_capacity(sema.startups.len());

    for t in sema.tests {
        let name = sema.syms.sym(t.1).name();
        tests.push(sema.string_map.get(name).to_string());
    }

    (src, tests)
}


pub fn stdlib(hosts: &mut HashMap<String, unsafe extern "C" fn(&mut VM, &mut Reg, &mut Status)>) {

    unsafe extern "C" fn print_raw(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let val = unsafe { vm.stack.reg(0) };
        let ty_id = unsafe { vm.stack.reg(1).as_int() };

        unsafe {
        match SymbolId(ty_id as u32) {
            SymbolId::I64 => println!("{}", val.as_int()),
            SymbolId::F64 => println!("{}", val.as_float()),
            SymbolId::BOOL => println!("{}", val.as_bool()),
            SymbolId::STR => println!("{}", vm.objs[val.as_obj()].as_str()),
            SymbolId::LIST => println!("{:?}", vm.objs[val.as_obj()].as_list()),

            //@todo
            _ => println!("{:?}", vm.objs[val.as_obj()])
        }
        }
    }

    unsafe extern "C" fn new_any(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let value = unsafe { vm.stack.reg(0) };
        let type_id = unsafe { vm.stack.reg(1) };

        let obj = vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![type_id, value] });

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }

    }


    unsafe extern "C" fn downcast_any(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let any_value = unsafe { vm.stack.reg(0) };
        let target_ty = unsafe { vm.stack.reg(1) };

        let obj = unsafe { any_value.as_obj() };
        let obj = vm.objs[obj].as_fields();

        let obj = unsafe {
            if obj[0].as_int() == target_ty.as_int() {
                vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(0), obj[1]] })
            } else {
                vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
            }
        };

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }



    }


    unsafe extern "C" fn push_list(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let list = unsafe { vm.stack.reg(0) };
        let value = unsafe { vm.stack.reg(1) };

        let list = unsafe { &mut vm.objs[list.as_obj()] };
        list.as_mut_list().push(value);
    }


    unsafe extern "C" fn pop_list(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let list = unsafe { vm.stack.reg(0) };

        let list = unsafe { &mut vm.objs[list.as_obj()] };
        let value = list.as_mut_list().pop();

        let obj = if let Some(value) = value {
            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(0), value] })
        } else {
            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
        };

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }


    }


    unsafe extern "C" fn len_list(vm: &mut VM, ret: &mut Reg, _: &mut Status) {
        let list = unsafe { vm.stack.reg(0) };
        let list = unsafe { &mut vm.objs[list.as_obj()] };
        *ret = Reg::new_int(list.as_list().len() as i64);
    }


    unsafe extern "C" fn now_secs(_: &mut VM, ret: &mut Reg, _: &mut Status) {
        let Ok(time) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
        else { panic!("failed to get the epoch") };

        let secs = time.as_secs();
        *ret = Reg::new_int(secs as i64)
    }


    unsafe extern "C" fn now_nanos(_: &mut VM, ret: &mut Reg, _: &mut Status) {
        let Ok(time) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
        else { panic!("failed to get the epoch") };

        let secs = time.subsec_nanos();
        *ret = Reg::new_int(secs as i64)
    }


    unsafe extern "C" fn int_to_str(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let int = unsafe { vm.stack.reg(0).as_int() };
        let obj = vm.new_obj(runtime::obj_map::ObjectData::String(int.to_string().into()));

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }

    }


    unsafe extern "C" fn float_to_str(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let int = unsafe { vm.stack.reg(0).as_float() };
        let obj = vm.new_obj(runtime::obj_map::ObjectData::String(int.to_string().into()));

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }

    }


    unsafe extern "C" fn random(_: &mut VM, ret: &mut Reg, _: &mut Status) {
        let obj = Reg::new_float(rand::random());
        *ret = obj
    }


    unsafe extern "C" fn hashmap_new(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let obj = vm.new_obj(runtime::obj_map::ObjectData::Dict(HashMap::new()));

        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }

    }


    unsafe extern "C" fn hashmap_insert(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let hm = vm.stack.reg(0).as_obj();
        let key = vm.stack.reg(1);
        let value = vm.stack.reg(2);

        let hm = vm.objs[hm].as_hm();
        hm.insert(key, value);
    }


    unsafe extern "C" fn hashmap_clear(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm].as_hm();

        hm.clear();
    }


    unsafe extern "C" fn hashmap_contains_key(vm: &mut VM, ret: &mut Reg, _: &mut Status) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm].as_hm();
        let key = vm.stack.reg(1);

        let val = hm.contains_key(&key);
        *ret = Reg::new_bool(val)
    }


    unsafe extern "C" fn hashmap_remove(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm].as_hm();
        let key = vm.stack.reg(1);

        let value = hm.remove(&key);

        let obj = if let Some(value) = value {
            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(0), value] })
        } else {
            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
        };


        match obj {
            Ok(v) => *ret = v,
            Err(e) => *status = Status::err(e),
        }
    }


    unsafe extern "C" fn panic(vm: &mut VM, _: &mut Reg, status: &mut Status) {
        let str = vm.stack.reg(0).as_obj();
        let str = vm.objs[str].as_str();

        *status = Status::err(FatalError::new(str))
    }


    hosts.insert("print_raw".to_string(), print_raw);
    hosts.insert("new_any".to_string(), new_any);
    hosts.insert("downcast_any".to_string(), downcast_any);
    hosts.insert("push_list".to_string(), push_list);
    hosts.insert("pop_list".to_string(), pop_list);
    hosts.insert("len_list".to_string(), len_list);
    hosts.insert("now_secs".to_string(), now_secs);
    hosts.insert("now_nanos".to_string(), now_nanos);
    hosts.insert("int_to_str".to_string(), int_to_str);
    hosts.insert("float_to_str".to_string(), float_to_str);
    hosts.insert("random".to_string(), random);
    hosts.insert("hashmap_new".to_string(), hashmap_new);
    hosts.insert("hashmap_insert".to_string(), hashmap_insert);
    hosts.insert("hashmap_clear".to_string(), hashmap_clear);
    hosts.insert("hashmap_contains_key".to_string(), hashmap_contains_key);
    hosts.insert("hashmap_remove".to_string(), hashmap_remove);
    hosts.insert("panic".to_string(), panic);
}
