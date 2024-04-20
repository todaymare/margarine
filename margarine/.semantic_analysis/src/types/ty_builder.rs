use common::{string_map::{StringIndex, StringMap}, source::SourceRange};
use errors::SemaError;
use sti::{vec::Vec, hash::{HashMap, DefaultSeed}, arena::Arena, traits::FromIn, keyed::KVec, arena_pool::ArenaPool};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType};

use crate::{concat_path, errors::Error, funcs::{Func, FunctionKind, FunctionMap}, namespace::{Namespace, NamespaceMap}, types::ty_sym::{StructField, TaggedUnionField, TypeEnumKind, TypeKind, TypeStruct, TypeTaggedUnion}};

use super::{ty::Type, ty_map::{TypeId, TypeMap}, ty_sym::{TypeEnum, TypeEnumStatus, TypeStructStatus, TypeSymbol, TypeTag}};

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
        fields: &'a mut [PartialStructField],
        status: TypeStructStatus,
    },

    Enum {
        mappings: &'a mut [PartialEnumField],
        status: TypeEnumStatus,
    },

    Errored,
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
        iter: &[(StringIndex, Type)],
        status: TypeStructStatus,
    ) {
        let fields = Vec::from_in(
            self.storage, 
            iter.iter().map(|(name, ty)| PartialStructField { name: *name, ty: *ty })
        );
        
        let fields = fields.leak();

        self.set_kind(ty, PartialTypeKind::Struct { fields, status })
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
        iter: impl Iterator<Item=(StringIndex, Option<Type>)>,
        status: TypeEnumStatus,
    ) {
        let mappings = Vec::from_in(
            self.storage, 
            iter.map(|(name, ty)| PartialEnumField { name, ty })
        );
        
        let mappings = mappings.leak();

        self.set_kind(ty, PartialTypeKind::Enum { mappings, status })
    }


    pub fn errored(&mut self, name: StringIndex) {
        for ty in self.types.iter_mut() {
            if ty.1.name == name {
                ty.1.kind = Some(PartialTypeKind::Errored);
                return
            }
        }
        panic!();
    }
}


impl<'a> TypeBuilder<'a> {
    #[inline(always)]
    fn set_kind(&mut self, ty: TypeId, kind: PartialTypeKind<'a>) {
        let ty = self.types.get_mut(&ty).unwrap();
        assert!(ty.kind.is_none() ||
                matches!(ty.kind.as_ref().unwrap(), PartialTypeKind::Errored),
                "{:?}\n{kind:?}", ty.kind);

        if matches!(ty.kind.as_ref(), Some(PartialTypeKind::Errored)) { return }

        ty.kind.replace(kind);
    }
}


