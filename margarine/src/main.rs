use std::{collections::HashMap, ffi::OsStr, path::{Path, PathBuf}};

use common::{source::FileData, string_map::StringMap, DropTimer};
use margarine::{raylib::raylib, stdlib};
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
        "build" => {
            let Ok(toml) = std::fs::read("build.toml")
            else {
                println!("failed to read 'build.toml'");
                return;
            };


            let toml : toml::Table = match toml::from_slice(&toml) {
                Ok(v) => v,
                Err(e) => {
                    println!("failed to parse 'build.toml' {e}");
                    return;
                },
            };

            let empty = Table::new();

            let artifacts_path = Path::new("artifacts");
            if !std::fs::exists("artifacts").unwrap() {
                std::fs::create_dir("artifacts").unwrap();
            }

            //let package = toml.get("package").map(|x| x.as_table()).flatten().unwrap_or(&empty);
            let dependencies = toml.get("dependencies").map(|x| x.as_table()).flatten().unwrap_or(&empty);

            let string_map_arena = Arena::new();
            let mut string_map = StringMap::new(&string_map_arena);

            let mut files = vec![];

            let mut stack = vec![];
            for value in dependencies {
                let path = artifacts_path.join(&value.0);

                if !std::fs::exists(&path).unwrap() {
                    println!("fetching '{}'", value.1.as_str().unwrap());
                    let _ = git2::Repository::clone(value.1.as_str().unwrap(), &path).unwrap();
                }

                let dir = std::fs::read_dir(&path).unwrap();
                let path = artifacts_path.join(&value.0).join("src");

                let name = string_map.insert(value.0);
                stack.push(dir);
                while let Some(dir) = stack.pop() {
                    for file in dir {
                        let file = file.unwrap();

                        if file.path() == path.join("lib.mar") {
                            files.push(FileData::open_ex(file.path(), name, &mut string_map).unwrap())
                        } else if file.path().extension() == Some(OsStr::new("mar")) {
                            let name = file.file_name();
                            let name = name.to_string_lossy();
                            let path = string_map.insert(
                                &format!("{}/{}", value.0,
                                    &name[..name.len()-4]
                                )
                            );

                            files.push(FileData::open_ex(file.path(), path, &mut string_map).unwrap());
                        } else if file.metadata().unwrap().is_dir() {
                            stack.push(std::fs::read_dir(file.path()).unwrap());
                        }
                    }
                }
            }


            stack.push(std::fs::read_dir("src").unwrap());
            let name = string_map.insert("self");
            let src_dir = Path::new("src");
            while let Some(dir) = stack.pop() {
                for file in dir {
                    let file = file.unwrap();

                    if file.path() == src_dir.join("main.mar") {
                        files.push(FileData::open_ex(file.path(), name, &mut string_map).unwrap())
                    } else if file.path().extension() == Some(OsStr::new("mar")) {
                        let name = file.file_name();
                        let name = name.to_string_lossy();
                        let path = string_map.insert(
                            &format!("self/{}",
                                &name[..name.len()-4]
                            )
                        );

                        files.push(FileData::open_ex(file.path(), path, &mut string_map).unwrap());
                    } else if file.metadata().unwrap().is_dir() {
                        stack.push(std::fs::read_dir(file.path()).unwrap());
                    }
                }
            }

            let code = margarine::run(&mut string_map, files);


            let mut hosts : HashMap<String, _>= HashMap::new();
            stdlib(&mut hosts);
            raylib(&mut hosts);

            let mut vm = VM::new(hosts, &*code).unwrap();
            {
                let _t = DropTimer::new("runtime");
                if let Some(e) = vm.run("self::main").as_err() {
                    println!("{}", e.to_str().unwrap());
                }
            }
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


