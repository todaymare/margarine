use std::{ptr::null, marker::PhantomData};

///
/// A pointer to wasm memory
///
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct WasmPtr(u32);

impl WasmPtr {
    ///
    /// Returns a const reference to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_ptr(self, ctx: &Ctx) -> *const u8 {
        assert!(ctx.size >= self.0);
        unsafe { ctx.base.add(self.0 as usize) }
    }


    ///
    /// Returns a mutable pointer to the wasm memory
    ///
    /// # Panics
    /// If the pointer value is not within the range of
    /// the `ctx` memory
    ///
    #[inline(always)]
    pub extern "C" fn as_mut(self, ctx: &Ctx) -> *mut u8 {
        self.as_ptr(ctx).cast_mut()
    }


    ///
    /// Constructs a new `WasmPtr` out of a `u32`
    ///
    /// This function is safe as other subsequent safe calls
    /// will check for the bounds and panic if out of bounds
    ///
    pub extern "C" fn from_u32(i: u32) -> Self { Self(i) }
}


#[repr(C)]
#[derive(Debug, PartialEq)]
pub struct Ptr<T>(i64, PhantomData<T>);


impl<T> Ptr<T> {
    pub extern "C" fn new(data: *mut T) -> Self  { Ptr (data as i64, PhantomData) }


    pub extern "C" fn as_ref<'a>(self) -> &'a T {
        unsafe { &*(self.0 as *const i64).cast() }
    }


    pub extern "C" fn as_mut<'a>(self) -> &'a mut T {
        unsafe { &mut *(self.0 as *mut i64).cast() }
    }
}


impl<T> Clone for Ptr<T> {
    fn clone(&self) -> Self {
        Self(self.0, self.1)
    }
}

impl<T> Copy for Ptr<T> {}
unsafe impl<T> Sync for Ptr<T> {}
unsafe impl<T> Send for Ptr<T> {}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    base: *const u8,
    size: u32,
}

impl Ctx {
    pub const fn new() -> Self { Self { base: null(), size: 0 } }

    pub fn set_base(&mut self, ptr: *const u8) {
        assert!(self.base.is_null());
        assert!(!ptr.is_null());
        self.base = ptr;
    }

    pub fn set_size(&mut self, len: u32) {
        self.size = len;
    }
}



#[macro_export]
macro_rules! enum_ty {
    ($u_name: ident, $name: ident { $($t: literal $n: ident : $ty: ty),* }) => {
        #[repr(C)]
        #[derive(Clone, Copy)]
        struct $name {
            tag: u32,
            data: $u_name,
        }

        #[derive(Clone, Copy)]
        union $u_name {
            $(
                $n : $ty
            ),*
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut dbg = f.debug_struct(stringify!($name));

                match self.tag {
                    $(
                        $t => dbg.field(stringify!($n), unsafe { &self.data.$n }),
                    )*
                    _ => panic!("unknown variant"),
                };
                
                dbg.finish()
            }
        }


        impl std::cmp::PartialEq for $name {
            fn eq(&self, oth: &Self) -> bool {
                if self.tag != oth.tag { return false }

                match self.tag {
                    $(
                        $t => unsafe { self.data.$n == oth.data.$n },
                    )*
                    _ => panic!("unknown variant"),
                }
            }
        }


    };
}


#[macro_export]
macro_rules! func {
    ($(fn $name: ident ( $($n: ident : $aty: ty),* ) -> $ret: ty $body: block )*) => {
        #[allow(non_camel_case_types)]
        mod __func_tys {
            use super::{$($($aty,)* $ret),+};
            $(
            #[repr(C)]
            #[derive(Debug, Clone, Copy)]
            pub struct $name {
                $(
                    pub $n: $aty,
                )*

                pub __ret: $ret
            }
            )*
        }


        $(
        #[no_mangle]
        pub extern "C" fn $name(ctx: &Ctx, __argp: *mut __func_tys::$name) {
            let ($($n),*) = {
                let data = unsafe { &*__argp };
                ($(data.$n),*)
            };

            let ret = || -> $ret { $body };
            let ret = ret();

            unsafe { &mut *__argp }.__ret = ret;
        }
        )+
    };
}


