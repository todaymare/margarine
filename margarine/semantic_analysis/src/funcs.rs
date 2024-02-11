use common::{string_map::StringIndex, Swap};
use sti::{define_key, keyed::KVec, hash::{HashMap, DefaultSeed}, arena::Arena};
use wasm::FunctionId;

use crate::types::{ty::Type, ty_map::TypeId};

define_key!(u32, pub FuncId);


#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: StringIndex,
    pub args: &'a [(StringIndex, bool, Type)],
    pub ret : Type,
    pub kind: FunctionKind,
    pub wasm_id: FunctionId,
}


#[derive(Debug, Clone, Copy)]
pub enum FunctionKind {
    UserDefined {
        inout: Option<TypeId>,
    },

    Extern {
        ty: TypeId,
    }, 
}


impl<'a> Function<'a> {
    pub fn new(name: StringIndex, args: &'a [(StringIndex, bool, Type)], ret: Type, wasm_id: FunctionId, kind: FunctionKind) -> Self { Self { name, args, ret, kind, wasm_id } }
}


#[derive(Debug)]
pub struct FunctionMap<'a> {
    map: KVec<FuncId, Option<Function<'a>>>,
}


impl<'a> FunctionMap<'a> {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
        }
    }


    #[inline(always)]
    pub fn get(&self, id: FuncId) -> &Function<'a> {
        &self.map.get(id).unwrap().as_ref().unwrap()
    }


    #[inline(always)]
    pub fn pending(&mut self) -> FuncId {
        self.map.push(None)
    }


    #[inline(always)]
    pub fn put(&mut self, func_id: FuncId, ns: Function<'a>) {
        assert!(self.map[func_id].swap(Some(ns)).is_none());
    }
}
