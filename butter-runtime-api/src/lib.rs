use std::fmt::write;

use ffi::Ctx;
pub use proc_macros::*;
pub mod ffi;
pub mod alloc;


pub fn dump_stack_trace(ctx: &Ctx) {
    let bsp = ctx.instance().get_global(ctx.store(), "bstack_pointer").unwrap();
    let sp = ctx.instance().get_global(ctx.store(), "stack_pointer").unwrap();
    let bsp = u32::from_ne_bytes(bsp.get(ctx.store()).unwrap_i32().to_ne_bytes());
    let mut sp = u32::from_ne_bytes(sp.get(ctx.store()).unwrap_i32().to_ne_bytes());

    let mem = ctx.mem();
    let mut buffer = [0; 4];

    println!("dumping stack");
    while sp != bsp {
        mem.read(ctx.store(), sp as usize + 4, &mut buffer).unwrap();
        let func_id = u32::from_ne_bytes(buffer);

        println!("func: {func_id}, sp {sp}, bsp {bsp}");

        mem.read(ctx.store(), sp.try_into().unwrap(), &mut buffer).unwrap();
        sp = u32::from_ne_bytes(buffer);
        if sp == 0 { break }
    }
}
