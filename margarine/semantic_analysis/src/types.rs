use std::{cell::Cell, marker::PhantomData, backtrace, mem::{align_of, size_of}};

use common::{string_map::{StringMap, StringIndex}, source::SourceRange};
use errors::{ErrorId, SemaError};
use parser::DataType;
use polonius_the_crab::{polonius, polonius_return};
use sti::{define_key, vec::Vec, hash::{HashMap, DefaultSeed}, keyed::{KVec, KSlice}, traits::MapIt, prelude::Arena, arena_pool::ArenaPool, alloc::GlobalAlloc};
use wasm::{WasmType, WasmFunctionBuilder, WasmModuleBuilder};

use crate::{namespace::{Namespace, NamespaceId, NamespaceMap}, errors::Error, funcs::{FunctionMap, Function}};

define_key!(u32, pub TypeId);

#[derive(Hash, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Int,
    UInt,
    Float,
    Any,
    Unit,
    Never,
    Error,
    Custom(TypeId),
}


impl Type {
    pub const BOOL : Type = Type::Custom(TypeId(0));

    pub fn display<'a>(
        self,
        string_map: &'a StringMap<'a>,
        types: &TypeMap,
    ) -> &'a str {
        match self {
            Type::Int => "int",
            Type::UInt => "uint",
            Type::Float => "float",
            Type::Any => "any",
            Type::Unit => "unit",
            Type::Never => "never",
            Type::Error => "error",
            Type::Custom(t) => string_map.get(types.get(t).display_name),
        }
    }


    pub const fn id(self) -> TypeId {
        match self {
            Type::Int => todo!(),
            Type::UInt => todo!(),
            Type::Float => todo!(),
            Type::Any => todo!(),
            Type::Unit => todo!(),
            Type::Never => todo!(),
            Type::Error => todo!(),
            Type::Custom(v) => v,
        }
    }


    pub fn to_wasm_ty(self, map: &TypeMap) -> WasmType {
        match self {
            Type::Int => WasmType::I64,
            Type::UInt => WasmType::I64,
            Type::Float => WasmType::F64,
            Type::Unit => WasmType::I64,
            Type::Never => WasmType::I64,

            // Pointers
            | Type::Error
            | Type::Any => WasmType::I64,


            Type::Custom(v) => WasmType::Ptr(map.get(v).size),
        }
    }


    pub fn eq_lit(self, ty: Type) -> bool {
        match (self, ty) {
            | (Type::Int, Type::Int) 
            | (Type::UInt, Type::UInt) 
            | (Type::Float, Type::Float) 
            | (Type::Any, Type::Any) 
            | (Type::Unit, Type::Unit) 
            | (Type::Never, Type::Never) 
            | (Type::Error, Type::Error) 
             => true,

            (Type::Custom(v1), Type::Custom(v2)) => v1 == v2,

            _ => false,
        }
    }


    pub fn eq_sem(self, ty: Type) -> bool {
        match (self, ty) {
            | (Type::Error, _) | (_, Type::Error)
            | (Type::Never, _) | (_, Type::Never)
             => true,

            | (Type::Int, Type::Int) 
            | (Type::UInt, Type::UInt) 
            | (Type::Float, Type::Float) 
            | (Type::Any, Type::Any) 
            | (Type::Unit, Type::Unit) 
             => true,

            (Type::Custom(v1), Type::Custom(v2)) => v1 == v2,

            _ => false,
        }
    }


    pub fn is_number(self) -> bool {
        self.eq_sem(Type::Int)
        || self.eq_sem(Type::Float)
        || self.eq_sem(Type::UInt)
    }
}


#[derive(Debug, Clone, Copy)]
pub struct TypeSymbol<'a> {
    display_name: StringIndex,
    
    align: usize,
    size : usize,

    kind: TypeSymbolKind<'a>,
}


