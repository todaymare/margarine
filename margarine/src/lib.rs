pub mod raylib;

use std::collections::HashMap;

use common::string_map::StringIndex;
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


pub fn run(string_map: &mut StringMap<'_>, files: Vec<FileData>) -> Vec<u8> {
    let arena = Arena::new();
    let mut global = AST::new(&arena);
    let mut modules = HashMap::<&[StringIndex], Block>::new();
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

        let path = {
            let mut list = sti::vec::Vec::new_in(&arena);
            let path = string_map.get(f.name()).split('/');
            for module in path {
                let id = string_map.insert(module);
                list.push(id);
            }
            list.leak()
        };

        modules.insert(path, body);

        source_offset += f.read().len() as u32;
    }

    #[derive(Debug)]
    struct Module {
        tree: HashMap<StringIndex, Module>,
        body: sti::vec::Vec<NodeId>,
        range: SourceRange,
    }

    let mut module_tree : HashMap<StringIndex, Module> = HashMap::new();

    let mut depth = 1;
    let mut active = false;
    loop {
        for (path, block) in &modules {
            if path.len() != depth { continue }

            active = true;
            let mut module : Option<&mut Module> = None;
            for path in path.iter().take(path.len() - 1) {
                let m = match module {
                    Some(v) => v.tree.get_mut(path).unwrap(),
                    None => module_tree.get_mut(path).unwrap(),
                };

                module = Some(m);
            }

            let curr_module = Module {
                tree: HashMap::new(),
                body: sti::vec::Vec::from_slice(block.body()),
                range: block.range(),
            };

            match module {
                Some(module) => {
                    module.tree.insert(*path.last().unwrap(), curr_module)
                },
                None => module_tree.insert(*path.last().unwrap(), curr_module),
            };
        }
        depth += 1;

        if !active {
            break;
        }
        active = false;
    }


    fn register_module(name: StringIndex, module: &mut Module, ast: &mut AST) -> DeclId {
        for (&name, child) in module.tree.iter_mut() {
            module.body.insert(0, register_module(name, child, ast).into());
        }

        ast.add_decl(
            Decl::Module {
                name,
                header: module.range,
                body: Block::new(module.body.clone_in(ast.arena).leak(), module.range)
            },
            module.range
        )
    }


    let mut modules = vec![];
    for (&name, module) in module_tree.iter_mut() {
        let decl = register_module(name, module, &mut global);
        modules.insert(0, decl.into());
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
    src
}


