use common::{string_map::StringIndex, Swap};
use sti::{define_key, keyed::KVec};
use wasm::FunctionId;

use crate::types::{ty::Type, ty_map::TypeId};

define_key!(u32, pub FuncId);


#[derive(Debug, Clone)]
pub struct Function<'a> {
    pub name: StringIndex,
    pub path: StringIndex,
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
    pub fn new(name: StringIndex, path: StringIndex, args: &'a [(StringIndex, bool, Type)],
               ret: Type, wasm_id: FunctionId, kind: FunctionKind) -> Self {
        Self { name, args, ret, kind, wasm_id, path }
    }
}


#[derive(Debug)]
enum FunctionItem<'a> {
    Some(Function<'a>),
    Errored,
    None,
}


#[derive(Debug)]
pub struct FunctionMap<'a> {
    map: KVec<FuncId, FunctionItem<'a>>,
}


impl<'a> FunctionMap<'a> {
    pub fn new() -> Self {
        Self {
            map: KVec::new(),
        }
    }


    #[inline(always)]
    pub fn get(&self, id: FuncId) -> Option<&Function<'a>> {
        match self.map.get(id).unwrap() {
            FunctionItem::Some(v) => Some(v),
            FunctionItem::Errored => None,
            _ => unreachable!(),
        }
    }


    #[inline(always)]
    pub fn pending(&mut self) -> FuncId {
        self.map.push(FunctionItem::None)
    }


    #[inline(always)]
    pub fn put(&mut self, func_id: FuncId, ns: Function<'a>) {
        match self.map[func_id] {
            FunctionItem::Some(_) => panic!(),
            FunctionItem::Errored => (),
            FunctionItem::None => self.map[func_id] = FunctionItem::Some(ns),
        }
    }

    pub fn error(&mut self, id: FuncId) {
        self.map[id] = FunctionItem::Errored;
    }

    pub fn iter<'b>(&'b self) -> impl Iterator<Item=&'b Function<'a>> {
        self.map.iter().filter_map(|x| if let FunctionItem::Some(v) = x.1 { Some(v) } else { None })
    }


    pub fn len(&self) -> usize { self.map.len() }
}