#[derive(Debug, Clone, Copy)]
pub enum TypeSymbolKind<'a> {
    Struct(TypeStruct<'a>),
    Enum(TypeEnum<'a>),
    Union(&'a [Field]),

}


#[derive(Debug, Clone, Copy)]
pub struct TypeStruct<'a> {
    fields: &'a [Field],
}


#[derive(Debug, Clone, Copy)]
pub struct TypeEnum<'a> {
    fields: &'a [Field],
}


#[derive(Debug)]
pub struct TypeMap<'a> {
    map: KVec<TypeId, Option<TypeSymbol<'a>>>,
}


impl<'a> TypeMap<'a> {
    pub fn new(string_map: &mut StringMap) -> Self {
        let mut map = KVec::new();

        map.push(Some(TypeSymbol {
            display_name: string_map.insert("bool"),
            align: 4, 
            size: 4,
            kind: TypeSymbolKind::Enum(TypeEnum { fields: &[] }),
        }
        ));

        Self { map }
    }


    ///
    /// Reserves a spot in the type map and
    /// gives a `TypeId` to that spot
    /// 
    #[inline(always)]
    pub fn pending(&mut self) -> TypeId {
        self.map.push(None)
    }


    ///
    /// Initialises a pending type
    ///
    /// # Panics
    /// - If the given type is already initialised
    /// - If the `TypeId` is out of bounds
    ///
    #[inline(always)]
    pub fn initialise(&mut self, ty: TypeId, sym: TypeSymbol<'a>) {
        assert!(self.map[ty].replace(sym).is_none(), "type is already initialised")
    }


    pub fn get(&self, val: TypeId) -> TypeSymbol<'a> {
        self.get_opt(val).unwrap()
    }


    pub fn get_opt(&self, val: TypeId) -> Option<TypeSymbol<'a>> {
        self.map[val]
    }


    pub fn align(&self, ty: Type) -> Option<usize> {
        Some(match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => 16,
            Type::Never => 0,
            Type::Error => 0,
            Type::Unit => 1,
            Type::Custom(v) => self.get_opt(v)?.align,
        })
    }


    pub fn size(&self, ty: Type) -> Option<usize> {
        Some(match ty {
            Type::Int => 8,
            Type::UInt => 8,
            Type::Float => 8,
            Type::Any => 16,
            Type::Unit => 1,
            Type::Never => 0,
            Type::Error => 0,
            Type::Custom(v) => self.get_opt(v)?.size,
        })
    }
}


#[derive(Debug, Clone, Copy)]
pub struct Field {
    name: StringIndex,
    ty: Type,
    offset: usize,
}

impl Field {
    pub fn new(name: StringIndex, ty: Type, offset: usize) -> Self { Self { name, ty, offset } }
}


#[derive(Debug)]
pub struct TypeBuilder<'storage> {
    types: HashMap<TypeId, PartialType<'storage>, DefaultSeed, &'storage Arena>,
    storage: &'storage Arena,
    processing: Vec<StringIndex, &'storage Arena>,
}


pub struct TypeBuilderData<'me, 'out> {
    arena: &'out Arena,
    type_map: &'me mut TypeMap<'out>,
    namespace_map: &'me mut NamespaceMap,
    function_map: &'me mut FunctionMap<'out>,
    module_builder: &'me mut WasmModuleBuilder<'out>,
}

impl<'me, 'out> TypeBuilderData<'me, 'out> {
    pub fn new(
        type_map: &'me mut TypeMap<'out>, 
        namespace_map: &'me mut NamespaceMap, 
        function_map: &'me mut FunctionMap<'out>, 
        module_builder: &'me mut WasmModuleBuilder<'out>
    ) -> Self { 
        Self { 
            arena: module_builder.arena,
            type_map, 
            namespace_map, 
            function_map, 
            module_builder 
        } 
    }
}


#[derive(Debug)]
struct PartialType<'a> {
    name: StringIndex,
    fields: Option<&'a mut [FieldBlueprint]>,
    state: PartialTypeState,
    source: SourceRange,
    is_enum: bool,
}


