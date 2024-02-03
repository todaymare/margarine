use common::{string_map::StringIndex, Swap};
use parser::{Block, nodes::decl::{Generic, FunctionArgument}, DataType};
use sti::{define_key, keyed::KVec, hash::{HashMap, DefaultSeed}, arena::Arena};
use wasm::FunctionId;

use crate::{types::{ty::Type, ty_map::TypeId}, scope::ScopeId};

define_key!(u32, pub FuncId);


#[derive(Debug, Clone)]
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
        args: &'ast [FunctionArgument<'ast>],
        ret: DataType<'ast>,
        scope: ScopeId,
        generics: &'ast [Generic],
    },
}


impl<'a, 'ast> Function<'a, 'ast> {
    pub fn new(name: StringIndex, args: &'a [(StringIndex, bool, Type)], ret: Type, wasm_id: FunctionId, kind: FunctionKind<'ast>) -> Self { Self { name, args, ret, kind, wasm_id } }
}


#[derive(Debug)]
pub struct FunctionMap<'a, 'ast> {
    map: KVec<
        FuncId, 
        Option<(
            Function<'a, 'ast>,
            HashMap<&'a [(StringIndex, Type)], FuncId, DefaultSeed, &'a Arena>
    )>>,
    arena: &'a Arena,
}


impl<'a, 'ast> FunctionMap<'a, 'ast> {
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            map: KVec::new(),
            arena,
        }
    }


    #[inline(always)]
    pub fn get(&self, id: FuncId) -> &Function<'a, 'ast> {
        &self.map.get(id).unwrap().as_ref().unwrap().0
    }


    pub fn get_func_variant(&self, func_id: FuncId, ty: &[(StringIndex, Type)]) -> Option<FuncId> {
        self.map[func_id].as_ref().unwrap().1.get(ty).copied()
    }


    #[inline(always)]
    pub fn pending(&mut self) -> FuncId {
        self.map.push(None)
    }


    #[inline(always)]
    pub fn put(&mut self, func_id: FuncId, ns: Function<'a, 'ast>) {
        assert!(self.map[func_id].swap(Some((ns, HashMap::new_in(self.arena)))).is_none());
    }


    #[inline(always)]
    pub fn put_variant(&mut self, base: FuncId, gens: &'a [(StringIndex, Type)], variant: FuncId) {
        assert!(self.map[base].as_mut().unwrap().1.insert(gens, variant).is_none());
    }
}
