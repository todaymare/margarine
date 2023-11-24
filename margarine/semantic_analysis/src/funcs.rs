use common::string_map::StringIndex;
use sti::{define_key, keyed::KVec};

use crate::types::Type;

define_key!(u32, pub FuncId);


#[derive(Debug)]
pub struct Function<'a> {
    name: StringIndex,
    args: &'a [Type],
    ret : Type,
}

impl<'a> Function<'a> {
    pub fn new(name: StringIndex, args: &'a [Type], ret: Type) -> Self { Self { name, args, ret } }
}


#[derive(Debug)]
pub struct FunctionMap<'a> {
    map: KVec<FuncId, Function<'a>>,
}


impl<'a> FunctionMap<'a> {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
        }
    }


    #[inline(always)]
    pub fn get(&self, id: FuncId) -> &Function {
        self.map.get(id).unwrap()
    }


    #[inline(always)]
    pub fn put(&mut self, ns: Function<'a>) -> FuncId {
        self.map.push(ns)
    }
}
