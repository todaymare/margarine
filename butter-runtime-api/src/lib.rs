use ffi::Ctx;
pub use proc_macros::*;

use crate::ptr::WasmPtr;
pub mod ffi;
pub mod alloc;
pub mod ptr;


pub fn dump_stack_trace(ctx: &Ctx) {
    let bsp = ctx.read_global("bstack_pointer");
    let sp = ctx.read_global("stack_pointer");
    let bsp = bsp.u32();
    let mut sp = sp.u32();

    println!("--------- PRINTING STACK ---------");
    let mut i = 0;
    while sp != bsp {
        let ptr = WasmPtr::from_u32(sp + 4);
        let func_id = ctx.copy_mem::<u32>(ptr);
        let func_name = ctx.funcs()[func_id as usize];

        println!("{i} - {func_name} ({func_id})");

        let new_sp = ctx.copy_mem(WasmPtr::from_u32(sp));

        sp = new_sp;

        i += 1;
    }
}
