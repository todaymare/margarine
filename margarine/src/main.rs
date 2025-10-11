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




    let mut hosts : HashMap<String, _>= HashMap::new();
    stdlib(&mut hosts);
    raylib(&mut hosts);

    let mut vm = VM::new(hosts, &*src).unwrap();
    dbg!(&vm.funcs);
    {
        let _t = DropTimer::new("runtime");
        if let Some(e) = vm.run("test::main").as_err() {
            println!("{}", e.to_str().unwrap());
        }
    }
}
