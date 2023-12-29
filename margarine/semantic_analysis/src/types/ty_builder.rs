use common::{string_map::{StringIndex, StringMap}, source::SourceRange};
use errors::SemaError;
use sti::{vec::Vec, hash::{HashMap, DefaultSeed}, arena::Arena, traits::FromIn, keyed::KVec, arena_pool::ArenaPool, alloc::Alloc};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType};

use crate::{errors::Error, namespace::{NamespaceMap, Namespace}, funcs::{FunctionMap, Function}, types::ty_sym::{StructField, TypeSymbolKind, TypeStruct, TypeTaggedUnion, TaggedUnionField}};

use super::{ty::Type, ty_map::{TypeId, TypeMap}, ty_sym::{TypeSymbol, TypeEnum, TypeTag}};

#[derive(Debug)]
pub struct TypeBuilder<'a> {
    storage: &'a Arena,

    types: HashMap<TypeId, PartialType<'a>, DefaultSeed, &'a Arena>,
    process_stack: Vec<StringIndex, &'a Arena>,
}


#[derive(Debug)]
pub enum ProcessingState {
    Uninit,
    Processing,
    Errored,
}


#[derive(Debug)]
pub struct PartialType<'a> {
    name: StringIndex,
    source: SourceRange,
    process_state: ProcessingState,
    kind: Option<PartialTypeKind<'a>>,
}


#[derive(Debug)]
pub enum PartialTypeKind<'a> {
    Struct {
        fields: &'a mut [PartialStructField]
    },

    Enum {
        mappings: &'a mut [PartialEnumField]
    }
}


#[derive(Debug)]
pub struct PartialStructField {
    name: StringIndex,
    ty: Type,
}


#[derive(Debug)]
pub struct PartialEnumField {
    name: StringIndex,
    ty: Option<Type>,
}


impl<'a> TypeBuilder<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            storage: arena,
            types: HashMap::new_in(arena),
            process_stack: Vec::new_in(arena),
        }
    }


    ///
    /// Adds a type to the builder
    ///
    /// # Panics
    /// - If there is already a type with the same `TypeId` in this `TypeBuilder`
    ///
    pub fn add_ty(&mut self, ty: TypeId, name: StringIndex, source: SourceRange) {
        let pty = PartialType {
            name,
            source,
            process_state: ProcessingState::Uninit,
            kind: None,
        };

        let result = self.types.insert(ty, pty);
        assert!(result.is_none());
    }


    ///
    /// Sets the fields of a partial struct type
    ///
    /// # Panics
    /// - If the partial type with the id `ty` already has it's kind set
    ///
    pub fn set_struct_fields(
        &mut self, 
        ty: TypeId, 
        iter: impl Iterator<Item=(StringIndex, Type)>
    ) {
        let fields = Vec::from_in(
            self.storage, 
            iter.map(|(name, ty)| PartialStructField { name, ty })
        );
        
        let fields = fields.leak();

        self.set_kind(ty, PartialTypeKind::Struct { fields })
    }


    ///
    /// Sets the fields of a partial enum type
    ///
    /// # Panics
    /// - If the partial type with the id `ty` already has it's kind set
    ///
    pub fn set_enum_fields(
        &mut self, 
        ty: TypeId, 
        iter: impl Iterator<Item=(StringIndex, Option<Type>)>
    ) {
        let mappings = Vec::from_in(
            self.storage, 
            iter.map(|(name, ty)| PartialEnumField { name, ty })
        );
        
        let mappings = mappings.leak();

        self.set_kind(ty, PartialTypeKind::Enum { mappings })
    }
}


impl<'a> TypeBuilder<'a> {
    #[inline(always)]
    fn set_kind(&mut self, ty: TypeId, kind: PartialTypeKind<'a>) {
        let ty = self.types.get_mut(&ty).unwrap();
        assert!(ty.kind.is_none());
        ty.kind.replace(kind);
    }
}


pub struct TypeBuilderData<'me, 'out, 'str> {
    arena: &'out Arena,
    type_map: &'me mut TypeMap<'out>,
    namespace_map: &'me mut NamespaceMap,
    function_map: &'me mut FunctionMap<'out>,
    module_builder: &'me mut WasmModuleBuilder<'out, 'str>,
}


impl<'out> TypeBuilder<'_> {
    pub fn finalise(
        mut self,
        mut data: TypeBuilderData<'_, 'out, '_>,
        errors: &mut KVec<SemaError, Error>,
    ) {
        let pool = ArenaPool::tls_get_temp();
        let mut vec = Vec::with_cap_in(&*pool, self.types.len());
        self.types.iter().for_each(|x| vec.push(*x.0));

        for ty in &vec {
            if let Err(err) = self.resolve_type(&mut data, *ty) {
                errors.push(err);
            }
        }
    }


