use common::{string_map::StringIndex, Swap};
use parser::Block;
use sti::{define_key, keyed::KVec};
use wasm::FunctionId;

use crate::{types::{ty::Type, ty_map::TypeId}, scope::ScopeId};

define_key!(u32, pub FuncId);


#[derive(Debug, Clone, Copy)]
pub struct Function<'a, 'ast> {
    pub name: StringIndex,
    pub args: &'a [(StringIndex, bool, Type)],
    pub ret : Type,
    pub kind: FunctionKind<'ast>,
    pub wasm_id: FunctionId,
}


#[derive(Debug, Clone, Copy)]
pub enum FunctionKind<'ast> {
    UserDefined {
        inout: Option<TypeId>,
    },

    Extern {
        ty: TypeId,
    }, 

    Template {
        body: Block<'ast>,
        scope: ScopeId,
    },
}


impl<'a, 'ast> Function<'a, 'ast> {
    pub fn new(name: StringIndex, args: &'a [(StringIndex, bool, Type)], ret: Type, wasm_id: FunctionId, kind: FunctionKind<'ast>) -> Self { Self { name, args, ret, kind, wasm_id } }
}


#[derive(Debug)]
pub struct FunctionMap<'a, 'ast> {
    map: KVec<FuncId, Option<Function<'a, 'ast>>>,
}


impl<'a, 'ast> FunctionMap<'a, 'ast> {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
        }
    }


    #[inline(always)]
    pub fn get(&self, id: FuncId) -> Function<'a, 'ast> {
        self.map.get(id).unwrap().unwrap()
    }


    #[inline(always)]
    pub fn pending(&mut self) -> FuncId {
        self.map.push(None)
    }


    #[inline(always)]
    pub fn put(&mut self, func_id: FuncId, ns: Function<'a, 'ast>) {
        assert!(self.map[func_id].swap(Some(ns)).is_none());
    }
}