#[derive(Debug)]
pub struct TypeBuilderData<'me, 'out, 'str> {
    arena: &'out Arena,
    type_map: &'me mut TypeMap<'out>,
    namespace_map: &'me mut NamespaceMap<'out>,
    function_map: &'me mut FunctionMap<'out>,
    module_builder: &'me mut WasmModuleBuilder<'out, 'str>,
    string_map: &'me mut StringMap<'str>,
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

        for ty in &vec {
            let sym = data.type_map.get(*ty);

            if let TypeKind::Struct(val) = sym.kind() {
                if val.status == TypeStructStatus::Ptr {
                    self.register_ptr_methods(&mut data, *ty, sym.path(), sym, val);
                }
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

            let path = data.type_map.path(ty);
            match kind {
                PartialTypeKind::Struct { fields, status } => {
                    self.process_struct(data, fields, name, status, path)
                }


                PartialTypeKind::Enum { mappings, status } => { 
                    self.process_enum(data, mappings, status, name, path)
                },

                PartialTypeKind::Errored => Ok(TypeSymbol::new(name, path, 1, 0, TypeKind::Error)),
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

        
        if let TypeKind::Enum(val) = sym.kind() {
            self.register_enum_methods(data, ty, sym.path(), val);
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
        status: TypeStructStatus,
        path: StringIndex,
    ) -> Result<TypeSymbol<'out>, Error> {
        let mut align = 1;
        let mut cursor = 0;
        let mut new_fields = Vec::with_cap_in(data.arena, fields.len());

        for f in fields.iter() {
            let f_align = if status == TypeStructStatus::Ptr { 16 }
                          else { self.align(data, f.ty)? };

            // Calculate the alignment of the type
            align = align.max(f_align);

            // Calculate the size of the type with field offset
            cursor = sti::num::ceil_to_multiple_pow2(cursor, f_align);

            let offset = cursor;
            cursor += if status == TypeStructStatus::Ptr { 16 }
                      else { self.size(data, f.ty).unwrap() };

            // New field
            let field = StructField::new(f.name, f.ty);
            new_fields.push((field, offset));
        }

        let align = align;
        let size = sti::num::ceil_to_multiple_pow2(cursor, align);
        let fields : &[_] = new_fields.leak();

        // Finalise
        let kind = TypeStruct::new(fields, status);
        let kind = TypeKind::Struct(kind);
        let symbol = TypeSymbol::new(name, path, align, size, kind);

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
        status: TypeEnumStatus,
        name: StringIndex,
        path: StringIndex,
    ) -> Result<TypeSymbol<'out>, Error> {
        // Tag

        // TODO: Don't assume u32

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

            let kind = TypeTag::new(Vec::from_in(data.arena, fields.iter().map(|x| x.name)).leak());
            let kind = TypeEnum::new(TypeEnumStatus::User, TypeEnumKind::Tag(kind));

            return Ok(TypeSymbol::new(name, path, tag_align, size, TypeKind::Enum(kind)))
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
        let kind = TypeEnum::new(status, TypeEnumKind::TaggedUnion(TypeTaggedUnion::new(union_offset.try_into().unwrap(), fields)));
        let kind = TypeKind::Enum(kind);
        Ok(TypeSymbol::new(name, path, align, size, kind))
    } 

    
    fn register_enum_methods(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: TypeId,
        path: StringIndex,
        kind: TypeEnum,
    ) {
        let ns = data.namespace_map.get_type_mut(Type::Custom(ty), &mut data.type_map);
        
        match kind.kind() {
            TypeEnumKind::TaggedUnion(sym) => {
                let tysym = data.type_map.get(ty);
                let wasm_ty = WasmType::Ptr { size: tysym.size() };

                for (i, f) in sym.fields().into_iter().enumerate() {
                    let wfid = data.module_builder.function_id();
                    let mut wf = WasmFunctionBuilder::new(data.arena, wfid);
                    let alloc = wf.return_value(wasm_ty);

                    wf.u32_const(i as u32);
                    wf.sptr_const(alloc);
                    wf.i32_write();

                    let func;
                    let path = concat_path(data.arena, data.string_map, path, f.name());
                    wf.export(path);

                    if let Some(fty) = f.ty() {
                        let wfty = fty.to_wasm_ty(data.type_map);
                        let param = wf.param(wfty);
                        let union_ptr = alloc.add(sym.union_offset().try_into().unwrap());

                        wf.local_get(param);
                        wf.sptr_const(union_ptr);
                        wf.write(wfty);


                        func = Func::new(
                            f.name(),
                            path,
                            data.arena.alloc_new([(StringMap::VALUE, false, fty)]),
                            Type::Custom(ty), wfid,
                            FunctionKind::UserDefined { inout: None },
                        );

                    } else {
                        func = Func::new(f.name(), path, &[], Type::Custom(ty), wfid,
                                FunctionKind::UserDefined { inout: None });
                    }


                    if wasm_ty.stack_size() != 0 {
                        wf.param(WasmType::I32);
                    }

                    wf.sptr_const(alloc);
                    data.module_builder.register(wf);
                    
                    let func_id = data.function_map.pending();
                    data.function_map.put(func_id, func);
                    ns.add_func(f.name(), func_id);
                }
            },


            TypeEnumKind::Tag(sym) => {
                for (i, f) in sym.fields().into_iter().enumerate() {
                    let path = concat_path(data.arena, data.string_map, path, *f);
                    let wfid = data.module_builder.function_id();
                    let mut wf = WasmFunctionBuilder::new(data.arena, wfid);
                    wf.export(path);

                    wf.return_value(wasm::WasmType::I32);

                    wf.u32_const(i as u32);

                    data.module_builder.register(wf);
                    
                    let func = Func::new(*f, path, &[], Type::Custom(ty), wfid,
                        FunctionKind::UserDefined { inout: None });

                    let func_id = data.function_map.pending();
                    data.function_map.put(func_id, func);
                    ns.add_func(*f, func_id);
                }
            },
        }

    }


    fn register_ptr_methods(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        rc_ty: TypeId,
        path: StringIndex,
        rc_sym: TypeSymbol,
        rc_kind: TypeStruct,
    ) {
        let ns = data.namespace_map.get_type_mut(Type::Custom(rc_ty), &mut data.type_map);

        assert_eq!(rc_kind.status, TypeStructStatus::Ptr);

        let path_new = concat_path(data.arena, data.string_map, path, StringMap::NEW);
        let path_count = concat_path(data.arena, data.string_map, path, StringMap::COUNT);

        // new function
        // fn count(ty: T): *T
        {
            let wid = data.module_builder.function_id();
            let func = Func::new(
                StringMap::NEW,
                path_new,
                data.arena.alloc_new([
                    (StringMap::VALUE, false, rc_kind.fields[1].0.ty),
                ]), 
                Type::Custom(rc_ty),
                wid,
                FunctionKind::UserDefined { inout: None });

            let func_id = data.function_map.pending();
            data.function_map.put(func_id, func);

            ns.add_func(StringMap::NEW, func_id);

            {
                let mut builder = WasmFunctionBuilder::new(data.arena, wid);
                builder.export(path_new);

                let param = builder.param(rc_kind.fields[1].0.ty.to_wasm_ty(&data.type_map));
                let ret = builder.local(WasmType::I32);
                builder.return_value(WasmType::I32);

                // Allocate enough memory
                builder.malloc(rc_sym.size());
                builder.local_set(ret);

                // Zero the num
                builder.i64_const(0);
                builder.local_get(ret);
                builder.i64_write();

                // Copy the data
                {
                    let ty = rc_kind.fields[1];

                    // src
                    builder.local_get(param);

                    // dst
                    builder.local_get(ret);
                    builder.u32_const(ty.1 as u32);
                    builder.i32_add();

                    builder.write(ty.0.ty.to_wasm_ty(&data.type_map));
                }

                builder.local_get(ret);

                data.module_builder.register(builder);
            }

        }


        // counter function
        // fn count(self): int
        {
            let wid = data.module_builder.function_id();
            let func = Func::new(
                StringMap::COUNT,
                path_count,
                data.arena.alloc_new([
                    (StringMap::VALUE, false, Type::Custom(rc_ty)),
                ]), 
                Type::I64,
                wid,
                FunctionKind::UserDefined { inout: None });

            let func_id = data.function_map.pending();
            data.function_map.put(func_id, func);

            ns.add_func(StringMap::COUNT, func_id);
            {
                let mut builder = WasmFunctionBuilder::new(data.arena, wid);
                builder.export(path_count);

                let param = builder.param(WasmType::I32);
                let local = builder.local(WasmType::I64);
                builder.return_value(WasmType::I64);

                // Read the count 
                builder.local_get(param);
                builder.i64_read();

                builder.i64_const(1);
                builder.i64_sub();

                builder.local_set(local);

                builder.local_get(local);
                builder.local_get(param);
                builder.i64_write();

                builder.local_get(local);

                data.module_builder.register(builder);
            }


        }


    }



    fn align(
        &mut self,
        data: &mut TypeBuilderData<'_, 'out, '_>,
        ty: Type
    ) -> Result<usize, Error> {
        Ok(match ty {
            Type::I8  => 1,
            Type::I16 => 2,
            Type::I32 => 4,
            Type::I64 => 8,
            Type::U8  => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 => 8,
            Type::F32 => 4,
            Type::F64 => 8,

            Type::Unit  => 1,
            Type::Never => 1,
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
            Type::Custom(v) => self.resolve_type(data, v)?.size(),
            _ => ty.size(&data.type_map),
        })
    }
}


impl<'me, 'out, 'str> TypeBuilderData<'me, 'out, 'str> {
    pub fn new(
        type_map: &'me mut TypeMap<'out>, 
        namespace_map: &'me mut NamespaceMap<'out>, 
        function_map: &'me mut FunctionMap<'out>, 
        module_builder: &'me mut WasmModuleBuilder<'out, 'str>,
        string_map: &'me mut StringMap<'str>,
    ) -> Self {
        Self { arena: module_builder.arena, type_map, namespace_map, function_map, module_builder, string_map }
    }
}

