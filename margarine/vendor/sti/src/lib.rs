#![cfg_attr(not(feature="std"), no_std)]
#![forbid(unsafe_op_in_unsafe_fn)]

pub mod prelude;


pub mod hint;

pub mod arch;
pub mod ffi;
pub mod atomic;
pub mod mem;
pub mod ptr;
pub mod key;
pub mod slice;
pub mod str;

pub mod num;
pub mod ops;
pub mod cmp;
pub mod borrow;

pub mod panic;
pub mod fmt;

pub mod ext;
pub mod reader;
pub mod static_vec;
pub mod slice_vec;
pub mod byte_mask;

pub mod alloc;
pub mod asan;
pub mod arena;
pub mod boxed;
pub mod vec;
pub mod hash;
pub mod string;
pub mod lru;

pub mod sync;

pub mod leb128;

mod macros;
pub use core::{file, line, column, stringify};
mod cfg_if;

