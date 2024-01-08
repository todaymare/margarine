use std::{env, fs};

use game_runtime::decode;
use wasmer::{Module, Engine, NativeEngineExt, Store, Cranelift, Instance, imports};

fn main() {

    let data = {
        let file = env::current_exe().unwrap();
        let file = fs::read(file).unwrap();

        decode(&file)
    };

    let cranelift = Cranelift::new();
    let mut store = Store::new(cranelift);
    let module = unsafe { Module::deserialize(&store, &*data) }.unwrap();
    let instance = Instance::new(&mut store, &module, &imports! {}).unwrap();
    let main = instance.exports.get_function("main").unwrap();
    let result = main.call(&mut store, &[]).unwrap();
    println!("{result:?}");

}
