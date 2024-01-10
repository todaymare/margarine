use std::ptr::null;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WasmPtr(i32);

impl WasmPtr {
    #[no_mangle]
    #[inline(always)]
    pub extern "C" fn as_ptr(self, ctx: &Ctx) -> *const u8 {
        unsafe { ctx.base.add(self.0 as usize) }
    }


    #[no_mangle]
    #[inline(always)]
    pub extern "C" fn as_mut(self, ctx: &Ctx) -> *mut u8 {
        self.as_ptr(ctx).cast_mut()
    }


    pub fn from_i32(i: i32) -> Self { Self(i) }
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    base: *const u8,
}

impl Ctx {
    pub const fn new() -> Self { Self { base: null() } }

    pub fn set_base(&mut self, ptr: *const u8) {
        assert!(self.base.is_null());
        assert!(!ptr.is_null());
        self.base = ptr;
    }
}
