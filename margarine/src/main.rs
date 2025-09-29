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

    let mut vm = VM::new(hosts, &*src).unwrap();
    dbg!(&vm.funcs);
    {
        let _t = DropTimer::new("runtime");
        match vm.run("test::main") {
            Status::Ok => (),
            Status::Err(fatal_error) => println!("{}", fatal_error.msg),
        }
    }


}
