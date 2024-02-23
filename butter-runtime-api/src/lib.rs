use core::slice;
use std::{fmt::write, fs, io::{stdin, BufRead, Read}};

use ffi::Ctx;
pub use proc_macros::*;
pub mod ffi;
pub mod alloc;


pub fn dump_stack_trace(ctx: &Ctx) {
    let bsp = ctx.instance().get_global(ctx.store(), "bstack_pointer").unwrap();
    let sp = ctx.instance().get_global(ctx.store(), "stack_pointer").unwrap();
    let bsp = bsp.get(ctx.store()).unwrap_i32() as u32;
    let mut sp = sp.get(ctx.store()).unwrap_i32() as u32;

    let mem = ctx.mem();
    let mut buffer = [0; 4];

    println!("--------- PRINTING STACK ---------");
    let mut i = 0;
    while sp != bsp {
        mem.read(ctx.store(), sp as usize + 4, &mut buffer).unwrap();
        let func_id = u32::from_ne_bytes(buffer);

        let func_name = ctx.funcs()[func_id as usize];

        println!("{i} - {func_name} ({func_id})");

        mem.read(ctx.store(), sp.try_into().unwrap(), &mut buffer).unwrap();
        let new_sp = u32::from_ne_bytes(buffer);

        sp = new_sp;

        fs::write("hexdump", unsafe { slice::from_raw_parts(mem.data_ptr(ctx.store()), 64 * 1024) }).unwrap();
        let _ = stdin().lock().read_line(&mut String::new());

        i += 1;
    }


}
