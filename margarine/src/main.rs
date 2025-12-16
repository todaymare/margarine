use std::{collections::HashMap, fmt::Write};

use colourful::ColourBrush;
use common::{source::FileData, string_map::StringMap, DropTimer};
use sti::{arena::Arena};

fn main() {
    let mut args = std::env::args().skip(1);

    let Some(command) = args.next()
    else { 
        println!("invalid command");
        return;
    };

    match command.as_str() {
        "run" => {
            let path = args.next().unwrap();
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let files = FileData::open(path, &mut sm).unwrap();
            let (code, _) = margarine::run(sm, files);

            println!("running");

            /*
            let mut hosts : HashMap<String, _>= HashMap::new();
            stdlib(&mut hosts);

            let mut vm = VM::new(hosts, &*code).unwrap();
            {
                let _t = DropTimer::new("runtime");
                if let Some(e) = vm.run("main", &[]).as_err() {
                    println!("{}", e.to_str().unwrap());
                }
            }
            */
            return;
        },


        "test" => {
            let path = args.next().unwrap_or_else(|| ".".to_string());
            let arena = Arena::new();
            let mut sm = StringMap::new(&arena);
            let files = FileData::open(path, &mut sm).unwrap();
            let (code, tests) = margarine::run(sm, files);

            /*
            let mut hosts : HashMap<String, _>= HashMap::new();
            stdlib(&mut hosts);

            let mut vm = VM::new(hosts, &*code).unwrap();
            {
                let _t = DropTimer::new("runtime");

                println!();
                println!("running {} tests", tests.len());
                println!();

                let mut fails = String::new();
                for t in tests {
                    let result = vm.run(&t, &[]);

                    println!("test '{t}' .. {}", if result.as_err().is_some() { "FAILED".red() } else { "ok".green() });

                    if let Some(err) = result.as_err() {
                        let err_str = err.to_str().unwrap_or("unknown error");
                        writeln!(&mut fails, "failed '{t}':\n{}", err_str).unwrap();
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


        "update" => {
            if std::fs::exists("build.lock").unwrap() {
                std::fs::remove_file("build.lock").unwrap();
            }

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

