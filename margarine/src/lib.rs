#![feature(if_let_guard)]
use std::collections::HashMap;
use std::fs;

use colourful::ColourBrush;
use git2::Repository;
pub use lexer::lex;
use parser::nodes::decl::Decl;
use parser::nodes::AST;
pub use parser::parse;
pub use parser::nodes;
pub use common::source::{FileData, Extension};
pub use common::string_map::StringMap;
pub use common::{DropTimer, source::SourceRange};
use runtime::obj_map::ObjectData;
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
    let mut build_lock = BuildLock::load();

    let mut stack = vec![(None, files.clone(), 0)];
    let mut source_offset = 0;
    let mut counter = 0;

    let mut root = None;

    let mut files = vec![];
    while let Some((decl, f, depth)) = stack.pop() {
        if depth != 0 {
            let name = string_map.get(f.name());

            let name =
            if name.starts_with("artifacts/") {
                &name["artifacts/".len()..]
            } else { &name };

            println!(
                "{}{}{} {} {}.mar", 
                "|".dark_grey(), 
                "-".repeat(depth).dark_grey(), 
                ">".dark_grey(), 
                "compiling:".green().bold(),
                name.replace("<>", "::")
            );


        } else {
            println!(
                "{} {}.mar",
                "compiling:".green().bold(),
                string_map.get(f.name())
            );
        }


        let (tokens, le) = DropTimer::with_timer("tokenisation", || {
            let tokens = lex(&f, string_map, source_offset);
            tokens
        });

        let (body, imports, mut pe) = DropTimer::with_timer("parsing", || {
            parse(tokens, counter, &arena, string_map, &mut global)
        });


        for (_, i) in imports {
            let source = global.range(i);
            match global.decl(i) {
                Decl::ImportFile { name, .. } => {
                    let path = string_map.get(f.name());
                    let path = format_in!(&arena, "{}/{}.mar", path, string_map.get(name));
                    let Ok(file) = FileData::open(&*path, string_map)
                    else {
                        let path_str = string_map.insert(&path);
                        let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: path_str });
                        global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                        
                        continue;
                    };

                    stack.push((Some((i, name)), file, depth+1));
                }


                Decl::ImportRepo { alias, repo } => {
                    let repo_str = string_map.get(repo);
                    let (url, commit) = if repo_str.contains('@') {
                        let parts: Vec<_> = repo_str.splitn(2, '@').collect();
                        (parts[0], Some(parts[1]))
                    } else {
                        (repo_str, None)
                    };



                    let local_path = string_map.get(f.name())
                        .replace("<>", "<_>")
                        .replace("/", "<>");
                    let alias_str = string_map.get(alias);
                    let alias_str = format!("{local_path}<>{alias_str}");

                    // Convert github/owner/repo format to URL
                    let url = if url.starts_with("github/") {
                        let parts: Vec<&str> = url.split('/').collect();
                        if parts.len() == 3 {
                            format!("https://github.com/{}/{}.git", parts[1], parts[2])
                        } else {
                            let err = pe.push(parser::errors::Error::FileDoesntExist { 
                                source, 
                                path: repo 
                            });
                            global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                            continue;
                        }
                    } else {
                        url.to_string()
                    };

                    let artifacts_dir = "artifacts";
                    if !std::fs::exists(artifacts_dir).unwrap_or(false) {
                        std::fs::create_dir(artifacts_dir).unwrap();
                    }

                    let local_path = format!("{}/{}", artifacts_dir, alias_str);

                    if !std::fs::exists(&local_path).unwrap_or(false) {
                        println!("{}{}{} {} {}", "|".dark_grey(), "-".repeat(depth+1).dark_grey(), ">".dark_grey(), "downloading...".green().bold(), url);

                        let repo =
                        match Repository::clone(&url, &local_path) {
                            Ok(v) => v,
                            Err(_) => {
                                let err = pe.push(parser::errors::Error::FileDoesntExist { 
                                    source, 
                                    path: repo 
                                });
                                global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                                continue;
                            },
                        };


                        let target_commit =
                        if let Some(commit) = commit {
                            commit.to_string()
                        } else if let Some(lock) = build_lock.get(&alias_str) {
                            lock
                        } else {
                            // Get HEAD commit
                            match repo.head() {
                                Ok(head) => {
                                    head.target()
                                        .map(|oid| oid.to_string())
                                        .unwrap_or_else(|| "HEAD".to_string())
                                }
                                Err(_) => "HEAD".to_string(),
                            }
                        };

                        // Checkout the commit
                        if let Ok(obj) = repo.revparse_single(&target_commit) {
                            let _ = repo.checkout_tree(&obj, None);
                        }


                        // Update lock file
                        build_lock.set(alias_str.to_string(), target_commit);

                    } else if let Some(commit) = commit {
                        // If repo exists but user specified a commit, checkout it
                        if let Ok(repo) = Repository::open(&local_path) {
                            if let Ok(obj) = repo.revparse_single(commit) {
                                let _ = repo.checkout_tree(&obj, None);
                            }
                            build_lock.set(alias_str.to_string(), commit.to_string());
                        }
                    }

                    // Load lib.mar from the cloned repo
                    let lib_path = format!("{}/lib.mar", local_path);
                    let Ok(file) = FileData::open(&lib_path, string_map)
                    else {
                        let lib_path_str = string_map.insert(&lib_path);
                        let err = pe.push(parser::errors::Error::FileDoesntExist { source, path: lib_path_str });
                        global.set_decl(i, Decl::Error(errors::ErrorId::Parser((counter, err))));
                        
                        continue;
                    };

                    stack.push((Some((i, alias)), file, depth+1));
                }


                _ => unreachable!()
            }

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

    // Save the build lock file
    let _ = build_lock.save();

    (src, tests)
}


