pub mod ctx;
pub mod tys;
pub mod info;
pub mod module;
pub mod builder;
pub mod values;
pub mod global;

#[macro_export]
macro_rules! cstr { ($s: literal) => { concat!($s, "\0").as_ptr() as *const i8 }; }


