#![feature(try_trait_v2)]
pub mod fmt;

use std::{fmt::Write, ops::{Deref, DerefMut, FromResidual}};

use common::{source::FileData, string_map::StringMap, small_vec::SmallVec};
use fmt::ErrorFormatter;


pub struct ErrorCode(u32);


pub trait ErrorType {
    fn display(&self, fmt: &mut ErrorFormatter);
}


#[derive(Debug, Clone)]
pub struct Errors<T: ErrorType>(SmallVec<T, 1>);

impl<T: ErrorType> Errors<T> {
    pub fn new() -> Self {
        Self(SmallVec::new())
    }


    pub fn push(&mut self, error: T) {
        self.0.push(error)
    }


    pub fn with(&mut self, errors: Errors<T>) {
        self.0.reserve(errors.len());

        for e in errors.0.into_iter() {
            self.0.push(e);
        }
    }


    pub fn is_empty(&self) -> bool {
        self.0.is_empty() 
    }


    pub fn display(&self, string_map: &StringMap, file: &[FileData]) -> String {
        let mut string = String::new();
        for e in self.0.as_slice() {
            if !string.is_empty() {
                let _ = writeln!(string);
            }

            let mut fmt = ErrorFormatter::new(&mut string, string_map, file);
            e.display(&mut fmt);
        }

        string
    }
}


impl<T: ErrorType> From<T> for Errors<T> {
    fn from(value: T) -> Self {
        let mut me = Self::new();
        me.push(value);
        me
    }
}


impl<T: ErrorType> FromResidual<T> for Errors<T> {
    fn from_residual(residual: T) -> Self {
        residual.into()
    }
}


impl<T: ErrorType> Deref for Errors<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.0.as_slice()
    }
}


impl<T: ErrorType> DerefMut for Errors<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut()
    }
}
