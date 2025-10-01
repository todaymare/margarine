use std::collections::HashMap;

use common::{source::FileData, string_map::StringMap, DropTimer};
use runtime::{Reg, Status, VM};
use sti::arena::Arena;

fn main() {
    let src = DropTimer::with_timer("compilation", || {
       let string_map_arena = Arena::new();
       let mut string_map = StringMap::new(&string_map_arena);
       let files = {
           let mut files = Vec::new();
           for i in std::env::args().skip(1) {
               files.push(FileData::open(&i, &mut string_map).expect(&format!("{}", i)));
           }

           files
       };

       margarine::run(&mut string_map, files)
    });




    let mut hosts : HashMap<String, fn(&mut VM) -> Reg>= HashMap::new();
    hosts.insert("print".to_string(), |vm| {
        println!("{:?}", unsafe { &vm.stack.reg(0) });
        Reg::new_unit()
    });

    hosts.insert("print_str".to_string(), |vm| {
        let obj = *unsafe { &vm.stack.reg(0).as_obj() };
        let obj = &vm.objs[obj as usize];
        println!("{}", obj.as_str());
        Reg::new_unit()
    });


    hosts.insert("too".to_string(), |vm| {
        let obj = *unsafe { &vm.stack.reg(0) };
        obj
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

        dbg!(obj);
        dbg!(target_ty);

        unsafe {
            if obj[0].as_int() == target_ty.as_int() {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(0), obj[1]] })
            } else {
                vm.new_obj(runtime::Object::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
            }
        }
    });

    let mut vm = VM::new(hosts, &*src).unwrap();
    dbg!(&vm.funcs);
    {
        let _t = DropTimer::new("runtime");
        match vm.run("examples/test::main") {
            Status::Ok => (),
            Status::Err(fatal_error) => println!("{}", fatal_error.msg),
        }
    }


}