    fn resolve_type(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: TypeId,
    ) -> Result<TypeSymbol<'out>, Error> {
        if let Some(v) = data.type_map.get_opt(ty) {
            return Ok(v);
        }

        let pty = self.types.get_mut(&ty).unwrap();
        let PartialType { name, process_state, kind, .. } = match pty.process_state {
            ProcessingState::Uninit => pty,


            ProcessingState::Processing => {
                pty.process_state = ProcessingState::Errored;

                let backtrace = self.process_stack.iter().enumerate()
                                .find(|x| *x.1 == pty.name).unwrap().0;

                let backtrace = self.process_stack[backtrace..].to_vec();

                return Err(Error::CyclicType {
                    source: pty.source,
                    backtrace, 
                    name: pty.name,
                })
            },


            ProcessingState::Errored => return Err(Error::Bypass),
        };

        *process_state = ProcessingState::Processing;
        self.process_stack.push(*name);

        let ret = {
            let name = *name;
            let kind = kind.take().unwrap();

            match kind {
                PartialTypeKind::Struct { fields } => 
                    self.process_struct(data, fields, name),


                PartialTypeKind::Enum { mappings } => { 
                    let sym = self.process_enum(data, mappings, name);

                    sym
                },
            }
        };

        self.process_stack.pop();

        let sym = match ret {
            Ok(v) => {
                data.type_map.put(ty, v);
                v
            },

            Err(e) => { 
                let ty = self.types.get_mut(&ty).unwrap();
                ty.process_state = ProcessingState::Errored;
                return Err(e)
            }
        };

        
        if let TypeSymbolKind::Enum(val) = sym.kind() {
            self.register_enum_methods(data, ty, val);
        }


