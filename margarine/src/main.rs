use std::collections::HashMap;

use common::{source::FileData, string_map::StringMap, DropTimer};
use margarine::{raylib::raylib, stdlib};
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
    stdlib(&mut hosts);
    raylib(&mut hosts);

    let mut vm = VM::new(hosts, &*src).unwrap();
    dbg!(&vm.funcs);
    {
        let _t = DropTimer::new("runtime");
        match vm.run("flappy_bird::main") {
            Status::Ok => (),
            Status::Err(fatal_error) => println!("{}", fatal_error.msg),
        }
    }
}
