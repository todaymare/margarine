use std::env;

use margarine::{DropTimer, FileData, StringMap};
use sti::arena::Arena;

fn main() -> Result<(), &'static str> {
    DropTimer::with_timer("compilation", || {
       let string_map_arena = Arena::new();
       let mut string_map = StringMap::new(&string_map_arena);
       let files = {
           let mut files = Vec::new();
           for i in env::args().skip(1) {
               files.push(FileData::open(&i, &mut string_map).expect(&format!("{}", i)));
           }

           files
       };

       butter::run(&mut string_map, files.clone())?;

       Ok(())
    })
}

