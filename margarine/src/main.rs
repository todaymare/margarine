use std::{collections::HashMap, ffi::OsStr, fmt::Write, path::{Path, PathBuf}};

use colourful::ColourBrush;
use common::{source::FileData, string_map::StringMap, DropTimer};
use margarine::{build_system, init_compilation_unit_id, raylib::raylib, stdlib, CompilationUnit, ACTIVE_UNITS};
use runtime::{Reg, Status, VM};
use sti::arena::Arena;
use toml::Table;

fn main() {
    let mut args = std::env::args().skip(1);

    let Some(command) = args.next()
    else { 
        println!("invalid command");
        return;
    };

    match command.as_str() {
        "run" => {
            let (code, _) = compile_curr_project();

            println!("running");
            let mut hosts : HashMap<String, _>= HashMap::new();
            stdlib(&mut hosts);
            build_system(&mut hosts);
            raylib(&mut hosts);

            let mut vm = VM::new(hosts, &*code).unwrap();
            {
                let _t = DropTimer::new("runtime");
                if let Some(e) = vm.run("self::main", &[]).as_err() {
                    println!("{}", e.to_str().unwrap());
                }
            }
            return;
        },


        "test" => {
            /*
            let Some((code, tests)) = compile_curr_project()
            else { return };

            dbg!(&tests);

            let mut hosts : HashMap<String, _>= HashMap::new();
            stdlib(&mut hosts);
            raylib(&mut hosts);

            let mut vm = VM::new(hosts, &*code).unwrap();
            {
                let _t = DropTimer::new("runtime");

                println!();
                println!("running {} tests", tests.len());
                println!();

                let mut fails = String::new();
                for t in tests {
                    let result = vm.run(&t);

                    println!("test {t} .. {}", if result.as_err().is_some() { "FAILED".red() } else { "ok".green() });

                    if let Some(err) = result.as_err() {
                        writeln!(&mut fails, "failed '{t}':\n{}", err.to_string_lossy()).unwrap();
                    }

                    vm.reset();
                }

                println!();
                if !fails.is_empty() {
                    println!("failures:");
                    println!();
                    println!("{}", fails);
                    println!();
                }

            }
            */
            return;
        },


        "clean" => {
            if std::fs::exists("artifacts").unwrap() {
                std::fs::remove_dir_all("artifacts").unwrap();
            }
        }


        _ => {
            println!("invalid command");
            return;
        }
    }

    /*
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
    {
        let _t = DropTimer::new("runtime");
        if let Some(e) = vm.run("flappy_bird::main").as_err() {
            println!("{}", e.to_str().unwrap());
        }
    }*/

}




fn compile_curr_project() -> (Vec<u8>, Vec<String>) {
    println!("{} 'build.mar'", "compiling".green());

    let mut unit = CompilationUnit::default();
    unit.import_repo("std", "https://github.com/todaymare/margarine-std");

    let string_map_arena = Arena::new();
    let mut string_map = StringMap::new(&string_map_arena);

    let mut files = vec![];
    files.push(FileData::open("build.mar", &mut string_map).unwrap());

    let (code, _) = unit.build(&mut string_map, files);

    let mut hosts : HashMap<String, _>= HashMap::new();
    stdlib(&mut hosts);
    build_system(&mut hosts);


    let mut vm = VM::new(hosts, &*code).unwrap();
    let id = init_compilation_unit_id();

    {
        let _t = DropTimer::new("runtime");
        if let Some(e) = vm.run("build::build", &[Reg::new_int(id as i64)]).as_err() {
            println!("{}", e.to_str().unwrap());
        }
    }

    let mut lock = ACTIVE_UNITS.lock().unwrap();
    lock.get_mut(id).unwrap().as_mut().unwrap().build_curr_project()
}