struct BuildLock {
    packages: HashMap<String, String>, // alias -> commit hash
}

impl BuildLock {
    fn load() -> Self {
        match fs::read_to_string("build.lock") {
            Ok(content) => {
                let mut lock = BuildLock { packages: HashMap::new() };

                for line in content.lines() {
                    let (name, commit) = line.split_once(",").unwrap();
                    lock.packages.insert(name.to_string(), commit.to_string());
                }

                lock
            }

            Err(_) => BuildLock { packages: HashMap::new() },
        }
    }

    fn save(&self) -> std::io::Result<()> {
        let mut content = String::new();
        for (alias, commit) in &self.packages {
            sti::write!(&mut content, "{},{}\n", alias, commit);
        }

        fs::write("build.lock", content)
    }

    fn get(&self, alias: &str) -> Option<String> {
        self.packages.get(alias).cloned()
    }

    fn set(&mut self, alias: String, commit: String) {
        self.packages.insert(alias, commit);
    }
}


pub fn stdlib(hosts: &mut HashMap<String, unsafe extern "C" fn(&mut VM, &mut Reg, &mut Status)>) {

    unsafe extern "C" fn print_raw(vm: &mut VM, _: &mut Reg, _: &mut Status) {
        let val = unsafe { vm.stack.reg(0) };
        let ty_id = unsafe { vm.stack.reg(1).as_int() };

        unsafe {
        match SymbolId(ty_id as u32) {
            SymbolId::I64 => print!("{}", val.as_int()),
            SymbolId::F64 => print!("{}", val.as_float()),
            SymbolId::BOOL => print!("{}", val.as_bool()),
            SymbolId::STR => print!("{}", vm.objs[val.as_obj()].as_str()),
            SymbolId::LIST => print!("{:?}", vm.objs[val.as_obj()].as_list()),

            //@todo
            _ => print!("{:?}", vm.objs[val.as_obj()])
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


    unsafe extern "C" fn io_read_line(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let mut str = String::new();
        let value = std::io::stdin().read_line(&mut str);

        let obj = 'b: {
        if let Err(e) = value {
            let str = e.to_string();
            let str =
            match vm.new_obj(ObjectData::String(str.into())) {
                Ok(v) => v,
                Err(v) => break 'b Err(v),
            };

            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(1), str] })
        } else {
            let str =
            match vm.new_obj(ObjectData::String(str.into())) {
                Ok(v) => v,
                Err(v) => break 'b Err(v),
            };

            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(0), str] })
        } };


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


    unsafe extern "C" fn str_lines_iter(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let str = vm.stack.reg(0);

        let obj = vm.new_obj(ObjectData::List(vec![
            str,
            Reg::new_int(0)
        ]));


        *ret = match obj {
            Ok(v) => v,
            Err(v) => {
                *status = Status::err(v);
                return;
            },
        };
    }


    unsafe extern "C" fn str_lines_iter_next(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let obj_id = vm.stack.reg(0).as_obj();

        let obj = vm.objs.get(obj_id).as_list();

        let str = obj[0].as_obj();
        let str = vm.objs.get(str).as_str();

        let offset = obj[1].as_int();
        if offset >= str.len() as _ {
            let obj = vm.new_obj(
                ObjectData::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] });

            *ret = match obj {
                Ok(v) => v,
                Err(e) => {
                    *status = Status::err(e);
                    return
                },
            };

            return;
        }

        let str = &str[offset as usize..];
        let str = str.lines().next();
        let new_offset = offset + str.unwrap_or("").len() as i64 + 1;


        let reg = if let Some(line) = str {
            let line = vm.new_obj(ObjectData::String(line.into()));
            let line = match line {
                Ok(v) => v,
                Err(v) => {
                    *status = Status::err(v);
                    return;
                },
            };

            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(0), line] })
        } else {
            vm.new_obj(runtime::obj_map::ObjectData::Struct { fields: vec![Reg::new_int(1), Reg::new_unit()] })
        };


        let reg = match reg {
            Ok(v) => v,
            Err(v) => {
                *status = Status::err(v);
                return;
            },
        };
        
        let obj = vm.objs.get_mut(obj_id).as_mut_list();
        obj[1] = Reg::new_int(new_offset);

        *ret = reg;
    }


    unsafe extern "C" fn str_split_at(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let obj_id = vm.stack.reg(0).as_obj();

        let str = vm.objs.get(obj_id).as_str();

        let split_pos = vm.stack.reg(1).as_int();

        if split_pos >= str.len() as _ {
            *status = Status::err(FatalError::new(
                    &format!("index '{split_pos}' is out of bounds for string '{str}'")));
            return;
        }

        let (s1, s2) = str.split_at(split_pos as usize);
        let s1 = s1.into();
        let s2 = s2.into();

        let s1 = match vm.new_obj(ObjectData::String(s1)) {
            Ok(v) => v,
            Err(v) => {
                *status = Status::err(v);
                return;
            },
        };

        let s2 = match vm.new_obj(ObjectData::String(s2)) {
            Ok(v) => v,
            Err(v) => {
                *status = Status::err(v);
                return;
            },
        };

        let tuple = match vm.new_obj(ObjectData::Struct { fields: vec![s1, s2] }) {
            Ok(v) => v,
            Err(v) => {
                *status = Status::err(v);
                return;
            },
        };

        *ret = tuple;
    }



    unsafe extern "C" fn str_parse(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let str_id = vm.stack.reg(0).as_obj();
        let str = vm.objs.get(str_id).as_str().trim();

        let ty = vm.stack.reg(1).as_int();

        let result = match SymbolId(ty as u32) {
            SymbolId::I64 if let Ok(v) = str.parse() => {
                vec![Reg::new_int(0), Reg::new_int(v)]
            },

            _ => {
                vec![Reg::new_int(1), Reg::new_unit()]
            }
        };



        let obj = vm.new_obj(ObjectData::Struct { fields: result });
        *ret = match obj {
            Ok(v) => v,
            Err(e) => {
                *status = Status::err(e);
                return;
            },
        }
    }


    unsafe extern "C" fn str_len(vm: &mut VM, ret: &mut Reg, _: &mut Status) {
        let str_id = vm.stack.reg(0).as_obj();
        let str = vm.objs.get(str_id).as_str();

        *ret = Reg::new_int(str.len() as _);
    }


    unsafe extern "C" fn str_slice(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let str_id = vm.stack.reg(0).as_obj();
        let str = vm.objs.get(str_id).as_str();

        let min = vm.stack.reg(1).as_int();
        let max = vm.stack.reg(2).as_int();

        let sliced = 
            str.chars()
            .skip(min as usize)
            .take((max - min) as usize)
            .collect::<String>();

        let obj = vm.new_obj(ObjectData::String(sliced.into()));

        *ret = match obj {
            Ok(v) => v,
            Err(e) => {
                *status = Status::err(e);
                return;
            },
        }
    }


    unsafe extern "C" fn str_split_once(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let str_id = vm.stack.reg(0).as_obj();
        let str = vm.objs.get(str_id).as_str();

        let del_id = vm.stack.reg(1).as_obj();
        let del = vm.objs.get(del_id).as_str();

        let sliced = str.split_once(del);

        let fields =
        if let Some((s1, s2)) = sliced {
            let s1 = s1.into();
            let s2 = s2.into();

            let s1 = match vm.new_obj(ObjectData::String(s1)) {
                Ok(v) => v,
                Err(v) => {
                    *status = Status::err(v);
                    return;
                },
            };


            let s2 = match vm.new_obj(ObjectData::String(s2)) {
                Ok(v) => v,
                Err(v) => {
                    *status = Status::err(v);
                    return;
                },
            };


            let tuple = match vm.new_obj(ObjectData::Struct { fields: vec![s1, s2] }) {
                Ok(v) => v,
                Err(v) => {
                    *status = Status::err(v);
                    return;
                },
            };

            vec![Reg::new_int(0), tuple]
        } else {
            vec![Reg::new_int(1), Reg::new_unit()]
        };


        let obj = vm.new_obj(ObjectData::Struct { fields });

        *ret = match obj {
            Ok(v) => v,
            Err(e) => {
                *status = Status::err(e);
                return;
            },
        }
    }


    unsafe extern "C" fn io_read_file(vm: &mut VM, ret: &mut Reg, status: &mut Status) {
        let str_id = vm.stack.reg(0).as_obj();
        let str = vm.objs.get(str_id).as_str();

        let path = std::fs::read_to_string(str).unwrap();

        *ret = match vm.new_obj(ObjectData::String(path.into())) {
            Ok(v) => v,
            Err(e) => {
                *status = Status::err(e);
                return;
            },
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
    hosts.insert("io_read_line".to_string(), io_read_line);
    hosts.insert("io_read_file".to_string(), io_read_file);
    hosts.insert("str_lines_iter".to_string(), str_lines_iter);
    hosts.insert("str_lines_iter_next".to_string(), str_lines_iter_next);
    hosts.insert("str_split_at".to_string(), str_split_at);
    hosts.insert("str_parse".to_string(), str_parse);
    hosts.insert("str_len".to_string(), str_len);
    hosts.insert("str_slice".to_string(), str_slice);
    hosts.insert("str_split_once".to_string(), str_split_once);
    hosts.insert("panic".to_string(), panic);
}