#[derive(Debug)]
enum PartialTypeState {
    Uninit,
    Processing,
    Errored,
}


#[derive(Debug)]
pub struct FieldBlueprint {
    name: StringIndex,
    ty: Type,
}

impl FieldBlueprint {
    pub fn new(name: StringIndex, ty: Type) -> Self { Self { name, ty } }
}

impl<'storage> TypeBuilder<'storage> {
    pub fn new(storage: &'storage Arena) -> Self {
        Self {
            types: HashMap::new_in(storage),
            processing: Vec::new_in(storage),
            storage,
        }
    }


    pub fn add_ty(&mut self, ty: TypeId, name: StringIndex, source: SourceRange, is_enum: bool) {
        self.types.insert(ty, PartialType { name, fields: None, state: PartialTypeState::Uninit, source, is_enum });
    }


    pub fn add_fields(&mut self, ty: TypeId, new_fields: &'storage mut [FieldBlueprint]) {
        let PartialType { fields, .. } = self.types.get_mut(&ty).unwrap();

        assert!(fields.is_none());
        fields.replace(new_fields);
    }


    pub fn finalise<'a>(
        mut self, 
        out: &'a Arena, 
        data: &mut TypeBuilderData<'_, 'a>,
        errors: &mut KVec<SemaError, Error>,
    ) {

        let pool = ArenaPool::tls_get_temp();
        let mut vec = Vec::with_cap_in(&*pool, self.types.len());
        self.types.iter().for_each(|x| vec.push(*x.0));

        for ty in &vec {
            if let Err(err) = self.resolve_type(out, data, *ty) {
                errors.push(err);
            }
        }
    }


    pub fn alloc(&self) -> &'storage Arena {
        self.storage
    }
}


impl<'storage> TypeBuilder<'storage> {
    fn resolve_type<'a, 'b>(
        &mut self, 
        out: &'a Arena, 
        mut data: &mut TypeBuilderData<'_, 'a>,
        ty: TypeId
    ) -> Result<TypeSymbol<'a>, Error> {
        polonius! {
            |data| -> Result<TypeSymbol<'a>, Error> {
                if let Some(v) = data.type_map.get_opt(ty) {
                    polonius_return!(Ok(v));
                }
            }
        }

        let partial_ty = self.types.get_mut(&ty).unwrap();
        let PartialType { fields, name, state, is_enum, .. } = match partial_ty.state {
            PartialTypeState::Uninit => partial_ty,

            PartialTypeState::Processing => {
                partial_ty.state = PartialTypeState::Errored;
                
                let backtrace = self.processing.iter().enumerate()
                    .find(|x| *x.1 == partial_ty.name).unwrap().0;

                let backtrace = self.processing[backtrace..].to_vec();

                return Err(Error::CyclicType {
                    source: partial_ty.source,
                    backtrace, 
                    name: partial_ty.name,
                })
            },

            PartialTypeState::Errored => return Err(Error::Bypass),
        };

        *state = PartialTypeState::Processing;
        let fields = fields.take().expect("fields are not initialised");
        let name = *name;
        let is_enum = *is_enum;

        self.processing.push(name);

        let ret = match is_enum {
            true  => self.process_enum(out, data, fields, name, ty),
            false => self.process_struct(out, data, fields, name, ty),
        };

        self.processing.pop();

        if let Err(e) = ret {
            self.types.get_mut(&ty).unwrap().state = PartialTypeState::Errored;
            return Err(e);
        }

        Ok(data.type_map.get(ty))
    }


    fn align<'a>(
        &mut self, out: &'a Arena,
        data: &mut TypeBuilderData<'_, 'a>,
        ty: Type,
    ) -> Result<usize, Error> {
        if let Some(v) = data.type_map.align(ty) {
            return Ok(v)
        }

        let ty_id = match ty {
            Type::Custom(v) => v,
            _ => unreachable!()
        };

        Ok(self.resolve_type(out, data, ty_id)?.align)
    }


    fn size<'a>(
        &mut self, 
        out: &'a Arena, 
        data: &mut TypeBuilderData<'_, 'a>,
        ty: Type
    ) -> Result<usize, Error> {
        if let Some(v) = data.type_map.size(ty) {
            return Ok(v)
        }

        let ty_id = match ty {
            Type::Custom(v) => v,
            _ => unreachable!()
        };

        Ok(self.resolve_type(out, data, ty_id)?.size)
    }

}


