#![allow(dead_code)]
pub mod string_map;
pub mod hashables;
pub mod source;
pub mod buffer;
pub mod utf8;

use std::{ops::Deref, time::Instant};

use sti::{alloc::Alloc, arena::Arena, vec::Vec};
use colourful::*;

pub use derive::ImmutableData;

pub struct DropTimer<'a> {
    message: &'a str,
    time: Instant,
}


impl<'a> DropTimer<'a> {
    pub fn new(message: &'a str) -> Self {
        Self {
            message,
            time: Instant::now(),
        }
    }


    #[inline(always)]
    pub fn with_timer<T, F: FnOnce() -> T>(message: &'a str, block: F) -> T {
        let _drop = DropTimer::new(message);
        block()
    }
}


impl Drop for DropTimer<'_> {
    fn drop(&mut self) {
        #[cfg(not(feature = "fuzzer"))]
        #[cfg(debug_assertions)]
        #[cfg(false)]
        println!("droptimer: ran '{}' in {:?}", self.message, self.time.elapsed());
    }
}


pub trait Swap {
    ///
    /// Swaps the current value with the given value
    /// returning the current value
    ///
    #[inline(always)]
    fn swap(&mut self, val: Self) -> Self where Self: Sized {
        core::mem::replace(self, val)
    }
}


impl<T> Swap for T {}


pub fn find_duplicate<'a, T: PartialEq, A: Alloc>(
    fields: &'a [T], 
    buff: &mut Vec<(&'a T, &'a T), A>
) {

    for i in 0..fields.len() {
        for j in 0..i {
            if fields[i] == fields[j] {
                buff.push((&fields[i], &fields[j]));
            }
        }
    }
}


pub fn warn(string: &str) {
    #[cfg(not(feature = "fuzzer"))]
    println!("{}: {string}", "warn".colour(Colour::rgb(207, 188, 148)).bold())
}


#[inline(always)]
pub fn num_size(num: u32) -> u32 {
    num.checked_ilog10().unwrap_or(0) + 1
}


pub fn copy_slice_in<'a, 'b, T: Copy>(arena: &'a Arena, slice: &'b [T]) -> &'a [T] {
    let mut vec = sti::vec::Vec::with_cap_in(arena, slice.len());
    unsafe { std::ptr::copy(slice.as_ptr(), vec.as_mut_ptr(), slice.len()) };
    unsafe { vec.set_len(slice.len()) }
    vec.leak()
}



pub struct NonEmpty<'a, T>(&'a [T]);

impl<'a, T> NonEmpty<'a, T> {
    pub fn new(body: &'a [T]) -> Option<Self> {
        if body.is_empty() { return None }
        Some(Self(body))
    }

}

impl<'a, T> Deref for NonEmpty<'a, T> {
    type Target = &'a [T];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}


#[derive(Clone, Copy, Debug)]
pub enum Either<A, B> {
    Left(A),
    Right(B),
}


pub struct Once<T>(Option<T>);

impl<T> Once<T> {
    pub fn new() -> Self { Self(None) }

    pub fn get(&self) -> Option<&T> { self.0.as_ref() }
    pub fn into_inner(self) -> Option<T> { self.0 }

    pub fn set(&mut self, value: T) {
        assert!(self.0.is_none());
        self.0 = Some(value);
    }
}