        Ok(sym)
    }


    ///
    /// Processes and generates a struct type
    ///
    /// This function does **NOT** register the
    /// type into the type map
    ///
    /// # Errors
    /// - If any of the type's fields are cyclic
    ///
    #[must_use]
    fn process_struct(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        fields: &[PartialStructField],
        name: StringIndex,
    ) -> Result<TypeSymbol<'out>, Error> {
        let mut align = 1;
        let mut cursor = 0;
        let mut new_fields = Vec::with_cap_in(data.arena, fields.len());

        for f in fields.iter() {
            let f_align = self.align(data, f.ty)?;

            // Calculate the alignment of the type
            align = align.max(f_align);

            // Calculate the size of the type with field offset
            cursor = sti::num::ceil_to_multiple_pow2(cursor, f_align);

            let offset = cursor;
            cursor += self.size(data, f.ty).unwrap();

            // New field
            let field = StructField::new(f.name, f.ty, offset);
            new_fields.push(field);
        }

        let align = align;
        let size = sti::num::ceil_to_multiple_pow2(cursor, align);
        let fields : &[_] = new_fields.leak();

        // Finalise
        let kind = TypeStruct::new(fields);
        let kind = TypeSymbolKind::Struct(kind);
        let symbol = TypeSymbol::new(name, align, size, kind);

        Ok(symbol)
    }

    ///
    /// Processes and generates an enum type
    ///
    /// This function does **NOT** register the
    /// type into the type map
    ///
    /// # Errors
    /// - If any of the type's fields are cyclic
    ///
    #[must_use]
    fn process_enum(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        fields: &[PartialEnumField],
        name: StringIndex,
    ) -> Result<TypeSymbol<'out>, Error> {
        // Tag

        // TODO: Don't assume u32
        /*
        let tag_size = {
            let mut c = fields.len() as f64;
            let mut tag_size = 0;
            while c > 1.0 {
                c /= 256.0;
                tag_size += 1;
            }

            tag_size
        };
        */

        assert!(fields.len() < u64::MAX as usize, "enums with more than u32::MAX variants are not yet supported");
        let tag_align = 4;
        let tag_size = 4;

        // Union
        let mut union_align = 1;
        let mut union_size = 0;
        let mut new_fields = Vec::with_cap_in(data.arena, fields.len());

        for f in fields.iter() {
            let field = TaggedUnionField::new(f.name, f.ty);
            new_fields.push(field);

            let Some(fty) = f.ty
            else { continue; };

            let f_align = self.align(data, fty)?;
            let f_size = self.size(data, fty)?;

            union_align = union_align.max(f_align);
            union_size = union_size.max(f_size);


        }

        if new_fields.iter().all(|x| x.ty() == None){
            let mut size = 0;
            size = sti::num::ceil_to_multiple_pow2(size, tag_align);
            size += tag_align;
            size = sti::num::ceil_to_multiple_pow2(size, tag_align);
            return Ok(TypeSymbol::new(
                    name, tag_align,
                    size, TypeSymbolKind::Enum(TypeEnum::Tag(TypeTag::new(
                            Vec::from_in(data.arena, fields.iter().map(|x| x.name)).leak())))
                    ));
        }

        // Smush 'Em Together
        let mut align = 1;
        let mut size = 0;

        // Tag
        {
            align = align.max(tag_align);
            size = sti::num::ceil_to_multiple_pow2(size, tag_align);
            size += tag_size;
        }

        let union_offset;
        // Union
        {
            align = align.max(union_align);
            size = sti::num::ceil_to_multiple_pow2(size, union_align);
            union_offset = size;
            size += union_size;
        }

        let align = align;
        let size = sti::num::ceil_to_multiple_pow2(size, align);
        let fields = new_fields.leak();
        
        // Finalise
        let kind = TypeEnum::TaggedUnion(TypeTaggedUnion::new(union_offset.try_into().unwrap(), fields));
        Ok(TypeSymbol::new(name, align, size, TypeSymbolKind::Enum(kind)))
    } 

    
    fn register_enum_methods(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: TypeId,
        kind: TypeEnum,
    ) {
        let mut ns = Namespace::new();
        
        match kind {
            TypeEnum::TaggedUnion(sym) => {
                let tysym = data.type_map.get(ty);
                let wasm_ty = WasmType::Ptr { size: tysym.size() };

                for (i, f) in sym.fields().into_iter().enumerate() {
                    let wfid = data.module_builder.function_id();
                    let mut wf = WasmFunctionBuilder::new(data.arena, wfid);
                    let alloc = wf.return_value(wasm_ty);

                    wf.i32_const(i as i32);
                    wf.sptr_const(alloc);
                    wf.i32_write();


                    let func;
                    if let Some(fty) = f.ty() {
                        let wfty = fty.to_wasm_ty(data.type_map);
                        let param = wf.param(wfty);
                        let union_ptr = alloc.add(sym.union_offset().try_into().unwrap());

                        wf.local_get(param);
                        wf.sptr_const(union_ptr);
                        wf.write(wfty);

                        func = Function::new(
                            f.name(),
                            data.arena.alloc_new([(StringMap::VALUE, false, fty)]),
                            Type::Custom(ty), wfid, None,
                        );

                    } else {
                        func = Function::new(f.name(), &[], Type::Custom(ty), wfid, None);
                    }

                    data.module_builder.register(wf);
                    
                    let func = data.function_map.put(func);
                    ns.add_func(f.name(), func);
                }
            },


            TypeEnum::Tag(sym) => {
                for (i, f) in sym.fields().into_iter().enumerate() {
                    let wfid = data.module_builder.function_id();
                    let mut wf = WasmFunctionBuilder::new(data.arena, wfid);
                    wf.return_value(wasm::WasmType::I32);

                    wf.i32_const(i as i32);

                    data.module_builder.register(wf);
                    
                    let func = Function::new(*f, &[], Type::Custom(ty), wfid, None);
                    let func = data.function_map.put(func);
                    ns.add_func(*f, func);
                }
            },
        }


        let ns = data.namespace_map.put(ns);
        data.namespace_map.map_type(Type::Custom(ty), ns);
    }



    fn align(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: Type
    ) -> Result<usize, Error> {
        Ok(match ty {
            Type::I64 => 8,
            Type::I32 => 4,
            Type::F64 => 8,
            Type::Any   => todo!(),
            Type::Unit  => 1,
            Type::Never => todo!(),
            Type::Error => 1,
            Type::Custom(v) => self.resolve_type(data, v)?.align(),
        })
    }


    fn size(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: Type
    ) -> Result<usize, Error> {
        Ok(match ty {
            Type::I64   => 8,
            Type::I32   => 4,
            Type::F64   => 8,
            Type::Any   => todo!(),
            Type::Unit  => 1,
            Type::Never => todo!(),
            Type::Error => 0,
            Type::Custom(v) => self.resolve_type(data, v)?.size(),
        })
    }
}


impl<'me, 'out, 'str> TypeBuilderData<'me, 'out, 'str> {
    pub fn new(
        type_map: &'me mut TypeMap<'out>, 
        namespace_map: &'me mut NamespaceMap, 
        function_map: &'me mut FunctionMap<'out>, 
        module_builder: &'me mut WasmModuleBuilder<'out, 'str>
    ) -> Self {
        Self { arena: module_builder.arena, type_map, namespace_map, function_map, module_builder }
    }
}