impl TypeBuilder<'_> {
    fn process_struct<'a>(
        &mut self, out: &'a Arena, 
        data: &mut TypeBuilderData<'_, 'a>,
        fields: &mut [FieldBlueprint], name: StringIndex,
        ty: TypeId
    ) -> Result<(), Error> { 
        let align = {
            let mut max = 1;
            for f in fields.iter() {
                let align = self.align(out, data, f.ty)?;
                if align > max {
                    max = align;
                }
            }
            max
        };

        let mut cursor = 0;
        let mut new_fields = Vec::with_cap_in(out, fields.len());

        for field in fields.iter_mut() {
            let align = self.align(out, data, field.ty)?;
            cursor = sti::num::ceil_to_multiple_pow2(cursor, align);

            let offset = cursor;
            cursor += self.size(out, data, field.ty)?;

            new_fields.push(Field::new(field.name, field.ty, offset));
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);

        let symbol = TypeSymbol {
            display_name: name,
            align,
            size,
            kind: TypeSymbolKind::Struct(TypeStruct { fields: new_fields.leak() }),
        };

        data.type_map.initialise(ty, symbol);
        Ok(())
    }


    fn process_enum<'a>(
        &mut self, out: &'a Arena,
        data: &mut TypeBuilderData<'_, 'a>,
        fields: &mut [FieldBlueprint], name: StringIndex,
        ty: TypeId
    ) -> Result<(), Error> { 
        // @TEMP: Let's assume the tag is always a u64
        let align = {
            let mut max = 8;
            for f in fields.iter() {
                let align = self.align(out, data, f.ty)?;
                if align > max {
                    max = align;
                }
            }
            max
        };

        let mut cursor = 8;
        let mut new_fields = Vec::with_cap_in(out, fields.len());

        for field in fields.iter_mut() {
            let align = self.align(out, data, field.ty)?;
            let mut c = sti::num::ceil_to_multiple_pow2(4, align);

            let offset = c;

            c += self.size(out, data, field.ty)?;

            cursor = cursor.max(c);

            new_fields.push(Field::new(field.name, field.ty, offset));
        }

        let size = sti::num::ceil_to_multiple_pow2(cursor, align);
        let fields = new_fields.leak();

        let symbol = TypeSymbol {
            display_name: name,
            align,
            size,
            kind: TypeSymbolKind::Enum(TypeEnum { fields}),
        };

        data.type_map.initialise(ty, symbol);

        {
            let mut ns = Namespace::new();

            for (i, f) in fields.iter().enumerate() {
                let wasm_id = data.module_builder.function_id();
                let mut wasm_func = WasmFunctionBuilder::new(data.module_builder.arena, wasm_id);

                let return_ptr = wasm_func.return_value(WasmType::Ptr(size));
                let field_wasm_ty = f.ty.to_wasm_ty(&data.type_map);
                let param = wasm_func.param(field_wasm_ty);

                wasm_func.i64_const(i as i64);
                wasm_func.write_i64_to_stack(return_ptr);
                wasm_func.local_get(param);
                wasm_func.copy_to_stack(return_ptr, field_wasm_ty);

                data.module_builder.register(wasm_func);

                let function = Function::new(
                    f.name, 
                    data.arena.alloc_new([(StringMap::VALUE, false, f.ty)]), 
                    Type::Custom(ty), 
                    wasm_id
                );


                let function_id = data.function_map.put(function);
                ns.add_func(f.name, function_id);
            }

            data.namespace_map.put(ns);
            
        }

        Ok(())
    }





}
