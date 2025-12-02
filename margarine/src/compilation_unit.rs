use std::{collections::{HashMap, HashSet}, path::Path};

use common::source::FileData;
use runtime::{FatalError, Reg, Status, VM};

pub struct CompilationInfo {
    imports: HashMap<String, String>,
    files  : Vec<FileData>,
}


impl CompilationInfo {
    pub fn new() -> Self {
        let mut imports = HashMap::new();
        imports.insert("std".to_string(), "https://github.com/todaymare/margarine-std");

        Self {
            imports: HashMap::new(),
            files: vec![],
        }
    }
}


unsafe extern "C" fn init_compilation_unit(vm: &mut VM, ret: &mut Reg, _: &mut Status) {
    let ptr = CompilationInfo::new();
    let ptr = Box::into_raw(Box::new(ptr));
    let obj = vm.new_obj(runtime::obj_map::ObjectData::Ptr(ptr.cast())).unwrap();
    *ret = obj;
}


unsafe extern "C" fn compilation_unit_import_repo(vm: &mut VM, _: &mut Reg, status: &mut Status) {
    unsafe {
    let compilation_unit = vm.stack.reg(0).as_obj();

    let name = vm.stack.reg(1).as_obj();
    let name = vm.objs[name].as_str();
    let path = vm.stack.reg(2).as_obj();
    let path = vm.objs[path].as_str();


    let compilation_unit = vm.objs[compilation_unit].as_ptr();
    let compilation_unit = compilation_unit.cast::<CompilationInfo>();

    if compilation_unit.is_null() {
        *status = Status::err(FatalError::new("compilation unit is already consumed"));
        return;
    }


    {
        let compilation_unit = &mut *compilation_unit;
        if compilation_unit.imports.contains_key(name) { return };

        compilation_unit.imports.insert(name.to_string(), path.to_string());
    }

    let artifacts_path = Path::new("artifacts");

    if !std::fs::exists("artifacts").unwrap() {
        println!("creating 'artifacts'");
        std::fs::create_dir("artifacts").unwrap();
    }

    let dir_path = artifacts_path.join(name);

    if !std::fs::exists(&path).unwrap() {
        println!("fetching '{}'", path);
        let _ = git2::Repository::clone(path, &dir_path).unwrap();
    }

    let build_script = dir_path.join("build.mar");

    println!("compiling {}", name);

    let data = std::fs::read(build_script).unwrap();

    let mut compilation_unit = CompilationInfo::new();



    }
}


unsafe extern "C" fn compilation_unit_build(vm: &mut VM, _: &mut Reg, status: &mut Status) {
    unsafe {
    let compilation_unit = vm.stack.reg(0).as_obj();
    let compilation_unit = vm.objs[compilation_unit].as_ptr();
    let compilation_unit = compilation_unit.cast::<CompilationInfo>();

    if compilation_unit.is_null() {
        *status = Status::err(FatalError::new("compilation unit is already consumed"));
        return;
    }


    let compilation_unit = &mut *compilation_unit;
    }
}