pub fn stdlib(hosts: &mut HashMap<String, unsafe extern "C" fn(&mut VM, &mut Reg)>) {

    unsafe extern "C" fn print_raw(vm: &mut VM, _: &mut Reg) {
        let val = unsafe { vm.stack.reg(0) };
        let ty_id = unsafe { vm.stack.reg(1).as_int() };

        unsafe {
        match SymbolId(ty_id as u32) {
            SymbolId::I64 => println!("{}", val.as_int()),
            SymbolId::F64 => println!("{}", val.as_float()),
            SymbolId::BOOL => println!("{}", val.as_bool()),
            SymbolId::STR => println!("{}", vm.objs[val.as_obj() as usize].as_str()),
            SymbolId::LIST => println!("{:?}", vm.objs[val.as_obj() as usize].as_list()),

            //@todo
            _ => println!("{:?}", vm.objs[val.as_obj() as usize])
        }
        }
    }

    unsafe extern "C" fn new_any(vm: &mut VM, ret: &mut Reg) {
        let value = unsafe { vm.stack.reg(0) };
        let type_id = unsafe { vm.stack.reg(1) };

        *ret = vm.new_obj(runtime::Object::Struct { fields: vec![type_id, value] });

    }


    unsafe extern "C" fn downcast_any(vm: &mut VM, ret: &mut Reg) {
        let any_value = unsafe { vm.stack.reg(0) };
        let target_ty = unsafe { vm.stack.reg(1) };

        let obj = unsafe { any_value.as_obj() };
        let obj = vm.objs[obj as usize].as_fields();

        *ret = unsafe {
            if obj[0].as_int() == target_ty.as_int() {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(0), obj[1]] })
            } else {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
            }
        }

    }


    unsafe extern "C" fn push_list(vm: &mut VM, _: &mut Reg) {
        let list = unsafe { vm.stack.reg(0) };
        let value = unsafe { vm.stack.reg(1) };

        let list = unsafe { &mut vm.objs[list.as_obj() as usize] };
        match list {
            runtime::Object::List(regs) => regs.push(value),
            _ => unreachable!(),
        }
    }


    unsafe extern "C" fn pop_list(vm: &mut VM, ret: &mut Reg) {
        let list = unsafe { vm.stack.reg(0) };

        let list = unsafe { &mut vm.objs[list.as_obj() as usize] };
        let value = match list {
            runtime::Object::List(regs) => regs.pop(),
            _ => unreachable!(),
        };

        *ret = if let Some(value) = value {
            vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(0), value] })
        } else {
            vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
        }
    }


    unsafe extern "C" fn len_list(vm: &mut VM, ret: &mut Reg) {
        let list = unsafe { vm.stack.reg(0) };
        let list = unsafe { &mut vm.objs[list.as_obj() as usize] };
        *ret = Reg::new_int(list.as_list().len() as i64);
    }


    unsafe extern "C" fn now_secs(_: &mut VM, ret: &mut Reg) {
        let Ok(time) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
        else { panic!("failed to get the epoch") };

        let secs = time.as_secs();
        *ret = Reg::new_int(secs as i64)
    }


    unsafe extern "C" fn now_nanos(_: &mut VM, ret: &mut Reg) {
        let Ok(time) = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
        else { panic!("failed to get the epoch") };

        let secs = time.subsec_nanos();
        *ret = Reg::new_int(secs as i64)
    }


    unsafe extern "C" fn int_to_str(vm: &mut VM, ret: &mut Reg) {
        let int = unsafe { vm.stack.reg(0).as_int() };
        let obj = vm.new_obj(runtime::Object::String(int.to_string().into()));
        *ret = obj
    }


    unsafe extern "C" fn float_to_str(vm: &mut VM, ret: &mut Reg) {
        let int = unsafe { vm.stack.reg(0).as_float() };
        let obj = vm.new_obj(runtime::Object::String(int.to_string().into()));
        *ret = obj
    }


    unsafe extern "C" fn random(_: &mut VM, ret: &mut Reg) {
        let obj = Reg::new_float(rand::random());
        *ret = obj
    }


    unsafe extern "C" fn hashmap_new(vm: &mut VM, ret: &mut Reg) {
        let obj = vm.new_obj(runtime::Object::Dict(HashMap::new()));
        *ret = obj
    }


    unsafe extern "C" fn hashmap_insert(vm: &mut VM, _: &mut Reg) {
        let hm = vm.stack.reg(0).as_obj();
        let key = vm.stack.reg(1);
        let value = vm.stack.reg(2);

        let hm = vm.objs[hm as usize].as_hm();
        hm.insert(key, value);
    }


    unsafe extern "C" fn hashmap_clear(vm: &mut VM, _: &mut Reg) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm as usize].as_hm();

        hm.clear();
    }


    unsafe extern "C" fn hashmap_contains_key(vm: &mut VM, ret: &mut Reg) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm as usize].as_hm();
        let key = vm.stack.reg(1);

        let val = hm.contains_key(&key);
        *ret = Reg::new_bool(val)
    }


    unsafe extern "C" fn hashmap_remove(vm: &mut VM, ret: &mut Reg) {
        let hm = vm.stack.reg(0).as_obj();
        let hm = vm.objs[hm as usize].as_hm();
        let key = vm.stack.reg(1);

        let value = hm.remove(&key);

        *ret = if let Some(value) = value {
            vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(0), value] })
        } else {
            vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
        }
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

}
