use common::string_map::StringIndex;
use sti::{define_key, hash::{HashMap, DefaultSeed}, keyed::KVec, arena::Arena};

use crate::{funcs::FuncId, types::{ty::Type, ty_map::{TypeId, TypeMap}}};

define_key!(u32, pub NamespaceId);


#[derive(Debug)]
pub struct Namespace<'out> {
    types: HashMap<StringIndex, TypeId, DefaultSeed, &'out Arena>,
    funcs: HashMap<StringIndex, FuncId, DefaultSeed, &'out Arena>,
    modules: HashMap<StringIndex, NamespaceId, DefaultSeed, &'out Arena>,
    path: StringIndex,
}


impl<'out> Namespace<'out> {
    pub fn new(arena: &'out Arena, path: StringIndex) -> Self {
        Namespace::with_ty_and_fn_cap(arena, path, 0, 0)
    }


    pub fn with_fn_cap(arena: &'out Arena, path: StringIndex, fn_cap: usize) -> Self {
        Self::with_ty_and_fn_cap(arena, path, 0, fn_cap)
    }


    pub fn with_ty_cap(arena: &'out Arena, path: StringIndex, ty_cap: usize) -> Self {
        Self::with_ty_and_fn_cap(arena, path, ty_cap, 0)
    }


    pub fn with_ty_and_fn_cap(arena: &'out Arena, path: StringIndex, ty_cap: usize, fn_cap: usize) -> Self {
        Namespace {
            types: HashMap::with_cap_in(arena, ty_cap),
            funcs: HashMap::with_cap_in(arena, fn_cap),
            modules: HashMap::with_cap_in(arena, 0),
            path,
        }
    }
    

    pub fn add_type(&mut self, name: StringIndex, ty: TypeId) {
        let prev_value = self.types.insert(name, ty);
        assert!(prev_value.is_none());
    }


    pub fn add_func(&mut self, name: StringIndex, func: FuncId) {
        let prev_value = self.funcs.insert(name, func);
        assert!(prev_value.is_none());
    }


    pub fn add_mod(&mut self, name: StringIndex, module: NamespaceId) {
        let prev_value = self.modules.insert(name, module);
        assert!(prev_value.is_none());
    }


    pub fn get_type(&self, id: StringIndex) -> Option<TypeId> {
        self.types.get(&id).copied()
    }


    pub fn get_func(&self, id: StringIndex) -> Option<FuncId> {
        self.funcs.get(&id).copied()
    }

    pub fn get_mod(&self, id: StringIndex) -> Option<NamespaceId> {
        self.modules.get(&id).copied()
    }

    pub fn path(&self) -> StringIndex { self.path }
}


#[derive(Debug)]
pub struct NamespaceMap<'out> {
    map: KVec<NamespaceId, Option<Namespace<'out>>>,
    type_to_ns: HashMap<Type, NamespaceId>,
    arena: &'out Arena,
}


impl<'out> NamespaceMap<'out> {
    pub fn new(arena: &'out Arena) -> Self {
        Self {
            map: KVec::new(),
            type_to_ns: HashMap::new(),
            arena,
        }
    }


    #[inline(always)]
    pub fn get_type(&mut self, id: Type, types: &TypeMap) -> NamespaceId {
        let id = self.type_to_ns.kget_or_insert_with(id, || {
            self.map.push(Some(Namespace::new(self.arena, id.path(types))))
        });

        *id
    }


    #[inline(always)]
    pub fn get_type_mut(&mut self, id: Type, types: &TypeMap) -> &mut Namespace<'out> {
        let id = self.type_to_ns.kget_or_insert_with(id, || {
            self.map.push(Some(Namespace::new(self.arena, id.path(types))))
        });

        self.map[*id].as_mut().unwrap()
    }


    #[inline(always)]
    pub fn get(&self, id: NamespaceId) -> Option<&Namespace<'out>> {
        self.map[id].as_ref()
    }


    #[inline(always)]
    pub fn get_mut(&mut self, id: NamespaceId) -> Option<&mut Namespace<'out>> {
        self.map[id].as_mut()
    }


    #[inline(always)]
    pub fn put(&mut self, ns: Namespace<'out>) -> NamespaceId {
        self.map.push(Some(ns))
    }


    pub fn error(&mut self, ns: NamespaceId) {
        self.map[ns] = None;
    }


    #[inline(always)]
    pub fn map_type(&mut self, ty: Type, ns: NamespaceId) {
        self.type_to_ns.insert(ty, ns);
    }
}
