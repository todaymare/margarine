use common::string_map::StringIndex;
use sti::{define_key, keyed::KVec};
use wasm::FunctionId;

use crate::types::ty::Type;

define_key!(u32, pub FuncId);


#[derive(Debug, Clone, Copy)]
pub struct Function<'a> {
    pub name: StringIndex,
    pub args: &'a [(StringIndex, bool, Type)],
    pub ret : Type,
    pub wasm_id: FunctionId,
}

impl<'a> Function<'a> {
    pub fn new(name: StringIndex, args: &'a [(StringIndex, bool, Type)], ret: Type, wasm_id: FunctionId) -> Self { Self { name, args, ret, wasm_id } }
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
    pub fn get(&self, id: FuncId) -> Function {
        *self.map.get(id).unwrap()
    }


    #[inline(always)]
    pub fn put(&mut self, ns: Function<'a>) -> FuncId {
        self.map.push(ns)
    }
}
