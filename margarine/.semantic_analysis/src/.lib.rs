pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use std::{fmt::Write, ops::{Deref, DerefMut}}; 

use common::{source::SourceRange, string_map::{StringIndex, StringMap}};
use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Func, FunctionKind, FuncId};
use llvm_api::{tys::IsType, values::Value, Context, Function, Module};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{nodes::{decl::{Declaration, DeclarationNode, FunctionSignature, UseItem, UseItemKind}, expr::{BinaryOperator, Expression, ExpressionNode, MatchMapping, UnaryOperator}, stmt::{Statement, StatementNode}, Node}, Block, DataType, DataTypeKind};
use scope::{ExplicitNamespace, FunctionDefinitionScope, LoopScope, Scope, ScopeId, ScopeKind, ScopeMap, VariableScope};
use types::{ty::Type, ty_map::TypeMap, ty_sym::{TypeEnum, TypeKind, TypeEnumKind, TypeEnumStatus, TypeStructStatus}};
use sti::{arena_pool::ArenaPool, format_in, hash::HashMap, keyed::KVec, packed_option::PackedOption, arena::Arena, string::String, traits::FromIn, vec::Vec};

use crate::types::{ty_builder::{TypeBuilder, TypeBuilderData}, ty_map::TypeId, ty_sym::{TaggedUnionField, TypeStruct}};


#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'str> {
    scopes: ScopeMap,
    namespaces: NamespaceMap<'out>,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,
    output: &'out Arena,
    pub string_map: &'me mut StringMap<'str>,

    pub module: Module,
    pub context: Context,
    pub errors: KVec<SemaError, Error>,
    pub startup_functions: Vec<FuncId>,

    options_map: HashMap<Type, TypeId>,
    results_map: HashMap<(Type, Type), TypeId>,
    tuple_map: HashMap<&'out [Type], TypeId>,
    rc_map: HashMap<Type, TypeId>,
}


#[derive(Debug, Clone, Copy)]
pub struct AnalysisResult {
    ty: Type,
    value: Value,
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(ty: Type, value: Value, is_mut: bool) -> Self { Self { ty, is_mut, value } }
    pub fn error(builder: &mut Builder) -> Self { Self::new(Type::Error, builder.unit(), true) }
}


impl<'out> Analyzer<'_, 'out, '_> {
     pub fn convert_ty(&mut self, scope: ScopeId, dt: DataType) -> Result<Type, Error> {
         let pool = ArenaPool::tls_get_rec();
         let mut tyb = TypeBuilder::new(&*pool);
         let ty = self.convert_ty_ex(&mut tyb, scope, dt)?;

        let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces,
                                        &mut self.funcs, &mut self.context,
                                        self.string_map);
        tyb.finalise(data, &mut self.errors);
        Ok(ty)
     }

     pub fn convert_ty_ex(&mut self, builder: &mut TypeBuilder,
                          scope: ScopeId, dt: DataType) -> Result<Type, Error> {
        let ty = match dt.kind() {
            DataTypeKind::I8  => Type::I8,
            DataTypeKind::I16 => Type::I16,
            DataTypeKind::I32 => Type::I32,
            DataTypeKind::I64 => Type::I64,
            DataTypeKind::U8  => Type::I8,
            DataTypeKind::U16 => Type::I16,
            DataTypeKind::U32 => Type::I32,
            DataTypeKind::U64 => Type::I64,
            DataTypeKind::F32 => Type::F32,
            DataTypeKind::F64 => Type::F64,
            DataTypeKind::Bool => Type::BOOL,
            DataTypeKind::Unit => Type::Unit,
            DataTypeKind::Never => Type::Never,
            DataTypeKind::Option(v) => {
                let inner_ty = self.convert_ty(scope, *v)?;
                let tyid = self.make_option(inner_ty, builder);
                Type::Custom(tyid)
            },


            DataTypeKind::Result(v1, v2) => {
                let inner_ty1 = self.convert_ty(scope, *v1)?;
                let inner_ty2 = self.convert_ty(scope, *v2)?;
                Type::Custom(self.make_result(inner_ty1, inner_ty2, builder))
            },


            DataTypeKind::CustomType(v) => {
                if v == StringMap::STR { return Ok(Type::STR) }
                let scope = self.scopes.get(scope);
                let Some(ty) = scope.get_type(v, &self.scopes, &self.namespaces)
                else { return Err(Error::UnknownType(v, dt.range())) };

                ty
            },


            DataTypeKind::Tuple(list) => {
                let pool = ArenaPool::tls_get_rec();
                let mut vec = Vec::with_cap_in(&*pool, list.len());
                for dt in list.iter() {
                    vec.push(self.convert_ty(scope, *dt)?)
                }

                if let Some(ty) = self.tuple_map.get(&*vec) { return Ok(Type::Custom(*ty)) }

                let tyid = self.make_tuple(vec, dt.range());

                Type::Custom(tyid)
            },


            DataTypeKind::Within(ns, dt) => {
                let Some(ns) = self.scopes.get(scope).get_ns(ns, &self.scopes, &mut self.namespaces, &self.types)
                else {
                    return Err(Error::NamespaceNotFound { source: dt.range(), namespace: ns });
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), None.into());
                let scope = self.scopes.push(scope);
                self.convert_ty(scope, *dt)?
            }


            DataTypeKind::Rc(ty) => {
                let ty = self.convert_ty(scope, *ty)?;

                self.make_ptr(ty, builder)
            }
        };
        

        Ok(ty)
    }


    pub fn make_tuple(&mut self, vec: Vec<Type, &Arena>, source: SourceRange) -> TypeId {
        let pool = ArenaPool::tls_get_rec();
        let mut tyb = TypeBuilder::new(&*pool);
        let res = self.make_tuple_ex(vec, source, &mut tyb);

        let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces,
                                        &mut self.funcs, &mut self.context,
                                        self.string_map);
        tyb.finalise(data, &mut self.errors);
        res
    }
    
    
    pub fn make_tuple_ex(&mut self, vec: Vec<Type, &Arena>, source: SourceRange, tyb: &mut TypeBuilder) -> TypeId {
        if let Some(val) = self.tuple_map.get(&*vec) { return *val };
        let temp = ArenaPool::tls_get_temp();
        let name = {
            let mut str = sti::string::String::new_in(&*temp);
            str.push_char('(');
            for (i, t) in vec.iter().enumerate() {
                if i != 0 {
                    str.push(", ");
                }

                str.push(t.display(self.string_map, &self.types));
            }
            str.push_char(')');

            self.string_map.insert(str.as_str())
        };

        let v = Vec::from_in(&*temp, vec.iter().enumerate().map(|(i, x)| {
            let mut str = sti::string::String::new_in(&*temp);
            let _ = write!(str, "{}", i);
            let id = self.string_map.insert(&str);
            (id, *x)
        }));

        let tyid = self.types.pending(name);
        tyb.add_ty(tyid, name, source);
        tyb.set_struct_fields(tyid, &v, TypeStructStatus::Tuple);


        self.tuple_map.insert(vec.move_into(self.output).leak(), tyid);

        tyid
    }


    pub fn make_result(&mut self, v1: Type, v2: Type, tyb: &mut TypeBuilder) -> TypeId {
        if let Some(v) = self.results_map.get(&(v1, v2))
            { return *v; }

        let tyid = {
            let temp = ArenaPool::tls_get_temp();
            let name = {
                let mut str = sti::string::String::new_in(&*temp);
                str.push(v1.display(self.string_map, &self.types));
                str.push(" ~ ");
                str.push(v2.display(self.string_map, &self.types));

                self.string_map.insert(str.as_str())
            };

            let tyid = self.types.pending(name);
            tyb.add_ty(tyid, name, SourceRange::new(0, 0));
            tyb.set_enum_fields(tyid, 
                [
                (self.string_map.insert("ok"), Some(v1)),
                (self.string_map.insert("err"),Some(v2)),
                ].iter().copied(),
                TypeEnumStatus::Result,
            );

            tyid
        };

        self.results_map.insert((v1, v2), tyid);

        tyid
    }


    fn make_option(&mut self, ty: Type, tyb: &mut TypeBuilder) -> TypeId {
        if let Some(v) = self.options_map.get(&ty) { return *v; }

        let tyid = {
            let temp = ArenaPool::tls_get_temp();
            let name = {
                let mut str = sti::string::String::new_in(&*temp);
                str.push(ty.display(self.string_map, &self.types));
                str.push_char('?');

                self.string_map.insert(str.as_str())
            };

            let tyid = self.types.pending(name);
            tyb.add_ty(tyid, name, SourceRange::new(0, 0));
            tyb.set_enum_fields(tyid, 
                [
                (self.string_map.insert("some"), Some(ty)),
                (self.string_map.insert("none"), None),
                ].iter().copied(),
                TypeEnumStatus::Option,
            );


            tyid
        };

        self.options_map.insert(ty, tyid);
        tyid
    }


    fn make_ptr(&mut self, ty: Type, tyb: &mut TypeBuilder) -> Type {
        if let Some(ty) = self.rc_map.get(&ty) { return Type::Custom(*ty) }
        let temp = ArenaPool::tls_get_temp();
        let name = {
            let mut str = sti::string::String::new_in(&*temp);
            str.push_char('*');
            str.push(ty.display(self.string_map, &self.types));

            self.string_map.insert(str.as_str())
        };

        let tyid = {
            let tyid = self.types.pending(name);
            tyb.add_ty(tyid, name, SourceRange::new(0, 0));
            tyb.set_struct_fields(tyid, 
                &[
                    // Counter
                    (StringMap::COUNT, Type::I64),
                    // Data
                    (StringMap::VALUE, ty),
                ],
                TypeStructStatus::Ptr
            );

            tyid
        };

        self.rc_map.insert(ty, tyid);
        Type::Custom(tyid)
    }
     

    pub fn error(&mut self, err: Error) -> ErrorId {
        ErrorId::Sema(self.errors.push(err))
    }
}


impl<'me, 'out, 'str, 'ast> Analyzer<'me, 'out, 'str> {
    pub fn run(
        output: &'out Arena,
        string_map: &'me mut StringMap<'str>,
        nodes: &[Node<'ast>],
    ) -> Self {
        let mut context = Context::new();
        let mut slf = Self {
            scopes: ScopeMap::new(),
            namespaces: NamespaceMap::new(output),
            types: TypeMap::new(),
            funcs: FunctionMap::new(),
            module: Module::new(&mut context, "margarine"),
            context,
            errors: KVec::new(),
            output,
            string_map,
            options_map: HashMap::new(),
            results_map: HashMap::new(),
            tuple_map: HashMap::new(),
            rc_map: HashMap::new(),
            startup_functions: Vec::new(),
        };
        
        {
            let pool = ArenaPool::tls_get_temp();
            let mut type_builder = TypeBuilder::new(&pool);

            let rc = slf.make_ptr(Type::I32, &mut type_builder);

            {
                let id = slf.types.pending(StringMap::BOOL);
                assert_eq!(TypeId::BOOL, id);

                type_builder.add_ty(TypeId::BOOL, StringMap::BOOL, SourceRange::new(0, 0));
                type_builder.set_enum_fields(
                    TypeId::BOOL,
                    [(StringMap::FALSE, None), (StringMap::TRUE, None)].into_iter(),
                    TypeEnumStatus::User,
                );
            }

            {
                let id = slf.types.pending(StringMap::RANGE);
                assert_eq!(TypeId::RANGE, id);

                type_builder.add_ty(TypeId::RANGE, StringMap::RANGE, SourceRange::new(0, 0));
                type_builder.set_struct_fields(
                    TypeId::RANGE,
                    &[
                        (StringMap::LOW, Type::I64),
                        (StringMap::HIGH, Type::I64),
                    ],
                    TypeStructStatus::User
                );
            }


            {
                let id = slf.types.pending(StringMap::STR);
                assert_eq!(TypeId::STR, id);

                type_builder.add_ty(TypeId::STR, StringMap::STR, SourceRange::new(0, 0));
                type_builder.set_struct_fields(
                    TypeId::STR,
                    &[
                        (slf.string_map.insert("len"), Type::I64),
                        (slf.string_map.insert("ptr"), rc),
                    ],
                    TypeStructStatus::User
                );
            }

            let data = TypeBuilderData::new(&mut slf.types, &mut slf.namespaces,
                                            &mut slf.funcs, &mut slf.context,
                                            slf.string_map);

            type_builder.finalise(data, &mut slf.errors);
        }

        // prelude namespace
        let global_ns = {
            let mut ns = Namespace::new(slf.output, StringMap::INVALID_IDENT);

            ns.add_type(StringMap::RANGE, TypeId::RANGE);

            slf.namespaces.put(ns)
        };

        let void = slf.context.void();
        let mut builder = Function::new(&mut slf.context, slf.module, "init", void.ty(), &[]);
        let scope = Scope::new(ScopeKind::ImplicitNamespace(global_ns), PackedOption::NONE);
        let scope = slf.scopes.push(scope);

        builder.available_externally();
        let mut builder = Builder(builder);


        let anal = slf.block(&mut builder, StringMap::INVALID_IDENT, scope, nodes);
        slf.drop_value(anal.0.ty, &mut builder);


        for s in slf.startup_functions.iter() {
            let Some(f) = slf.funcs.get(*s)
            else { continue };
            assert!(f.args.is_empty(), "resources are not ready yet");

            builder.call(f.func, &[]);
        }

        let f = slf.funcs.pending();
        slf.funcs.put(f, Func::new(
                StringMap::INIT_FUNC,
                StringMap::INIT_FUNC,
                &[],
                Type::Unit,
                builder.build(),
                FunctionKind::UserDefined { inout: None }));

        slf
    }


    pub fn block(
        &mut self,
        builder: &mut Builder,
        path: StringIndex,
        scope: ScopeId,
        nodes: &[Node<'ast>],
    ) -> (AnalysisResult, ScopeId) {
        let old_scope = scope;
        let pool = ArenaPool::tls_get_rec();
        let mut ty_builder = TypeBuilder::new(&*pool); 
        let (mut scope, ns_id) = {
            let namespace = Namespace::new(self.output, path);
            let namespace = self.namespaces.put(namespace);

            self.collect_names(
                path,
                as_decl_iterator(nodes.iter().copied()),
                builder, &mut ty_builder, namespace
            );

            (Scope::new(ScopeKind::ImplicitNamespace(namespace), scope.some()), namespace)
        };

        self.collect_uses(as_decl_iterator(nodes.iter().copied()), builder, &mut scope, ns_id);

        let mut scope = self.scopes.push(scope);

        self.collect_impls(builder, &mut ty_builder, as_decl_iterator(nodes.iter().copied()), scope, ns_id);
        self.resolve_names(as_decl_iterator(nodes.iter().copied()), builder, &mut ty_builder, scope, ns_id);
        
        {
            let err_len = self.errors.len();

            let data = TypeBuilderData::new(
                &mut self.types, &mut self.namespaces,
                &mut self.funcs, &mut self.context,
                self.string_map);

            ty_builder.finalise(data, &mut self.errors);

            for i in err_len..self.errors.len() {
                builder.error(ErrorId::Sema(SemaError::new((err_len + i).try_into().unwrap()).unwrap()))
            }
        }
        
        self.resolve_functions(path, as_decl_iterator(nodes.iter().copied()), builder, scope, ns_id);

        let mut anal = AnalysisResult::new(Type::Unit, builder.unit(), true);
        for (id, n) in nodes.iter().enumerate() {
            anal = self.node(path, &mut scope, builder, n);

            if id + 1 != nodes.len() {
                self.acquire(anal.ty, builder);
                self.drop_value(anal.ty, builder);
            } 
        }

        if nodes.is_empty() { builder.unit(); }

        {
            let mut current = self.scopes.get(scope);
            loop {
                if let ScopeKind::Variable(v) = current.kind() {
                    builder.local_get(v.local_id);
                    self.drop_value(v.ty, builder);
                }

                let Some(parent) = current.parent().to_option()
                    else { break };

                if parent == old_scope { break }

                current = self.scopes.get(parent);
            }
        }

        (AnalysisResult::new(anal.ty, anal.value, anal.is_mut), scope)
    }
}


impl<'out> Analyzer<'_, 'out, '_> {
    pub fn collect_names<'a>(
        &mut self,
        path: StringIndex,
        decls: impl Iterator<Item=DeclarationNode<'a>>,
        
        builder: &mut Builder,
        type_builder: &mut TypeBuilder,
        namespace: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                | Declaration::Enum { name, header, .. }
                | Declaration::Struct { name, header, .. } => {
                    let Some(ns) = self.namespaces.get(namespace)
                    else { return };

                    if ns.get_type(name).is_some() {
                        type_builder.errored(name);
                        builder.error(self.error(Error::NameIsAlreadyDefined {  
                           source: header, name }));

                        continue
                    }

                    let path = self.concat_path(path, name);
                    let namespace = self.namespaces.get_mut(namespace).unwrap();
                    let ty = self.types.pending(path);
                    namespace.add_type(name, ty);
                    type_builder.add_ty(ty, name, header);
                },


                Declaration::Function { sig, is_in_impl, .. } => {
                    if is_in_impl.is_some() && sig.name == StringMap::ITER_NEXT_FUNC {
                        let validate_sig = || {
                            if sig.arguments.len() != 1 { return false }
                            let ty = is_in_impl.unwrap();
                            if sig.arguments[0].data_type().kind() != ty.kind() {
                                return false 
                            }

                            if !sig.arguments[0].is_inout() { return false }
                            if !matches!(sig.return_type.kind(), DataTypeKind::Option(_)) { return false }

                            true
                        };

                        if !validate_sig() {
                            builder.error(self.error(Error::IteratorFunctionInvalidSig(sig.source)));
                        }
                    }

                    let Some(namespace) = self.namespaces.get_mut(namespace)
                    else { return };

                    if let Some(f) = namespace.get_func(sig.name) {
                        self.funcs.error(f);
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: sig.source, name: sig.name }));

                        continue
                    }

                    namespace.add_func(sig.name, self.funcs.pending())
                },


                Declaration::Impl { .. } => (),

                Declaration::Using { .. } => (),

                Declaration::Module { name, body } => {
                    let Some(root) = self.namespaces.get_mut(namespace)
                    else { return };

                    if let Some(ns) = root.get_mod(name) {
                        self.namespaces.error(ns);
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: decl.range(), name }));

                        continue
                    }

                    let path = self.concat_path(path, name);
                    let ns = Namespace::new(self.output, path);
                    let ns = self.namespaces.put(ns);

                    self.collect_names(path, as_decl_iterator(body.iter().copied()),
                                            builder, type_builder, ns);

                    let root = self.namespaces.get_mut(namespace).unwrap();
                    root.add_mod(name, ns);
                },

                Declaration::Extern { functions, .. } => {
                    for f in functions.iter() {
                        let Some(namespace) = self.namespaces.get_mut(namespace)
                        else { return };
                        if let Some(id) = namespace.get_func(f.name()) {
                            self.funcs.error(id);
                            builder.error(self.error(Error::NameIsAlreadyDefined { 
                               source: f.range(), name: f.name() }));

                            continue
                        }

                        namespace.add_func(f.name(), self.funcs.pending())
                    }

                },

            }
        }
    }


    pub fn collect_impls<'a>(
        &mut self,
        builder: &mut Builder,
        type_builder: &mut TypeBuilder,
        decls: impl Iterator<Item=DeclarationNode<'a>>,
        
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Impl { data_type, body } => {
                    let ty = match self.convert_ty_ex(type_builder, scope, data_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            continue;
                        },
                    };
                    
                    let ns_id = self.namespaces.get_type(ty, &self.types);

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns_id), scope.some());
                    let scope = self.scopes.push(scope);

                    let path = ty.path(&self.types);
                    self.collect_names(path, as_decl_iterator(body.iter().copied()), builder, type_builder, ns_id);
                    self.collect_impls(builder, type_builder, as_decl_iterator(body.iter().copied()), scope, ns_id);
                }


                Declaration::Module { name, body } => {
                    let Some(ns_id) = self.namespaces.get(ns_id)
                    else { continue };

                    let ns = ns_id.get_mod(name).unwrap();

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);

                    self.collect_impls(builder, type_builder, as_decl_iterator(body.iter().copied()), scope, ns);
                },

                _ => continue
            }
        }

    }


    pub fn collect_uses<'a>(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'a>>,

        builder: &mut Builder,
        scope: &mut Scope,
        _ns_id: NamespaceId,
    ) {
        let initial_scope = *scope;
        for decl in decls {
            match decl.kind() {
                Declaration::Module { body, .. } =>
                    self.collect_uses(as_decl_iterator(body.iter().copied()), builder, scope, _ns_id),
                _ => ()
            };

            let Declaration::Using { item } = decl.kind()
            else { continue };

            if let Err(e) = self.use_item(item, initial_scope, scope) {
                builder.error(e);
            }
        }
    }


    fn use_item(
        &mut self,
        use_item: UseItem,
        scope: Scope,
        modify: &mut Scope,
    ) -> Result<(), ErrorId> {
        match use_item.kind() {
            UseItemKind::List { list } => {
                let Some(ns) = scope.get_ns(use_item.name(), &self.scopes, &mut self.namespaces, &self.types)
                else {
                    return Err(self.error(Error::NamespaceNotFound {
                        source: use_item.range(), namespace: use_item.name() }));
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), None.into());
                for l in list {
                    self.use_item(*l, scope, modify)?;
                }
            },

            UseItemKind::BringName => {
                if let Some(ty) = scope.get_type(use_item.name(), &self.scopes, &self.namespaces) {
                    *modify = Scope::new(ScopeKind::ImportType((use_item.name(), ty)), self.scopes.push(*modify).some());
                    return Ok(())
                }

                if let Some(f) = scope.get_func(use_item.name(), &self.scopes, &self.namespaces) {
                    *modify = Scope::new(ScopeKind::ImportFunction((use_item.name(), f)), self.scopes.push(*modify).some());
                    return Ok(())
                }

                if let Some(ns) = scope.get_mod(use_item.name(), &self.scopes, &mut self.namespaces) {
                    *modify = Scope::new(ScopeKind::ExplicitNamespace(ExplicitNamespace { name: use_item.name(), namespace: ns }), self.scopes.push(*modify).some());
                    return Ok(())
                }

                return Err(self.error(Error::NamespaceNotFound {
                    source: use_item.range(), namespace: use_item.name() }));
            },

            UseItemKind::All => {
                if let Some(ns) = scope.get_ns(use_item.name(), &self.scopes, &mut self.namespaces, &self.types) {
                    *modify = Scope::new(ScopeKind::ImplicitNamespace(ns), self.scopes.push(*modify).some());
                    return Ok(())
                }

                return Err(self.error(Error::NamespaceNotFound {
                    source: use_item.range(), namespace: use_item.name() }));

            },
        };

        Ok(())
    }




    pub fn resolve_names<'builder, 'a: 'builder>(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'a>>,

        builder: &mut Builder,
        type_builder: &mut TypeBuilder<'builder>,
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Struct { name, fields,  .. } => {
                    let Some(ns) = self.namespaces.get(ns_id)
                    else { return };
                    let ty = ns.get_type(name).unwrap();

                    let fields = fields.iter()
                        .filter_map(|(name, ty, _)| {
                            let ty = self.convert_ty_ex(type_builder, scope, *ty);
                            match ty {
                                Ok(v) => return Some((*name, v)),
                                Err(e) => self.error(e),
                            };

                            None
                        }
                    );

                    let pool = ArenaPool::tls_get_temp();
                    let fields = Vec::from_in(&*pool, fields);

                    type_builder.set_struct_fields(ty, &fields, TypeStructStatus::User);
                },


                Declaration::Enum { name, mappings, .. } => {
                    let Some(ns) = self.namespaces.get(ns_id)
                    else { return };
                    let ty = ns.get_type(name).unwrap();

                    let mappings = mappings.iter()
                        .filter_map(|mapping| {
                            let ty = match mapping.is_implicit_unit() {
                                true => None,
                                false => {
                                    let ty = mapping.data_type();

                                    let ty = self.convert_ty(scope, *ty);
                                    let ty = match ty {
                                        Ok(v) => v,
                                        Err(e) => {
                                            self.error(e);
                                            return None;
                                        },
                                    };

                                    Some(ty)
                                }
                            };

                            Some((mapping.name(), ty))
                        });

                    type_builder.set_enum_fields(ty, mappings, TypeEnumStatus::User)
                },


                Declaration::Impl { data_type, body } => {
                    let ns = {
                        let Ok(ty) = self.convert_ty_ex(type_builder, scope, data_type)
                        else { continue };

                        self.namespaces.get_type(ty, &self.types)
                    };

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);
                    
                    self.resolve_names(as_decl_iterator(body.iter().copied()), builder, type_builder, scope, ns);
                },

                Declaration::Module { name, body } => {
                    let Some(ns) = self.namespaces.get(ns_id)
                    else { continue };
                    let ns = ns.get_mod(name).unwrap();

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);

                    self.resolve_names(as_decl_iterator(body.iter().copied()), builder, type_builder, scope, ns)
                },

                Declaration::Using { .. } => (),

                _ => continue,
           }
        }
    }


    pub fn resolve_functions<'a>(
        &mut self,
        path: StringIndex,
        decls: impl Iterator<Item=DeclarationNode<'a>>,

        builder: &mut Builder,
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Function { sig, .. } => {
                    let args = {
                        let mut args = Vec::with_cap_in(self.output, sig.arguments.len());

                        for arg in sig.arguments.iter() {
                            let ty = self.convert_ty(scope, arg.data_type());
                            let ty = match ty {
                                Ok(v) => v,
                                Err(e) => {
                                    builder.error(self.error(e));
                                    Type::Error
                                },
                            };

                            args.push((arg.name(), arg.is_inout(), ty));
                        }

                        args
                    };

                    let ret = match self.convert_ty(scope, sig.return_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            Type::Error
                        },
                    };

                    let Some(ns) = self.namespaces.get_mut(ns_id)
                    else { return };
                    let func_id = ns.get_func(sig.name).unwrap();
                    let func = {
                        let inout_ty_id = if args.iter().any(|a| a.1) {
                            let temp = ArenaPool::tls_get_temp();
                            let tuple = Vec::from_in(&*temp,
                                                       args.iter().map(|a| a.2));
                            Some(self.make_tuple(tuple, decl.range()))
                        } else { None };

                        FunctionKind::UserDefined {
                            inout: inout_ty_id,
                        }
                    };

                    let path = self.concat_path(path, sig.name);
                    let func = Func::new(sig.name, path, args.leak(), ret, self.module_builder.function_id(), func);
                    self.funcs.put(func_id, func);
                },


                Declaration::Extern { file, functions } => {
                    for f in functions.iter() {
                        let args = {
                            let mut args = Vec::with_cap_in(self.output, f.args().len());

                            for arg in f.args().iter() {
                                let ty = self.convert_ty(scope, arg.data_type());
                                let ty = match ty {
                                    Ok(v) => v,
                                    Err(e) => {
                                        builder.error(self.error(e));
                                        Type::Error
                                    },
                                };

                                args.push((arg.name(), arg.is_inout(), ty));
                            }

                            args
                        };

                        let ret = match self.convert_ty(scope, f.return_type()) {
                            Ok(v) => v,
                            Err(e) => {
                                builder.error(self.error(e));
                                Type::Error
                            },
                        };

                        let ty = {
                            let pool = ArenaPool::tls_get_temp();
                            let mut vec = Vec::with_cap_in(&*pool, args.len() + 1);
                            vec.extend(args.iter().map(|x| x.2));
                            vec.push(ret);
                            self.make_tuple(vec, decl.range())
                        };

                        let wfid = self.module_builder.extern_func(file, f.path());
                        
                        let Some(ns) = self.namespaces.get_mut(ns_id)
                        else { return };
                        let func_id = ns.get_func(f.name()).unwrap();
                        let func = FunctionKind::Extern { ty };

                        let path = self.concat_path(path, f.name());
                        let func = Func::new(f.name(), path, args.leak(), ret, wfid, func);
                        self.funcs.put(func_id, func);
                    }
                },


                Declaration::Impl { data_type, body } => {
                    let Ok(ty) = self.convert_ty(scope, data_type)
                    else { continue };
                    
                    let ns_id = self.namespaces.get_type(ty, &self.types);

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns_id), scope.some());
                    let scope = self.scopes.push(scope);

                    let path = ty.path(&self.types);
                    self.resolve_functions(path, as_decl_iterator(body.iter().copied()), builder, scope, ns_id);
                }


                Declaration::Module { name, body } => {
                    let Some(ns_id) = self.namespaces.get_mut(ns_id)
                    else { return };
                    let ns_id = ns_id.get_mod(name).unwrap();

                    let Some(ns) = self.namespaces.get(ns_id)
                    else { continue };

                    let path = ns.path();
                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns_id), scope.some());
                    let scope = self.scopes.push(scope);

                    self.resolve_functions(path, as_decl_iterator(body.iter().copied()), builder, scope, ns_id)
                }

                _ => continue,
            }
        }

    }


    fn node(
        &mut self,
        path: StringIndex,
        scope: &mut ScopeId,
        builder: &mut Builder,

        node: &Node,
    ) -> AnalysisResult {
        match node {
            Node::Declaration(decl) => {
                self.decl(*decl, *scope, builder);
                AnalysisResult::new(Type::Unit, builder.unit(), true)
            },

            Node::Statement(stmt) => {
                if self.stmt(*stmt, scope, builder, path).is_err() {
                    return AnalysisResult::error(builder)
                }
                AnalysisResult::new(Type::Unit, builder.unit(), true)

            },

            Node::Expression(expr) => self.expr(*expr, *scope, builder, path),

            Node::Error(err) => {
                builder.error(err.id());
                AnalysisResult::error(builder)
            },

            Node::Attribute(attr) => {
                match self.string_map.get(attr.attr().name()) {
                    "startup" => {
                        let check = || {
                            let Node::Declaration(decl) = attr.node()
                            else { return Err(()) };

                            if matches!(decl.kind(), Declaration::Function {
                                sig: FunctionSignature { is_system: true, .. }, .. }) {

                                Ok(decl)
                            } else { Err(()) }
                        };

                        let decl = match check() {
                            Ok(val) => val,
                            Err(_) => {
                                builder.error(self.error(Error::InvalidValueForAttr {
                                    attr: (attr.range(), attr.attr().name()),
                                    value: node.range(),
                                    expected: "a system function",
                                }));
                                return AnalysisResult::error(builder);
                            }
                        };

                        self.decl(decl, *scope, builder);

                        let Declaration::Function { sig, .. } = decl.kind()
                        else { unreachable!() };

                        let func = self.scopes.get(*scope).get_func(sig.name, &self.scopes, &self.namespaces).unwrap();

                        self.startup_functions.push(func);

                        AnalysisResult::new(Type::Unit, builder.unit(), true)
                    }
                    _ => {
                        builder.error(self.error(Error::UnknownAttr(attr.range(), attr.attr().name())));
                        AnalysisResult::error(builder)
                    }
                }
            },
        }
    }


    fn decl(
        &mut self,
        decl: DeclarationNode,
        scope: ScopeId,
        builder: &mut Builder,
    ) {
        match decl.kind() {
            Declaration::Struct { .. } => (),
            Declaration::Enum { .. } => (),


            Declaration::Function { sig, body, .. } => {
                let func = self.scopes.get(scope).get_func(sig.name, &self.scopes, &self.namespaces).unwrap();
                let Some(func) = self.funcs.get(func)
                else { return };

                match func.kind {
                    FunctionKind::UserDefined { inout } => {
                        let mut wasm = WasmFunctionBuilder::new(self.output, func.func);
                        wasm.export(func.path);

                        let ret = func.ret.to_wasm_ty(&self.types);
                        wasm.return_value(ret);

                        let scope = Scope::new(
                            ScopeKind::FunctionDefinition(
                                FunctionDefinitionScope::new(func.ret, sig.return_type.range())
                            ),
                            scope.some()
                        );

                        let mut scope = self.scopes.push(scope);

                        let fields = inout.map(|inout| self.types.get(inout).kind()).map(|x| {
                            let TypeKind::Struct(val) = x else { unreachable!() };
                            val
                        });

                        let mut counter = 0;
                        let mut string = String::new_in(self.output);
                        let pool = ArenaPool::tls_get_temp();
                        let mut ids = Vec::new_in(&*pool);
                        for a in func.args.iter() {
                            let wasm_ty = a.2.to_wasm_ty(&self.types);

                            let local_id = wasm.param(wasm_ty);
                            if a.1 {
                                ids.push(local_id);
                            }

                            let t = Scope::new(
                                ScopeKind::Variable(VariableScope::new(a.0, a.1, a.2, local_id)),
                                scope.some()
                            );

                            scope = self.scopes.push(t);
                        }

                        let inout_param = inout.map(|inout| {
                            let ty = self.types.get(inout);
                            wasm.param(WasmType::Ptr { size: ty.size() })
                        });

                        for i in ids.iter() {
                            wasm.write_to(&mut string, |wasm| {
                                let f = fields.unwrap().fields[counter];
                                let inout = inout_param.unwrap();
                                wasm.local_get(*i);

                                wasm.local_get(inout);
                                wasm.u32_const(f.1.try_into().unwrap());
                                wasm.i32_add();

                                wasm.write(f.0.ty.to_wasm_ty(&self.types));

                                counter += 1;
                            });
                        }

                        let wasm_ret = func.ret.to_wasm_ty(&self.types);
                        if wasm_ret.stack_size() != 0 {
                            wasm.param(wasm_ret);
                        }

                        wasm.set_finaliser(string.clone_in(self.module_builder.arena));
                        wasm.export(func.path);

                        let func_ret = func.ret;

                        let (anal, _) = self.block(&mut wasm, func.path, scope, &body);
                        if anal.ty.eq_lit(Type::Error) {
                            wasm.pop();
                            wasm.default(ret);
                        }

                        if !anal.ty.eq_sem(func_ret) {
                            wasm.pop();
                            wasm.default(ret);

                            wasm.error(self.error(Error::FunctionBodyAndReturnMismatch {
                                header: sig.source, item: body.last().map(|x| x.range()).unwrap_or(body.range()),
                                return_type: func_ret, body_type: anal.ty }));
                        }

                        self.module_builder.register(wasm);
                    },


                    FunctionKind::Extern { .. } => unreachable!(),
                };
            },


            Declaration::Impl { data_type, body } => {
                let Ok(ty) = self.convert_ty(scope, data_type)
                else { return };

                let ns = self.namespaces.get_type(ty, &self.types);
                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let mut scope = self.scopes.push(scope);

                let path = ty.path(&self.types);
                for decl in body.iter() {
                    self.node(path, &mut scope, builder, decl);
                }
            },

            Declaration::Using { .. } => (),
            Declaration::Module { name, body } => {
                let ns = self.scopes.get(scope);
                let ns = ns.get_mod(name, &self.scopes, &self.namespaces).unwrap();

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let mut scope = self.scopes.push(scope);

                let Some(ns) = self.namespaces.get(ns)
                else { return };
                let path = ns.path();
                for decl in body.iter() {
                    self.node(path, &mut scope, builder, decl);
                }
            },

            Declaration::Extern { .. } => (),
        }
    }


    fn stmt(
        &mut self,
        stmt: StatementNode,

        scope: &mut ScopeId,
        builder: &mut Builder,
        path: StringIndex,
    ) -> Result<(), ()> {
        let source = stmt.range();
        match stmt.kind() {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let mut func = || -> Result<(), ()> {
                    let rhs_anal = self.expr(rhs, *scope, builder, path);
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(());
                    }

                    if let Some(hint) = hint {
                        let hint = match self.convert_ty(*scope, hint) {
                            Ok(v) => v,
                            Err(e) => {
                                builder.error(self.error(e));
                                return Err(());
                            }
                        };

                        if !hint.eq_sem(rhs_anal.ty) {
                            builder.error(self.error(Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source }));
                            return Err(())
                        }
                    }

                    let local = builder.local(rhs_anal.ty.to_wasm_ty(&self.types));
                    self.acquire(rhs_anal.ty, builder);
                    builder.local_set(local, rhs_anal.value);

                    let variable_scope = VariableScope::new(name, is_mut, rhs_anal.ty, local);
                    *scope = self.scopes.push(
                        Scope::new(ScopeKind::Variable(variable_scope), scope.some()));

                    Ok(())
                };

                if func().is_err() {
                    let dummy = VariableScope::new(name, is_mut, Type::Error, builder.ctx().zst().ty());
                    *scope = self.scopes.push(Scope::new(ScopeKind::Variable(dummy), scope.some()));
                    return Err(());
                }
                
            },


            Statement::VariableTuple { names, hint, rhs } => {
                let mut func = || -> Result<(), ()> {
                    let rhs_anal = self.expr(rhs, *scope, builder, path);
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(());
                    }

                    if let Some(hint) = hint {
                        let hint = match self.convert_ty(*scope, hint) {
                            Ok(v) => v,
                            Err(e) => {
                                builder.error(self.error(e));
                                return Err(());
                            }
                        };

                        if !hint.eq_sem(rhs_anal.ty) {
                            builder.error(self.error(Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source }));
                            return Err(())
                        }
                    }

                    let rty = match rhs_anal.ty {
                        Type::Custom(v) => self.types.get(v),
                        _ => {
                            builder.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                            return Err(());
                        }
                    };

                    let TypeKind::Struct(sym) = rty.kind()
                    else {
                        builder.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                        return Err(());
                    };

                    if sym.status != TypeStructStatus::Tuple {
                        builder.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                        return Err(());
                    }

                    let ptr = builder.local(rhs_anal.ty.to_wasm_ty(&self.types));
                    builder.local_set(ptr);

                    for (binding, sym) in names.iter().zip(sym.fields.iter()) {
                        builder.local_get(ptr);
                        builder.u32_const(sym.1.try_into().unwrap());
                        builder.i32_add();


                        let sym_ty = sym.0.ty.to_wasm_ty(&self.types);
                        builder.read(sym_ty);

                        let local = builder.local(sym_ty);
                        builder.local_set(local);

                        let variable_scope = VariableScope::new(binding.0, binding.1, sym.0.ty, local);
                        *scope = self.scopes.push(
                            Scope::new(ScopeKind::Variable(variable_scope), scope.some()));
                    }

                    Ok(())
                };

                if func().is_err() {
                    for binding in names.iter() {
                        let dummy = VariableScope::new(binding.0, binding.1, Type::Error, builder.local(WasmType::I64));
                        *scope = self.scopes.push(Scope::new(ScopeKind::Variable(dummy), scope.some()));
                    }
                    return Err(());
                }

            },


            Statement::UpdateValue { lhs, rhs } => {
                let rhs_anal = self.expr(rhs, *scope, builder, path);
                if let Err(e) = self.assign(builder, *scope, lhs, rhs_anal.ty, 0) {
                    builder.error(self.error(e));
                    return Err(());
                }
            },


            Statement::ForLoop { binding, expr, body } => {
                let expr_anal = self.expr(expr.1, *scope, builder, path);

                if !expr_anal.is_mut && expr.0 {
                    builder.pop();
                    builder.error(self.error(Error::InOutValueIsntMut(expr.1.range())));
                    return Err(());
                }

                if binding.0 && !expr.0 {
                    builder.pop();
                    builder.error(self.error(Error::InOutBindingWithoutInOutValue {
                        value_range: expr.1.range() }));
                    return Err(());
                }

                if !binding.0 && expr.0 {
                    builder.pop();
                    builder.error(self.error(Error::InOutValueWithoutInOutBinding { value_range: expr.1.range() }));
                    return Err(());
                }

                let func = self.namespaces.get_type(expr_anal.ty, &self.types);
                let Some(ns) = self.namespaces.get(func)
                else { return Err(()) };
                if ns.get_func(StringMap::ITER_NEXT_FUNC).is_none() {
                    builder.pop();
                    builder.error(self.error(Error::ValueIsntAnIterator {
                        ty: expr_anal.ty, range: expr.1.range() }));
                    return Err(());
                };

                let local = builder.local(expr_anal.ty.to_wasm_ty(&self.types));
                builder.local_set(local);

                let vscope = VariableScope::new(
                    StringMap::INVALID_IDENT,
                    true,
                    expr_anal.ty,
                    local);

                let scope = Scope::new(ScopeKind::Variable(vscope), scope.some());
                let scope = self.scopes.push(scope);

                let get_iterator = ExpressionNode::new(
                        Expression::Identifier(StringMap::INVALID_IDENT),
                        source);

                let var_decl_func_call_args = [(get_iterator, true)];
                let loop_var_decl = Node::Statement(StatementNode::new(Statement::Variable {
                                name: StringMap::INVALID_IDENT, hint: None, is_mut: false,
                                rhs: ExpressionNode::new(Expression::CallFunction {
                                    name: StringMap::ITER_NEXT_FUNC, is_accessor: true,
                                    args: &var_decl_func_call_args }, source) },
                                source));

                let match_id = ExpressionNode::new(
                    Expression::Identifier(StringMap::INVALID_IDENT), source);

                let match_arms = [
                    MatchMapping::new(
                        StringMap::SOME, binding.1, source,
                        source,
                        ExpressionNode::new(Expression::Block { block: body }, source),
                        false),

                    MatchMapping::new(
                        StringMap::NONE, StringMap::INVALID_IDENT, source,
                        source,
                        ExpressionNode::new(Expression::Break, source),
                        false),
                ];

                let body_match = Node::Expression(ExpressionNode::new(Expression::Match { 
                    value: &match_id,
                    taken_as_inout: false,
                    mappings: &match_arms
                }, source));

                let mut loop_body = [
                    loop_var_decl,
                    body_match,
                ];

                let loop_node = ExpressionNode::new(
                    Expression::Loop { body: Block::new(
                    &mut loop_body,
                    source) },
                    source
                );

                let mut block_body = [
                    Node::Expression(loop_node)
                ];

                let tree = ExpressionNode::new(
                    Expression::Block { block: Block::new(
                            &mut block_body,
                            source,
                    )},
                    source,
                );

                self.expr(tree, scope, builder, path);
                builder.pop();
            },
        };

        Ok(())
   }


    fn expr(
        &mut self,
        expr: ExpressionNode,

        scope: ScopeId,
        wasm: &mut Builder,
        path: StringIndex,
    ) -> AnalysisResult {
        let source = expr.range();
        match expr.kind() {
            Expression::Unit => {
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },

            Expression::Literal(l) => {
                match l {
                    lexer::Literal::Integer(i) => {
                        wasm.i64_const(i);
                        AnalysisResult::new(Type::I64, true)
                    },


                    lexer::Literal::Float(f) => {
                        wasm.f64_const(f.inner());
                        AnalysisResult::new(Type::F64, true)
                    },


                    lexer::Literal::String(v) => {
                        let str = self.string_map.get(v);
                        let string_ptr = self.module_builder.add_string(str);
                        
                        let ty = self.types.get(TypeId::STR);
                        let TypeKind::Struct(strct) = ty.kind()
                        else { unreachable!() };

                        let alloc = wasm.alloc_stack(ty.size());

                        // len
                        {

                            let ptr = alloc.add(strct.fields[0].1);
                            wasm.i64_const(str.len() as i64);
                            wasm.sptr_const(ptr);
                            wasm.i64_write();
                        }

                        // ptr
                        {
                            let ptr = alloc.add(strct.fields[1].1);
                            wasm.string_const(string_ptr);
                            wasm.i32_as_i64();

                            wasm.sptr_const(ptr);

                            wasm.i64_write();
                        }

                        wasm.sptr_const(alloc);
                        
                        AnalysisResult::new(Type::STR, true)
                    },

                    lexer::Literal::Bool(v) => {
                        let ty = Type::BOOL;
                        let name = if v { StringMap::TRUE } else { StringMap::FALSE };

                        let func = self.namespaces.get_type(ty, &self.types);
                        let Some(ns) = self.namespaces.get(func)
                        else { return AnalysisResult::error() };

                        let func = ns.get_func(name).unwrap();
                        
                        self.call_func(func, &[], false, source, scope, wasm).unwrap();
                        AnalysisResult::new(Type::BOOL, true)
                    },
                }
            },


            Expression::Identifier(ident) => {
                let Some(variable) = self.scopes.get(scope).get_var(ident, &self.scopes)
                else {
                    wasm.error(self.error(Error::VariableNotFound { name: ident, source }));
                    return AnalysisResult::error()
                };

                wasm.local_get(variable.local_id);
                AnalysisResult::new(variable.ty, variable.is_mutable)
            },


            Expression::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.expr(*lhs, scope, wasm, path);
                let rhs_anal = self.expr(*rhs, scope, wasm, path);

                let is_valid = match operator {
                    _ if !lhs_anal.ty.eq_sem(rhs_anal.ty) => false,
                    _ if lhs_anal.ty.eq_lit(Type::Error)  
                         || rhs_anal.ty.eq_lit(Type::Error) => { 
                        wasm.pop();
                        wasm.pop();
                        wasm.unit();
                        return AnalysisResult::error();
                    },

                    _ if lhs_anal.ty.eq_lit(Type::Never) => false,
                    _ if lhs_anal.ty.eq_lit(Type::Error) => false,

                    v if v == BinaryOperator::Rem => lhs_anal.ty.is_integer(),
                    v if v.is_arith() => lhs_anal.ty.is_number(),
                    v if v.is_bw() => lhs_anal.ty.is_number(),
                    v if v.is_ocomp() => lhs_anal.ty.is_number(),
                    v if v.is_ecomp() => lhs_anal.ty.is_number(),

                    _ => false,
                };

                if !is_valid {
                    wasm.pop();
                    wasm.pop();
                    wasm.unit();
                    wasm.error(self.error(Error::InvalidBinaryOp {
                        lhs: lhs_anal.ty, rhs: rhs_anal.ty,
                        operator, source,
                    }));
                    return AnalysisResult::error();
                }

                macro_rules! wfunc {
                    ($n: ident, $ty: expr) => {
                        {
                            wasm.$n();
                            $ty
                        }
                    };
                }

                let ty = match (operator, lhs_anal.ty) {
                    (BinaryOperator::Add, Type::I8 ) => wfunc!( i8_add, Type::I8 ),
                    (BinaryOperator::Add, Type::U8 ) => wfunc!( u8_add, Type::U8 ),
                    (BinaryOperator::Add, Type::I16) => wfunc!(i16_add, Type::I16),
                    (BinaryOperator::Add, Type::U16) => wfunc!(u16_add, Type::U16),
                    (BinaryOperator::Add, Type::I32) => wfunc!(i32_add, Type::I32),
                    (BinaryOperator::Add, Type::U32) => wfunc!(i32_add, Type::U32),
                    (BinaryOperator::Add, Type::I64) => wfunc!(i64_add, Type::I64),
                    (BinaryOperator::Add, Type::U64) => wfunc!(i64_add, Type::U64),
                    (BinaryOperator::Add, Type::F32) => wfunc!(f32_add, Type::F32),
                    (BinaryOperator::Add, Type::F64) => wfunc!(f64_add, Type::F64),


                    (BinaryOperator::Sub, Type::I8 ) => wfunc!( i8_sub, Type::I8 ),
                    (BinaryOperator::Sub, Type::U8 ) => wfunc!( u8_sub, Type::U8 ),
                    (BinaryOperator::Sub, Type::I16) => wfunc!(i16_sub, Type::I16),
                    (BinaryOperator::Sub, Type::U16) => wfunc!(u16_sub, Type::U16),
                    (BinaryOperator::Sub, Type::I32) => wfunc!(i32_sub, Type::I32),
                    (BinaryOperator::Sub, Type::U32) => wfunc!(i32_sub, Type::U32),
                    (BinaryOperator::Sub, Type::I64) => wfunc!(i64_sub, Type::I64),
                    (BinaryOperator::Sub, Type::U64) => wfunc!(i64_sub, Type::U64),
                    (BinaryOperator::Sub, Type::F32) => wfunc!(f32_sub, Type::F32),
                    (BinaryOperator::Sub, Type::F64) => wfunc!(f64_sub, Type::F64),

                    (BinaryOperator::Mul, Type::I8 ) => wfunc!( i8_mul, Type::I8 ),
                    (BinaryOperator::Mul, Type::U8 ) => wfunc!( u8_mul, Type::U8 ),
                    (BinaryOperator::Mul, Type::I16) => wfunc!(i16_mul, Type::I16),
                    (BinaryOperator::Mul, Type::U16) => wfunc!(u16_mul, Type::U16),
                    (BinaryOperator::Mul, Type::I32) => wfunc!(i32_mul, Type::I32),
                    (BinaryOperator::Mul, Type::U32) => wfunc!(i32_mul, Type::U32),
                    (BinaryOperator::Mul, Type::I64) => wfunc!(i64_mul, Type::I64),
                    (BinaryOperator::Mul, Type::U64) => wfunc!(i64_mul, Type::U64),
                    (BinaryOperator::Mul, Type::F32) => wfunc!(f32_mul, Type::F32),
                    (BinaryOperator::Mul, Type::F64) => wfunc!(f64_mul, Type::F64),


                    (BinaryOperator::Div, Type::I8 ) => wfunc!( i8_div, Type::I8 ),
                    (BinaryOperator::Div, Type::U8 ) => wfunc!( u8_div, Type::U8 ),
                    (BinaryOperator::Div, Type::I16) => wfunc!(i16_div, Type::I16),
                    (BinaryOperator::Div, Type::U16) => wfunc!(u16_div, Type::U16),
                    (BinaryOperator::Div, Type::I32) => wfunc!(i32_div, Type::I32),
                    (BinaryOperator::Div, Type::U32) => wfunc!(u32_div, Type::U32),
                    (BinaryOperator::Div, Type::I64) => wfunc!(i64_div, Type::I64),
                    (BinaryOperator::Div, Type::U64) => wfunc!(u64_div, Type::U64),
                    (BinaryOperator::Div, Type::F32) => wfunc!(f32_div, Type::F32),
                    (BinaryOperator::Div, Type::F64) => wfunc!(f64_div, Type::F64),


                    (BinaryOperator::Rem, Type::I8 ) => wfunc!( i8_rem, Type::I8 ),
                    (BinaryOperator::Rem, Type::U8 ) => wfunc!( u8_rem, Type::U8 ),
                    (BinaryOperator::Rem, Type::I16) => wfunc!(i16_rem, Type::I16),
                    (BinaryOperator::Rem, Type::U16) => wfunc!(u16_rem, Type::U16),
                    (BinaryOperator::Rem, Type::I32) => wfunc!(i32_rem, Type::I32),
                    (BinaryOperator::Rem, Type::U32) => wfunc!(i32_rem, Type::U32),
                    (BinaryOperator::Rem, Type::I64) => wfunc!(i64_rem, Type::I64),
                    (BinaryOperator::Rem, Type::U64) => wfunc!(i64_rem, Type::U64),


                    (BinaryOperator::BitshiftLeft, Type::I8 ) => wfunc!( i8_bw_left_shift, Type::I8 ),
                    (BinaryOperator::BitshiftLeft, Type::U8 ) => wfunc!( u8_bw_left_shift, Type::U8 ),
                    (BinaryOperator::BitshiftLeft, Type::U16) => wfunc!(u16_bw_left_shift, Type::U16),
                    (BinaryOperator::BitshiftLeft, Type::I16) => wfunc!(i16_bw_left_shift, Type::I16),
                    (BinaryOperator::BitshiftLeft, Type::I32) => wfunc!(i32_bw_left_shift, Type::I32),
                    (BinaryOperator::BitshiftLeft, Type::U32) => wfunc!(i32_bw_left_shift, Type::U32),
                    (BinaryOperator::BitshiftLeft, Type::I64) => wfunc!(i64_bw_left_shift, Type::I64),
                    (BinaryOperator::BitshiftLeft, Type::U64) => wfunc!(i64_bw_left_shift, Type::U64),


                    (BinaryOperator::BitshiftRight, Type::I8 ) => wfunc!( i8_bw_right_shift, Type::I8 ),
                    (BinaryOperator::BitshiftRight, Type::U8 ) => wfunc!( u8_bw_right_shift, Type::U8 ),
                    (BinaryOperator::BitshiftRight, Type::U16) => wfunc!(u16_bw_right_shift, Type::U16),
                    (BinaryOperator::BitshiftRight, Type::I16) => wfunc!(i16_bw_right_shift, Type::I16),
                    (BinaryOperator::BitshiftRight, Type::I32) => wfunc!(i32_bw_right_shift, Type::I32),
                    (BinaryOperator::BitshiftRight, Type::U32) => wfunc!(i32_bw_right_shift, Type::U32),
                    (BinaryOperator::BitshiftRight, Type::I64) => wfunc!(i64_bw_right_shift, Type::I64),
                    (BinaryOperator::BitshiftRight, Type::U64) => wfunc!(i64_bw_right_shift, Type::U64),


                    (BinaryOperator::BitwiseAnd, Type::I8 ) => wfunc!( i8_bw_and, Type::I8 ),
                    (BinaryOperator::BitwiseAnd, Type::U8 ) => wfunc!( u8_bw_and, Type::U8 ),
                    (BinaryOperator::BitwiseAnd, Type::I16) => wfunc!(i16_bw_and, Type::I16),
                    (BinaryOperator::BitwiseAnd, Type::U16) => wfunc!(u16_bw_and, Type::U16),
                    (BinaryOperator::BitwiseAnd, Type::I32) => wfunc!(i32_bw_and, Type::I32),
                    (BinaryOperator::BitwiseAnd, Type::U32) => wfunc!(i32_bw_and, Type::U32),
                    (BinaryOperator::BitwiseAnd, Type::I64) => wfunc!(i64_bw_and, Type::I64),
                    (BinaryOperator::BitwiseAnd, Type::U64) => wfunc!(i64_bw_and, Type::U64),


                    (BinaryOperator::BitwiseOr, Type::I8 ) => wfunc!( i8_bw_or, Type::I8 ),
                    (BinaryOperator::BitwiseOr, Type::U8 ) => wfunc!( u8_bw_or, Type::U8 ),
                    (BinaryOperator::BitwiseOr, Type::I16) => wfunc!(i16_bw_or, Type::I16),
                    (BinaryOperator::BitwiseOr, Type::U16) => wfunc!(u16_bw_or, Type::U16),
                    (BinaryOperator::BitwiseOr, Type::I32) => wfunc!(i32_bw_or, Type::I32),
                    (BinaryOperator::BitwiseOr, Type::U32) => wfunc!(i32_bw_or, Type::U32),
                    (BinaryOperator::BitwiseOr, Type::I64) => wfunc!(i64_bw_or, Type::I64),
                    (BinaryOperator::BitwiseOr, Type::U64) => wfunc!(i64_bw_or, Type::U64),


                    (BinaryOperator::BitwiseXor, Type::I8 ) => wfunc!( i8_bw_xor, Type::I8 ),
                    (BinaryOperator::BitwiseXor, Type::U8 ) => wfunc!( u8_bw_xor, Type::U8 ),
                    (BinaryOperator::BitwiseXor, Type::I16) => wfunc!(i16_bw_xor, Type::I16),
                    (BinaryOperator::BitwiseXor, Type::U16) => wfunc!(u16_bw_xor, Type::U16),
                    (BinaryOperator::BitwiseXor, Type::I32) => wfunc!(i32_bw_xor, Type::I32),
                    (BinaryOperator::BitwiseXor, Type::U32) => wfunc!(i32_bw_xor, Type::U32),
                    (BinaryOperator::BitwiseXor, Type::I64) => wfunc!(i64_bw_xor, Type::I64),
                    (BinaryOperator::BitwiseXor, Type::U64) => wfunc!(i64_bw_xor, Type::U64),


                    (BinaryOperator::Eq, _) => {
                        wasm.eq(lhs_anal.ty.to_wasm_ty(&self.types));
                        Type::BOOL
                    }

                    (BinaryOperator::Ne, _) => {
                        wasm.ne(lhs_anal.ty.to_wasm_ty(&self.types));
                        Type::BOOL
                    }

                    (BinaryOperator::Gt, Type::I8)    => wfunc!(i8_gt , Type::BOOL),
                    (BinaryOperator::Gt, Type::U8)    => wfunc!(u8_gt , Type::BOOL),
                    (BinaryOperator::Gt, Type::I16)   => wfunc!(i16_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::U16)   => wfunc!(u16_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::I32)   => wfunc!(i32_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::U32)   => wfunc!(u32_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::U64)   => wfunc!(u64_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::I64)   => wfunc!(i64_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::F32)   => wfunc!(f32_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::F64)   => wfunc!(f64_gt, Type::BOOL),

                    (BinaryOperator::Lt, Type::I8)    => wfunc!(i8_lt , Type::BOOL),
                    (BinaryOperator::Lt, Type::U8)    => wfunc!(u8_lt , Type::BOOL),
                    (BinaryOperator::Lt, Type::I16)   => wfunc!(i16_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::U16)   => wfunc!(u16_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::I32)   => wfunc!(i32_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::U32)   => wfunc!(u32_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::U64)   => wfunc!(u64_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::I64)   => wfunc!(i64_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::F32)   => wfunc!(f32_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::F64)   => wfunc!(f64_lt, Type::BOOL),

                    (BinaryOperator::Ge, Type::I8)    => wfunc!(i8_ge , Type::BOOL),
                    (BinaryOperator::Ge, Type::U8)    => wfunc!(u8_ge , Type::BOOL),
                    (BinaryOperator::Ge, Type::I16)   => wfunc!(i16_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::U16)   => wfunc!(u16_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::I32)   => wfunc!(i32_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::U32)   => wfunc!(u32_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::U64)   => wfunc!(u64_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::I64)   => wfunc!(i64_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::F32)   => wfunc!(f32_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::F64)   => wfunc!(f64_ge, Type::BOOL),

                    (BinaryOperator::Le, Type::I8)    => wfunc!(i8_le , Type::BOOL),
                    (BinaryOperator::Le, Type::U8)    => wfunc!(u8_le , Type::BOOL),
                    (BinaryOperator::Le, Type::I16)   => wfunc!(i16_le, Type::BOOL),
                    (BinaryOperator::Le, Type::U16)   => wfunc!(u16_le, Type::BOOL),
                    (BinaryOperator::Le, Type::I32)   => wfunc!(i32_le, Type::BOOL),
                    (BinaryOperator::Le, Type::U32)   => wfunc!(u32_le, Type::BOOL),
                    (BinaryOperator::Le, Type::U64)   => wfunc!(u64_le, Type::BOOL),
                    (BinaryOperator::Le, Type::I64)   => wfunc!(i64_le, Type::BOOL),
                    (BinaryOperator::Le, Type::F32)   => wfunc!(f32_le, Type::BOOL),
                    (BinaryOperator::Le, Type::F64)   => wfunc!(f64_le, Type::BOOL),

                    _ => panic!("op: {operator:?} lhs: {lhs_anal:?} rhs: {rhs_anal:?}"),
                };

                AnalysisResult::new(ty, true)
            },


            Expression::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(*rhs, scope, wasm, path);
                
                let mut type_check = || {
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(())
                    }

                    if rhs_anal.ty.eq_lit(Type::Never) { return Err(()) }

                    match operator {
                        UnaryOperator::Not if rhs_anal.ty.eq_lit(Type::BOOL) => Ok(()),
                        UnaryOperator::Neg if rhs_anal.ty.is_signed() => Ok(()),
                        _ => {
                            wasm.error(self.error(Error::InvalidUnaryOp {
                                operator, rhs: rhs_anal.ty, source
                            }));

                            Err(())
                        },
                    }
                };

                if type_check().is_err() {
                    wasm.pop();
                    wasm.unit();
                    return AnalysisResult::error();
                }

                match (operator, rhs_anal.ty) {
                    (UnaryOperator::Not, Type::Custom(x)) if x == TypeId::BOOL => {
                        wasm.bool_not()
                    },

                    (UnaryOperator::Neg, Type::I64) => {
                        // thanks wasm.
                        wasm.i64_const(-1);
                        wasm.i64_mul();
                    },

                    (UnaryOperator::Neg, Type::I8 | Type::I16 | Type::I32) => {
                        // thanks wasm.
                        wasm.i32_const(-1);
                        wasm.i32_mul();
                    },

                    (UnaryOperator::Neg, Type::F32) => wasm.f32_neg(),
                    (UnaryOperator::Neg, Type::F64) => wasm.f64_neg(),

                    _ => panic!("op: {operator:?} rhs: {rhs:?}")
                }

                AnalysisResult::new(rhs_anal.ty, true)
            },


            Expression::If { condition, body, else_block } => {
                let cond = self.expr(*condition, scope, wasm, path);

                if !cond.ty.eq_sem(Type::BOOL) {
                    wasm.pop();
                    wasm.error(self.error(Error::InvalidType {
                        source: condition.range(),
                        found: cond.ty, expected: Type::BOOL
                    }));

                    return AnalysisResult::error();
                }
                    
                if cond.ty.eq_lit(Type::Error) {
                    return AnalysisResult::error();
                }

                let ty = self.types.get(TypeId::BOOL);
                let TypeKind::Enum(e) = ty.kind() else { unreachable!() };
                e.get_tag(wasm);

                let mut slf = self;
                let (local, l, r) = wasm.lite(
                    &mut (&mut slf, scope),
                    |(slf, scope), wasm| {
                        let body = slf.expr(*body, *scope, wasm, path);
                        let wty = body.ty.to_wasm_ty(&slf.types);
                        let local = wasm.local(wty);
                        (local, Some((body, wasm.offset())))
                    },

                    |((slf, scope), _), wasm| {
                        if let Some(else_block) = else_block {
                            return Some((slf.expr(*else_block, *scope, wasm, path), wasm.offset()))
                        }

                        None
                    }
                );

                let l = l.unwrap();

                if r.is_none() && !l.0.ty.eq_sem(Type::Unit) {
                    wasm.error(slf.error(Error::IfMissingElse { body: (body.range(), l.0.ty) }));
                    wasm.insert_drop(l.1);
                    return AnalysisResult::error();
                }
                
                if r.is_none() {
                    wasm.insert_drop(l.1);

                } else if r.is_some() && !l.0.ty.eq_sem(r.as_ref().unwrap().0.ty) {
                    wasm.error(slf.error(Error::IfBodyAndElseMismatch {
                        body: (body.range(), l.0.ty),
                        else_block: (else_block.unwrap().range(), r.as_ref().unwrap().0.ty)
                    }));

                    let i = wasm.insert_drop(l.1);
                    wasm.insert_drop(r.unwrap().1 + i);
                    return AnalysisResult::error()

                } else {
                    let i = wasm.insert_local_set(l.1, local);
                    if let Some(r) = r {
                        wasm.insert_local_set(r.1 + i, local);
                    }

                }

                l.0
            },


            Expression::Match { value, taken_as_inout, mappings } => {
                let anal = self.expr(*value, scope, wasm, path);
                if taken_as_inout && !anal.is_mut {
                    wasm.error(self.error(Error::InOutValueIsntMut(value.range())));
                    return AnalysisResult::error();
                }

                let sym_local = wasm.local(anal.ty.to_wasm_ty(&self.types));
                wasm.local_set(sym_local);

                let tyid = match anal.ty {
                    Type::Custom(v) => v,

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::MatchValueIsntEnum {
                            source: value.range(), typ: anal.ty }));
                        return AnalysisResult::error();
                    }
                };

                let ty = self.types.get(tyid); 
                let TypeKind::Enum(sym) = ty.kind()
                else { 
                    wasm.error(self.error(Error::MatchValueIsntEnum {
                        source: value.range(), typ: anal.ty }));
                    return AnalysisResult::error();
                };

                wasm.local_get(sym_local);

                let tag = wasm.i32_temp();
                sym.kind().get_tag(wasm);

                wasm.local_set(tag);

                for (i, m) in mappings.iter().enumerate() {
                    if let Some(e) = mappings[(i + 1)..].iter().find(|x| x.name() == m.name()) {
                        wasm.error(self.error(Error::DuplicateMatch {
                            declared_at: m.range(), error_point: e.range() }))
                    }
                }

                match sym.kind() {
                    TypeEnumKind::TaggedUnion(v) => {
                        for m in mappings.iter() {
                            if !v.fields().iter().any(|x| x.name() == m.name()) {
                                wasm.error(self.error(Error::InvalidMatch { 
                                    name: m.name(), range: m.range(), value: anal.ty }));
                            }
                        }

                        let mut vec = Vec::new();
                        for m in v.fields().iter() {
                            if !mappings.iter().any(|x| x.name() == m.name()) {
                                vec.push(m.name());
                            }
                        }
                        if !vec.is_empty() {
                            wasm.error(self.error(Error::MissingMatch { name: vec, range: source }));
                        }

                    },

                    TypeEnumKind::Tag(v) => {
                        for m in mappings.iter() {
                            if !v.fields().contains(&m.name()) {
                                wasm.error(self.error(Error::InvalidMatch { 
                                    name: m.name(), range: m.range(), value: anal.ty }));
                            }
                        }

                        let mut vec = Vec::new();
                        for m in v.fields().iter() {
                            if !mappings.iter().any(|x| x.name() == *m) {
                                vec.push(*m);
                            }
                        }
                        if !vec.is_empty() {
                            wasm.error(self.error(Error::MissingMatch { name: vec, range: source }));
                        }
                    },
                }

                {
                    fn do_mapping<'ast>(
                        anal: &mut Analyzer<'_, '_, '_>,
                        wasm: &mut WasmFunctionBuilder,
                        mappings: &[MatchMapping<'ast>],
                        enum_sym: TypeEnum,
                        sym_local: LocalId,
                        tag: LocalId,
                        taken_as_inout: bool,
                        value_range: SourceRange,
                        scope: ScopeId,
                        path: StringIndex,

                        index: usize,
                    ) -> Option<(Type, LocalId, SourceRange)> {
                        let Some(mapping) = mappings.get(index)
                        else {
                            wasm.block(|wasm, _| {
                                wasm.local_get(tag);
                                
                                let mut string = format_in!(anal.output, "br_table ");
 
                                for i in (0..mappings.len()).rev() {
                                    let _ = write!(string, "{} ", i);
                                }

                                let _ = write!(string, "{} ", 0);

                                wasm.raw(&string);
                            });

                            return None;
                        };

                        let mut final_result = None;
                        wasm.block(|wasm, _| {
                            let result = do_mapping(anal, wasm, mappings, enum_sym, sym_local, tag,
                                                    taken_as_inout, value_range, scope, path, index + 1);

                            if mapping.is_inout() && !taken_as_inout {
                                wasm.error(anal.error(Error::InOutBindingWithoutInOutValue {
                                    value_range }))
                            }

                            let (ty, local) = match enum_sym.kind() {
                                TypeEnumKind::TaggedUnion(v) => {
                                    let emapping = v.fields().iter().find(|x| x.name() == mapping.name()).unwrap();
                                    let ty = emapping.ty().unwrap_or(Type::Unit);
                                    let wty = ty.to_wasm_ty(&anal.types);
                                    let local = wasm.local(wty);

                                    wasm.local_get(sym_local);
                                    wasm.u32_const(v.union_offset());
                                    wasm.i32_add();

                                    wasm.read(wty);
                                    wasm.local_set(local);
                                    (ty, local)
                                },

                                TypeEnumKind::Tag(_) => {
                                    let local = wasm.local(Type::Unit.to_wasm_ty(&anal.types));
                                    (Type::Unit, local)
                                },
                            };

                            let var_scope = VariableScope::new(
                                mapping.binding(), mapping.is_inout(),
                                ty, local
                            );

                            let scope = Scope::new(ScopeKind::Variable(var_scope), scope.some());
                            let scope = anal.scopes.push(scope);

                            let analysis = anal.expr(mapping.node(), scope, wasm, path);

                            final_result = Some(if let Some((mut ty, mut local, src)) = result {
                                if ty.eq_lit(Type::Never) || ty.eq_lit(Type::Never) {
                                    ty = analysis.ty;
                                    local = wasm.local(ty.to_wasm_ty(&anal.types));
                                }

                                if analysis.ty.eq_lit(ty) {
                                    wasm.local_set(local);
                                } else {
                                    wasm.pop();
                                    wasm.error(anal.error(Error::MatchBranchesDifferInReturnType {
                                        initial_source: src, initial_typ: ty,
                                        branch_source: mapping.range(), branch_typ: analysis.ty }));
                                }
                                
                                let ty = if ty.eq_lit(Type::Error) { analysis.ty } else { ty };
                                (ty, local, src)
                            } else {
                                let local = wasm.local(analysis.ty.to_wasm_ty(&anal.types));
                                wasm.local_set(local);

                                (analysis.ty, local, mapping.range())
                            });

                        });
                        Some(final_result.unwrap())
                    }

                    let result = do_mapping(self, wasm, mappings, sym, sym_local, tag, taken_as_inout,
                                            value.range(), scope, path, 0);

                    if let Some(result) = result {
                        wasm.local_get(result.1);
                        AnalysisResult::new(result.0, true)
                    } else {
                        wasm.unit();
                        AnalysisResult::new(Type::Unit, true)
                    }
                }
            },

            Expression::Block { block } => self.block(wasm, path, scope, &*block).0,

            Expression::CreateStruct { data_type, fields } => {
                let ty = match self.convert_ty(scope, data_type) {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error()
                    },
                };


                let tyid = match ty {
                    Type::Custom(v) => v,

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::StructCreationOnNonStruct {
                            source, typ: ty }));
                        return AnalysisResult::error();
                    }
                };

                let ty = self.create_struct(wasm, scope, tyid, fields, path, source);
                AnalysisResult::new(ty, true)
            },
            
            
            Expression::AccessField { val, field_name } => {
                let value = self.expr(*val, scope, wasm, path);

                let tyid = match value.ty {
                    Type::Custom(v) => v,

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::FieldAccessOnNonEnumOrStruct {
                            source, typ: value.ty }));
                        return AnalysisResult::error();
                    }
                };


                let strct = self.types.get(tyid);
                let TypeKind::Struct(TypeStruct { fields: sfields, .. }) = strct.kind()
                else {
                    wasm.error(self.error(Error::FieldAccessOnNonEnumOrStruct {
                        source, typ: value.ty }));
                    return AnalysisResult::error();
                };
                
                for f in sfields.iter() {
                    if f.0.name == field_name {
                        wasm.u32_const(f.1.try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(f.0.ty.to_wasm_ty(&self.types));
                        return AnalysisResult::new(f.0.ty, value.is_mut);
                    }
                }

                wasm.error(self.error(Error::FieldDoesntExist {
                    source, field: field_name, typ: value.ty }));

                AnalysisResult::error()
            },


            Expression::CallFunction { name, is_accessor, args } => {
                let mut scope_id = scope;
                let pool = ArenaPool::tls_get_rec();
                let aargs = {
                    let mut vec = Vec::new_in(&*pool);
                    
                    for (i, a) in args.iter().enumerate() {
                        let mut a = *a;
                        if i == 0 && is_accessor { a.1 = true }
                        let scope = scope_id;
                        let anal = self.expr(a.0, scope, wasm, path);

                        if a.1 && !anal.is_mut && i != 0 {
                            wasm.error(self.error(Error::InOutValueIsntMut(a.0.range())))
                        }

                        vec.push((anal.ty, a.0.range(), if a.1 { Some(a.0) } else { None }));
                    }

                    vec
                };

                if is_accessor {
                    let ty = aargs[0].0;
                    let ns = self.namespaces.get_type(ty, &self.types);
                    scope_id = self.scopes.push(
                        Scope::new(ScopeKind::ImplicitNamespace(ns), scope_id.some()));
                }

                let scope = self.scopes.get(scope_id);
                let Some(func) = scope.get_func(name, &self.scopes, &self.namespaces)
                else {
                    wasm.error(self.error(Error::FunctionNotFound { source, name }));
                    return AnalysisResult::error();
                };

                let ty = self.call_func(
                    func, 
                    aargs.leak(),
                    is_accessor,
                    source,
                    scope_id,
                    wasm
                );

                let ty = match ty {
                    Ok(v) => v,
                    Err(_) => Type::Error,
                };
                
                AnalysisResult::new(ty, true)
            },


            Expression::WithinNamespace { namespace, namespace_source, action } => {
                let Some(ns) = self.scopes.get(scope).get_ns(namespace, &self.scopes, &mut self.namespaces, &self.types)
                else {
                    wasm.error(self.error(Error::NamespaceNotFound 
                                          { source: namespace_source, namespace }));
                    return AnalysisResult::error();
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let scope = self.scopes.push(scope);
                self.expr(*action, scope, wasm, path)
            },


            Expression::WithinTypeNamespace { namespace, action } => {
                let namespace = self.convert_ty(scope, namespace);
                let namespace = match namespace {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error();
                    },
                };

                let namespace = self.namespaces.get_type(namespace, &self.types);
                let scope = Scope::new(ScopeKind::ImplicitNamespace(namespace), scope.some());
                let scope = self.scopes.push(scope);
                self.expr(*action, scope, wasm, path)
            },

            
            Expression::Loop { body } => {
                wasm.do_loop(|wasm, id| {
                    let nscope = LoopScope::new(id);
                    let nscope = Scope::new(ScopeKind::Loop(nscope), scope.some());
                    let nscope = self.scopes.push(nscope);

                    let (anal, _) = self.block(wasm, path, nscope, &*body);
                    self.acquire(anal.ty, wasm);
                    self.drop_value(anal.ty, wasm);
                });

                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },


            Expression::Return(v) => {
                let value = self.expr(*v, scope, wasm, path);

                let func_return = {
                    let scope = self.scopes.get(scope);
                    match scope.get_func_def(&self.scopes) {
                        Some(v) => v,
                        None => {
                            wasm.error(self.error(Error::ReturnOutsideOfAFunction { source }));
                            return AnalysisResult::error()
                        },
                    }
                };

                if !func_return.return_type.eq_sem(value.ty) {
                    wasm.error(self.error(Error::ReturnAndFuncTypDiffer {
                        source, func_source: func_return.return_source,
                        typ: value.ty, func_typ: func_return.return_type }));

                    return AnalysisResult::error()
                }

                wasm.ret();
                AnalysisResult::new(Type::Never, true)
            },


            Expression::Continue => {
                let loop_val = {
                    let scope = self.scopes.get(scope);
                    match scope.get_loop(&self.scopes) {
                        Some(v) => v,
                        None => {
                            wasm.error(self.error(Error::ContinueOutsideOfLoop(source)));
                            return AnalysisResult::error()
                        },
                    }
                };

                wasm.continue_loop(loop_val.loop_id);
                AnalysisResult::new(Type::Never, true)
            },


            Expression::Break => {
                let loop_val = {
                    let scope = self.scopes.get(scope);
                    match scope.get_loop(&self.scopes) {
                        Some(v) => v,
                        None => {
                            wasm.error(self.error(Error::BreakOutsideOfLoop(source)));
                            return AnalysisResult::error()
                        },
                    }
                };

                wasm.break_loop(loop_val.loop_id);
                AnalysisResult::new(Type::Never, true)
            },


            Expression::Tuple(v) => {
                let pool = ArenaPool::tls_get_rec();
                let mut vec = Vec::with_cap_in(&*pool, v.len());
                for n in v.iter() {
                    let anal = self.expr(*n, scope, wasm, path);
                    self.acquire(anal.ty, wasm);
                    vec.push(anal.ty);
                }

                let ty_id = match self.tuple_map.get(&*vec) {
                    Some(v) => *v,
                    None => {
                        self.make_tuple(vec.clone_in(&*pool), source)
                    },
                };

                dbg!(ty_id, &self.types);
                let ty = self.types.get(ty_id);
                let TypeKind::Struct(sym) = ty.kind() else { unreachable!() };
                let ptr = wasm.alloc_stack(ty.size());

                let mut errored = false;
                for ((sf, f), n) in sym.fields.iter().zip(vec.iter()).zip(v.iter()).rev() {
                    if !sf.0.ty.eq_sem(*f) {
                        errored = true;
                        wasm.pop();
                        wasm.error(self.error(Error::InvalidType {
                            source: n.range(), found: *f, expected: sf.0.ty }));
                        continue
                    }

                    let ptr = ptr.add(sf.1);
                    wasm.sptr_const(ptr);
                    wasm.write(sf.0.ty.to_wasm_ty(&self.types));
                }

                if errored { return AnalysisResult::error() }

                wasm.sptr_const(ptr);
                AnalysisResult::new(Type::Custom(ty_id), true)
            },


            Expression::AsCast { lhs, data_type } => {
                let lhs_anal = self.expr(*lhs, scope, wasm, path);
                let dty = self.convert_ty(scope, data_type);
                let dty = match dty {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error()
                    }
                };


                match (lhs_anal.ty, dty) {
                    | (Type::Error, _)
                    | (_, Type::Error) => return AnalysisResult::error(),

                    (Type::I8 , Type::I16) => wasm.i8_as_i16(),
                    (Type::I8 , Type::I32) => wasm.i8_as_i32(),
                    (Type::I8 , Type::I64) => wasm.i8_as_i64(),
                    (Type::I8 , Type::U8 ) => wasm.i8_as_u8(),
                    (Type::I8 , Type::U16) => wasm.i8_as_u16(),
                    (Type::I8 , Type::U32) => wasm.i8_as_u32(),
                    (Type::I8 , Type::U64) => wasm.i8_as_u64(),
                    (Type::I8 , Type::F32) => wasm.i8_as_f32(),
                    (Type::I8 , Type::F64) => wasm.i8_as_f64(),

                    (Type::I16, Type::I8 ) => wasm.i16_as_i8(),
                    (Type::I16, Type::I32) => wasm.i16_as_i32(),
                    (Type::I16, Type::I64) => wasm.i16_as_i64(),
                    (Type::I16, Type::U8 ) => wasm.i16_as_u8(),
                    (Type::I16, Type::U16) => wasm.i16_as_u16(),
                    (Type::I16, Type::U32) => wasm.i16_as_u32(),
                    (Type::I16, Type::U64) => wasm.i16_as_u64(),
                    (Type::I16, Type::F32) => wasm.i16_as_f32(),
                    (Type::I16, Type::F64) => wasm.i16_as_f64(),

                    (Type::I32, Type::I8 ) => wasm.i32_as_i8(),
                    (Type::I32, Type::I16) => wasm.i32_as_i16(),
                    (Type::I32, Type::I64) => wasm.i32_as_i64(),
                    (Type::I32, Type::U8 ) => wasm.i32_as_u8(),
                    (Type::I32, Type::U16) => wasm.i32_as_u16(),
                    (Type::I32, Type::U32) => wasm.i32_as_u32(),
                    (Type::I32, Type::U64) => wasm.i32_as_u64(),
                    (Type::I32, Type::F32) => wasm.i32_as_f32(),
                    (Type::I32, Type::F64) => wasm.i32_as_f64(),

                    (Type::I64, Type::I8 ) => wasm.i64_as_i8(),
                    (Type::I64, Type::I16) => wasm.i64_as_i16(),
                    (Type::I64, Type::I32) => wasm.i64_as_i32(),
                    (Type::I64, Type::U8 ) => wasm.i64_as_u8(),
                    (Type::I64, Type::U16) => wasm.i64_as_u16(),
                    (Type::I64, Type::U32) => wasm.i64_as_u32(),
                    (Type::I64, Type::U64) => wasm.i64_as_u64(),
                    (Type::I64, Type::F32) => wasm.i64_as_f32(),
                    (Type::I64, Type::F64) => wasm.i64_as_f64(),

                    (Type::U8 , Type::I8 ) => wasm.u8_as_i8(),
                    (Type::U8 , Type::I16) => wasm.u8_as_i16(),
                    (Type::U8 , Type::I32) => wasm.u8_as_i32(),
                    (Type::U8 , Type::I64) => wasm.u8_as_i64(),
                    (Type::U8 , Type::U16) => wasm.u8_as_u16(),
                    (Type::U8 , Type::U32) => wasm.u8_as_u32(),
                    (Type::U8 , Type::U64) => wasm.u8_as_u64(),
                    (Type::U8 , Type::F32) => wasm.u8_as_f32(),
                    (Type::U8 , Type::F64) => wasm.u8_as_f64(),

                    (Type::U16, Type::I8 ) => wasm.u16_as_i8(),
                    (Type::U16, Type::I16) => wasm.u16_as_i16(),
                    (Type::U16, Type::I32) => wasm.u16_as_i32(),
                    (Type::U16, Type::I64) => wasm.u16_as_i64(),
                    (Type::U16, Type::U8 ) => wasm.u16_as_u8(),
                    (Type::U16, Type::U32) => wasm.u16_as_u32(),
                    (Type::U16, Type::U64) => wasm.u16_as_u64(),
                    (Type::U16, Type::F32) => wasm.u16_as_f32(),
                    (Type::U16, Type::F64) => wasm.u16_as_f64(),

                    (Type::U32, Type::I8 ) => wasm.u32_as_i8(),
                    (Type::U32, Type::I16) => wasm.u32_as_i16(),
                    (Type::U32, Type::I32) => wasm.u32_as_i32(),
                    (Type::U32, Type::I64) => wasm.u32_as_i64(),
                    (Type::U32, Type::U8 ) => wasm.u32_as_u8(),
                    (Type::U32, Type::U16) => wasm.u32_as_u16(),
                    (Type::U32, Type::U64) => wasm.u32_as_u64(),
                    (Type::U32, Type::F32) => wasm.u32_as_f32(),
                    (Type::U32, Type::F64) => wasm.u32_as_f64(),

                    (Type::U64, Type::I8 ) => wasm.u64_as_i8(),
                    (Type::U64, Type::I16) => wasm.u64_as_i16(),
                    (Type::U64, Type::I32) => wasm.u64_as_i32(),
                    (Type::U64, Type::I64) => wasm.u64_as_i64(),
                    (Type::U64, Type::U8 ) => wasm.u64_as_u8(),
                    (Type::U64, Type::U16) => wasm.u64_as_u16(),
                    (Type::U64, Type::U32) => wasm.u64_as_u32(),
                    (Type::U64, Type::F32) => wasm.u64_as_f32(),
                    (Type::U64, Type::F64) => wasm.u64_as_f64(),

                    (Type::F32, Type::I8 ) => wasm.f32_as_i8(),
                    (Type::F32, Type::I16) => wasm.f32_as_i16(),
                    (Type::F32, Type::I32) => wasm.f32_as_i32(),
                    (Type::F32, Type::I64) => wasm.f32_as_i64(),
                    (Type::F32, Type::U8 ) => wasm.f32_as_u8(),
                    (Type::F32, Type::U16) => wasm.f32_as_u16(),
                    (Type::F32, Type::U32) => wasm.f32_as_u32(),
                    (Type::F32, Type::U64) => wasm.f32_as_u64(),
                    (Type::F32, Type::F64) => wasm.f32_as_f64(),

                    (Type::F64, Type::I8 ) => wasm.f64_as_i8(),
                    (Type::F64, Type::I16) => wasm.f64_as_i16(),
                    (Type::F64, Type::I32) => wasm.f64_as_i32(),
                    (Type::F64, Type::I64) => wasm.f64_as_i64(),
                    (Type::F64, Type::U8 ) => wasm.f64_as_u8(),
                    (Type::F64, Type::U16) => wasm.f64_as_u16(),
                    (Type::F64, Type::U32) => wasm.f64_as_u32(),
                    (Type::F64, Type::U64) => wasm.f64_as_u64(),
                    (Type::F64, Type::F32) => wasm.f64_as_f32(),


                    | (Type::I8 , Type::I8 )
                    | (Type::I16, Type::I16)
                    | (Type::I32, Type::I32)
                    | (Type::I64, Type::I64)
                    | (Type::U8 , Type::U8 )
                    | (Type::U16, Type::U16)
                    | (Type::U32, Type::U32)
                    | (Type::U64, Type::U64)
                    | (Type::F32, Type::F32)
                    | (Type::F64, Type::F64) => (),

                    (Type::I64, Type::BOOL) => {
                        wasm.i64_as_i32();
                        let local = wasm.i32_temp();
                        wasm.ite(
                            |wasm| {
                                wasm.i32_const(1);
                                wasm.local_set(local);
                            }, 
                            |wasm| {
                                wasm.i32_const(0);
                                wasm.local_set(local);
                            }
                        );

                        wasm.local_get(local);
                    },

                    (Type::BOOL, Type::I64) => {
                        let local = wasm.i64_temp();
                        wasm.ite(
                            |wasm| {
                                wasm.i64_const(1);
                                wasm.local_set(local);
                            }, 
                            |wasm| {
                                wasm.i64_const(0);
                                wasm.local_set(local);
                            }
                        );
                        wasm.local_get(local);
                    }

                    _ => {
                        wasm.error(self.error(Error::InvalidCast {
                            range: source, from_ty: lhs_anal.ty, to_ty: dty }))
                    }
                }

                AnalysisResult::new(dty, true)
            },


            Expression::Unwrap(v) => {
                let anal = self.expr(*v, scope, wasm, path);
                let ty = match anal.ty {
                    Type::Custom(v) => self.types.get(v),

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::CantUnwrapOnGivenType(v.range(), anal.ty)));
                        return AnalysisResult::error();
                    }
                };
                
                let TypeKind::Enum(e) = ty.kind()
                else {
                    wasm.error(self.error(Error::CantUnwrapOnGivenType(v.range(), anal.ty)));
                    return AnalysisResult::error();
                };

                if !matches!(e.status(), TypeEnumStatus::Option | TypeEnumStatus::Result) {
                    wasm.error(self.error(Error::CantUnwrapOnGivenType(v.range(), anal.ty)));
                    return AnalysisResult::error();
                }

                let dup = wasm.local(anal.ty.to_wasm_ty(&self.types));
                wasm.local_tee(dup);
                
                // 0: Some/Ok
                // 1: None/Err
                e.kind().get_tag(wasm);

                let mut ret_ty = Type::Unit;
                wasm.ite(
                |wasm| {
                    wasm.panic("unwrapped on none value");

                    let ty = match e.kind() {
                        TypeEnumKind::TaggedUnion(v) => v.fields()[0].ty().unwrap_or(Type::Unit),
                        TypeEnumKind::Tag(_) => Type::Unit,
                    };

                    ret_ty = ty;
                },

                |_| {}
                );

                'b: { match e.kind() {
                    TypeEnumKind::TaggedUnion(v) => {
                        let ty = v.fields()[0].ty().unwrap_or(Type::Unit);
                        if ty == Type::Unit {
                            wasm.unit(); 
                            break 'b;
                        }

                        wasm.local_get(dup);
                        wasm.u32_const(v.union_offset().try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(ret_ty.to_wasm_ty(&self.types));
                    },

                    TypeEnumKind::Tag(_) => wasm.unit(),
                }};

                AnalysisResult::new(ret_ty, true)
            },


            Expression::OrReturn(v) => {
                let anal = self.expr(*v, scope, wasm, path);
                let ty = match anal.ty {
                    Type::Custom(v) => self.types.get(v),

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::CantTryOnGivenType(v.range(), anal.ty)));
                        return AnalysisResult::error();
                    }
                };
                
                let TypeKind::Enum(enum_sym) = ty.kind()
                else {
                    wasm.error(self.error(Error::CantTryOnGivenType(v.range(), anal.ty)));
                    return AnalysisResult::error();
                };

                if !matches!(enum_sym.status(), TypeEnumStatus::Option | TypeEnumStatus::Result) {
                    wasm.error(self.error(Error::CantTryOnGivenType(v.range(), anal.ty)));
                    return AnalysisResult::error();
                }

                let func = self.scopes.get(scope).get_func_def(&self.scopes).unwrap();

                let mut err = |anal: &mut Self| {
                    if enum_sym.status() == TypeEnumStatus::Result {
                        wasm.error(anal.error(Error::FunctionDoesntReturnAResult {
                            source, func_typ: func.return_type }));

                    } else if enum_sym.status() == TypeEnumStatus::Option {
                        wasm.error(anal.error(Error::FunctionDoesntReturnAnOption {
                            source, func_typ: func.return_type }));

                    } else { unreachable!() }
                    return AnalysisResult::error();
                };

                let TypeKind::Enum(func_sym) = ty.kind()
                else {
                    return err(self);
                };

                if !matches!(func_sym.status(), TypeEnumStatus::Option | TypeEnumStatus::Result) {
                    return err(self);
                }

                let dup = wasm.local(anal.ty.to_wasm_ty(&self.types));
                wasm.local_tee(dup);

                // 0: Some/Ok
                // 1: None/Err
                enum_sym.kind().get_tag(wasm);

                let mut ret_ty = Type::Unit;
                wasm.ite(
                |wasm| {
                    match func_sym.status() {
                        TypeEnumStatus::Option => {
                            let some_val = match enum_sym.kind() {
                                TypeEnumKind::TaggedUnion(v) => v.fields()[0].ty().unwrap_or(Type::Unit),
                                TypeEnumKind::Tag(_) => Type::Unit,
                            };

                            ret_ty = some_val;

                            // Check the functions return signature for failure
                            {
                                if func_sym.status() != TypeEnumStatus::Option {
                                    wasm.error(self.error(Error::FunctionDoesntReturnAnOption {
                                        source, func_typ: func.return_type }));
                                    return;
                                }
                            }

                            // Codegen
                            'l: {
                                let ns = self.namespaces.get_type(func.return_type, &self.types);
                                let Some(ns) = self.namespaces.get(ns)
                                else { break 'l };

                                let Some(call_func) = ns.get_func(StringMap::NONE)
                                else { break 'l };
                                let call_func = self.funcs.get(call_func).unwrap();
                                let FunctionKind::UserDefined { .. } = call_func.kind
                                else { unreachable!() };

                                let func_ret_wasm_ty = func.return_type.to_wasm_ty(&self.types);
                                if func_ret_wasm_ty.stack_size() != 0 {
                                    let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                                    wasm.sptr_const(alloc);
                                    wasm.call(call_func.func);

                                    wasm.sptr_const(alloc);
                                    wasm.ret();
                                } else {
                                    wasm.call(call_func.func);
                                    wasm.ret();
                                }
                            }
                        },


                        TypeEnumStatus::Result => {
                            let (ok, err) = match enum_sym.kind() {
                                TypeEnumKind::TaggedUnion(v) => (
                                    v.fields()[0].ty().unwrap_or(Type::Unit),
                                    v.fields()[1].ty().unwrap_or(Type::Unit),
                                ),
                                TypeEnumKind::Tag(_) => (Type::Unit, Type::Unit),
                            };

                            ret_ty = ok;

                            // Check the functions return signature for failure
                            {
                                if func_sym.status() != TypeEnumStatus::Result {
                                    wasm.error(self.error(Error::FunctionDoesntReturnAResult {
                                        source, func_typ: func.return_type }));
                                    return;
                                }

                                let ferr = match func_sym.kind() {
                                    TypeEnumKind::TaggedUnion(v) => 
                                        v.fields()[1].ty().unwrap_or(Type::Unit),
                                    TypeEnumKind::Tag(_) => Type::Unit,
                                };

                                if !ferr.eq_sem(err) {
                                    wasm.error(self.error(Error::FunctionReturnsAResultButTheErrIsntTheSame { 
                                        source, func_source: func.return_source, 
                                        func_err_typ: ferr, err_typ: err }));
                                    return;
                                }
                            }

                            // Codegen
                            'l: {
                                let ns = self.namespaces.get_type(func.return_type, &self.types);
                                let Some(ns) = self.namespaces.get(ns)
                                else { break 'l };
                                let Some(call_func) = ns.get_func(StringMap::ERR)
                                else { break 'l };
                                let call_func = self.funcs.get(call_func).unwrap();
                                let FunctionKind::UserDefined { .. } = call_func.kind
                                else { unreachable!() };

                                // Get the error value
                                {
                                    match enum_sym.kind() {
                                        TypeEnumKind::TaggedUnion(v) => {
                                            let ty = v.fields()[1].ty().unwrap_or(Type::Unit);

                                            wasm.local_get(dup);
                                            wasm.u32_const(v.union_offset().try_into().unwrap());
                                            wasm.i32_add();
                                            wasm.read(ty.to_wasm_ty(&self.types));
                                        },

                                        TypeEnumKind::Tag(_) => wasm.unit(),
                                    }
                                }

                                let func_ret_wasm_ty = func.return_type.to_wasm_ty(&self.types);
                                if func_ret_wasm_ty.stack_size() != 0 {
                                    let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                                    wasm.sptr_const(alloc);
                                    wasm.call(call_func.func);

                                    wasm.sptr_const(alloc);
                                    wasm.ret();
                                } else {
                                    wasm.call(call_func.func);
                                    wasm.ret();
                                }
                            }

                            },
                            
                        _ => unreachable!(),
                    }
                    },

                    |_| {}
                );

                'l: { match enum_sym.kind() {
                    TypeEnumKind::TaggedUnion(v) => {
                        let ty = v.fields()[0].ty().unwrap_or(Type::Unit);
                        if ty == Type::Unit {
                            wasm.unit();
                            break 'l
                        }

                        wasm.local_get(dup);
                        wasm.u32_const(v.union_offset().try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(ty.to_wasm_ty(&self.types));
                    },

                    TypeEnumKind::Tag(_) => wasm.unit(),
                } };

                AnalysisResult::new(ret_ty, true)
            },


            Expression::Range { lhs, rhs } => {
                let fields = [
                    (StringMap::LOW, lhs.range(), *lhs),
                    (StringMap::HIGH, rhs.range(), *rhs),
                ];

                self.create_struct(wasm, scope, TypeId::RANGE, &fields, path, source);

                AnalysisResult::new(Type::RANGE, true)

            },


            Expression::Deref(val) => {
                let val_anal = self.expr(*val, scope, wasm, path);

                let mut is_ptr = || {
                    let Type::Custom(ty_id) = val_anal.ty 
                    else { return None };

                    let ty = self.types.get(ty_id);
                    let TypeKind::Struct(sym) = ty.kind()
                    else { return None };

                    if sym.status != TypeStructStatus::Ptr { return None };

                    let field = sym.fields[1];
                    // the ptr is on the stack
                    wasm.u32_const(field.1 as u32);
                    wasm.i32_add();

                    wasm.read(field.0.ty.to_wasm_ty(&self.types));

                    Some(field.0.ty)
                };

                match is_ptr() {
                    Some(v) => {
                        AnalysisResult::new(v, true)
                    },
                    None => {
                        wasm.pop();
                        wasm.error(self.error(Error::DerefOnNonPtr(source)));
                        AnalysisResult::error()
                    },
                }
            },
        }
    }

    ///
    /// `$val_ty` -> ()
    ///
    fn assign(
        &mut self, 
        wasm: &mut WasmFunctionBuilder,
        scope: ScopeId, 
        node: ExpressionNode,
        val_ty: Type,
        depth: usize
    ) -> Result<Type, Error> {
        match node.kind() {
            Expression::Identifier(ident) => {
                let Some(variable) = self.scopes.get(scope).get_var(ident, &self.scopes)
                else {
                    return Err(Error::VariableNotFound { name: ident, source: node.range() });
                };

                if !variable.is_mutable {
                    return Err(Error::ValueUpdateNotMut { source: node.range() });
                }

                if depth != 0 {
                    wasm.local_get(variable.local_id);
                    return Ok(variable.ty)
                }

                if !variable.ty.eq_sem(val_ty) {
                    return Err(Error::ValueUpdateTypeMismatch 
                               { lhs: variable.ty, rhs: val_ty, source: node.range() })
                }
                
                let val_wasm = val_ty.to_wasm_ty(&self.types);
                let value_local = wasm.local(val_wasm);
                wasm.local_set(value_local);

                // get it to drop it later.
                wasm.local_get(variable.local_id);
                if let Type::Custom(_) = variable.ty {
                    wasm.read(val_wasm);
                }

                // set the new
                wasm.local_get(value_local);
                wasm.local_set(variable.local_id);

                // drop it
                self.drop_value(variable.ty, wasm);


                Ok(variable.ty)
            }

            
            Expression::AccessField { val, field_name } => {
                let ty = self.assign(wasm, scope, *val, val_ty, depth + 1)?;

                let tyid = match ty {
                    Type::Custom(v) => v,

                    Type::Error => { wasm.pop(); return Err(Error::Bypass) },

                    _ => {
                        wasm.pop();
                        return Err(Error::FieldAccessOnNonEnumOrStruct {
                            source: node.range(), typ: ty });
                    }
                };


                let strct = self.types.get(tyid);
                let TypeKind::Struct(TypeStruct { fields: sfields, .. }) = strct.kind()
                else {
                    return Err(Error::FieldAccessOnNonEnumOrStruct {
                        source: node.range(), typ: ty });
                };

                for sf in sfields.iter() {
                    if sf.0.name == field_name {
                        wasm.u32_const(sf.1.try_into().unwrap());
                        wasm.i32_add();

                        if depth != 0 {
                            return Ok(sf.0.ty);
                        }

                        if !sf.0.ty.eq_sem(val_ty) {
                            wasm.pop();
                            return Err(Error::ValueUpdateTypeMismatch 
                                       { lhs: sf.0.ty, rhs: val_ty, source: node.range() })
                        }

                        let val_ty_wasm = val_ty.to_wasm_ty(&self.types);
                        let local = wasm.local(WasmType::I32);
                        wasm.local_set(local);
                        // so the local is now set to the pointer
                        
                        // get it to drop it later!
                        wasm.local_get(local);
                        wasm.read(val_ty_wasm);

                        let old_data_local = wasm.local(val_ty_wasm);
                        wasm.local_set(old_data_local);

                        // set the new
                        wasm.local_get(local);
                        wasm.write(val_ty_wasm);

                        // drop it
                        wasm.local_get(old_data_local);
                        self.drop_value(val_ty, wasm);

                        return Ok(sf.0.ty);
                    }
                }

                Err(Error::FieldDoesntExist {
                    source: node.range(), field: field_name, typ: ty })
            }


            Expression::Deref(node) => {
                let ty = self.assign(wasm, scope, *node, val_ty, depth + 1)?;

                let mut is_ptr = || {
                    let Type::Custom(ty_id) = ty
                    else { return None };

                    let ty = self.types.get(ty_id);
                    let TypeKind::Struct(sym) = ty.kind()
                    else { return None };

                    if sym.status != TypeStructStatus::Ptr { return None };

                    let field = sym.fields[1];

                    // let Some(local) = self.make_mut(Type::Custom(ty_id), wasm)
                    // else { unreachable!() };

                    // wasm.local_get(local);
                    // self.assign(wasm, scope, *node, Type::Custom(ty_id), depth).unwrap();

                    // get the ptr to the value
                    wasm.u32_const(field.1 as u32);
                    wasm.i32_add();

                    if depth != 0 {
                        return Some(Ok(field.0.ty))
                    }

                    if !field.0.ty.eq_sem(val_ty) {
                        return Some(Err(Error::ValueUpdateTypeMismatch 
                                   { lhs: field.0.ty, rhs: val_ty,
                                   source: node.range() }))
                    }

                    let val_ty_wasm = val_ty.to_wasm_ty(&self.types);
                    let local = wasm.i32_temp();
                    wasm.local_set(local);

                    // get it to drop later
                    wasm.local_get(local);
                    wasm.read(val_ty_wasm);

                    let old_data_local = wasm.local(val_ty_wasm);
                    wasm.local_set(old_data_local);

                    // set new
                    wasm.local_get(local);
                    wasm.write(val_ty_wasm);

                    // drop it
                    wasm.local_get(local);
                    self.drop_value(val_ty, wasm);

                    Some(Ok(field.0.ty))
                };

                match is_ptr() {
                    Some(v) => v,
                    None => {
                        Err(Error::DerefOnNonPtr(node.range()))
                    },
                }
            }

            _ => return Err(Error::AssignIsNotLHSValue { source: node.range() }),
        }
    }


    fn call_func(
        &mut self,
        func_id: FuncId,
        args: &[(Type, SourceRange, Option<ExpressionNode>)],
        is_accessor: bool,
        source: SourceRange,
        scope: ScopeId,

        wasm: &mut WasmFunctionBuilder,
    ) -> Result<Type, ()> {
        let func = self.funcs.get(func_id);
        let Some(func) = func
        else { for _ in args { wasm.pop() } return Ok(Type::Error) };

        let func = func.clone();

        if func.args.len() != args.len() {
            wasm.error(self.error(Error::FunctionArgsMismatch {
                source, sig_len: func.args.len(), call_len: args.len() }));

            return Err(());
        }

        match func.kind {
            FunctionKind::UserDefined { inout } => {
                let inout = inout.map(|inout| {
                    let ty = self.types.get(inout);
                    let sp = wasm.alloc_stack(ty.size());
                    wasm.sptr_const(sp);
                    (ty, sp)
                });
                
                let ret = {
                    let size = func.ret.to_wasm_ty(&self.types).stack_size(); 
                    if size != 0 {
                        let sp = wasm.alloc_stack(size);
                        wasm.sptr_const(sp);
                        Some(sp)
                    } else { None }
                };

                let mut errored = false;
                for (i, (sig_arg, call_arg)) in func.args.iter().zip(args.iter()).enumerate() {
                    if !sig_arg.2.eq_sem(call_arg.0) {
                        errored = true;
                        wasm.error(self.error(Error::InvalidType {
                            source: call_arg.1, found: call_arg.0, expected: sig_arg.2 }));
                    }

                    if sig_arg.1 && call_arg.2.is_none() && !(i == 0 && is_accessor) {
                        errored = true;
                        wasm.error(self.error(Error::InOutBindingWithoutInOutValue {
                            value_range: call_arg.1 }))
                    }

                    if !sig_arg.1 && call_arg.2.is_some() && !(i == 0 && is_accessor) {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }
                }

                if errored {
                    for f in args.iter().rev() {
                        self.drop_value(f.0, wasm);
                    }

                    return Err(());
                }

                wasm.call(func.func);

                if let Some(sp) = ret {
                    wasm.sptr_const(sp);
                }

                if let Some((ty, sp)) = inout {
                    let TypeKind::Struct(sym) = ty.kind() else { unreachable!() };
                    let mut c = 0;
                    for (i, sig_arg) in func.args.iter().enumerate() {
                        if !sig_arg.1 { continue }
                        if i == 0 && is_accessor { continue }

                        let field = sym.fields[c];
                        wasm.sptr_const(sp);
                        wasm.u32_const(field.1.try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(sig_arg.2.to_wasm_ty(&self.types));

                        if let Err(e) = self.assign(
                                wasm, scope, args[i].2.unwrap(), sig_arg.2, 0) {
                            wasm.error(self.error(e));
                        }

                        c += 1;

                    }
                }

                Ok(func.ret)
            },

            FunctionKind::Extern { ty } => {
                let mut errored = false;
                for (i, (sig_arg, call_arg)) in func.args.iter().zip(args.iter()).enumerate() {
                    if !sig_arg.2.eq_sem(call_arg.0) {
                        errored = true;
                        wasm.error(self.error(Error::InvalidType {
                            source: call_arg.1, found: call_arg.0, expected: sig_arg.2 }));
                    }

                    if sig_arg.1 && call_arg.2.is_none() && !(i == 0 && is_accessor) {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }

                    if !sig_arg.1 && call_arg.2.is_some() && !(i == 0 && is_accessor) {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }
                }

                if errored {
                    for f in args.iter().rev() {
                        self.drop_value(f.0, wasm);
                    }

                    return Err(());
                }

                let ty_sym = self.types.get(ty);
                let ptr = {
                    wasm.alloc_stack(ty_sym.size())
                };

                let TypeKind::Struct(sym) = ty_sym.kind()
                else { unreachable!() };
                
                for sym_arg in sym.fields.iter().rev().skip(1) {
                    self.acquire(sym_arg.0.ty, wasm);

                    wasm.sptr_const(ptr);
                    wasm.u32_const(sym_arg.1.try_into().unwrap());
                    wasm.i32_add();

                    wasm.write(sym_arg.0.ty.to_wasm_ty(&self.types));
                }

                wasm.sptr_const(ptr);
                wasm.call(func.func);

                let mut c = 0;
                for (i, sig_arg) in func.args.iter().enumerate() {
                    let field = sym.fields[c];
                    wasm.sptr_const(ptr);
                    wasm.u32_const(field.1.try_into().unwrap());
                    wasm.i32_add();

                    wasm.read(sig_arg.2.to_wasm_ty(&self.types));

                    if !sig_arg.1 {
                        self.drop_value(sig_arg.2, wasm);
                        continue
                    }

                    if let Err(e) = self.assign(
                            wasm, scope, args[i].2.unwrap(), sig_arg.2, 0) {
                        wasm.error(self.error(e));
                    }

                    c += 1;

                }

                {
                    let r = sym.fields.last().unwrap();
                    wasm.sptr_const(ptr);
                    wasm.u32_const(r.1.try_into().unwrap());
                    wasm.i32_add();

                    wasm.read(r.0.ty.to_wasm_ty(&self.types));
                }

                return Ok(func.ret)
            },

        }
    }

    
    fn create_struct(
        &mut self,
        wasm: &mut WasmFunctionBuilder,
        scope: ScopeId,
        ty: TypeId,
        fields: &[(StringIndex, SourceRange, ExpressionNode)],
        path: StringIndex,
        source: SourceRange,
    ) -> Type {
        let tyv = self.types.get(ty);

        let TypeKind::Struct(TypeStruct { fields: sfields, .. }) = tyv.kind()
        else {
            wasm.error(self.error(Error::StructCreationOnNonStruct {
                source, typ: Type::Custom(ty) }));
            return Type::Custom(ty);
        };


        for f in fields.iter() {
            if !sfields.iter().any(|x| x.0.name == f.0) {
                wasm.error(self.error(Error::FieldDoesntExist {
                    source: f.1,
                    field: f.0,
                    typ: Type::Custom(ty),
                }));

                return Type::Custom(ty);
            }
        }
        
        let mut vec = Vec::new();
        for sf in sfields.iter() {
            if !fields.iter().any(|x| x.0 == sf.0.name) {
                vec.push(sf.0.name);
            }
        }


        if !vec.is_empty() {
            wasm.error(self.error(Error::MissingFields { source, fields: vec }));
            return Type::Custom(ty);
        }

        
        let alloc = wasm.alloc_stack(tyv.size());
        for sf in sfields.iter() {
            let val = fields.iter().find(|x| x.0 == sf.0.name).unwrap();
            let ptr = alloc.add(sf.1);

            let node = self.expr(val.2, scope, wasm, path);

            if !node.ty.eq_sem(sf.0.ty) {
                wasm.error(self.error(Error::InvalidType 
                    { source: val.1, found: node.ty, expected: sf.0.ty }));
                return Type::Custom(ty);
            }

            let wty = sf.0.ty.to_wasm_ty(&self.types);
            wasm.sptr_const(ptr);
            wasm.write(wty);
        }

        wasm.sptr_const(alloc);
        Type::Custom(ty)
    }


    fn acquire(
        &mut self,
        ty: Type,
        
        wasm: &mut WasmFunctionBuilder,
    ) {
        let Type::Custom(ty_id) = ty 
        // No acquire function
        else { return };

        let ty = self.types.get(ty_id);

        match ty.kind() {
            TypeKind::Struct(strct) => {
                let local = wasm.local(WasmType::Ptr { size: ty.size() });
                wasm.local_set(local);

                if strct.status == TypeStructStatus::Ptr {
                    // Read the counter
                    wasm.local_get(local);
                    wasm.i64_read();

                    // Increment one
                    wasm.i64_const(1);
                    wasm.i64_add();

                    // write it
                    wasm.local_get(local);
                    wasm.i64_write();

                } else {
                    for i in strct.fields {
                        wasm.local_get(local);
                        wasm.u32_const(i.1.try_into().unwrap());
                        wasm.i32_add();

                        wasm.read(i.0.ty.to_wasm_ty(&self.types));
                        self.acquire(i.0.ty, wasm);
                        wasm.pop();
                    }

                }

                wasm.local_get(local);
                return
            },


            TypeKind::Enum(e) => {
                fn do_mapping<'ast>(
                    anal: &mut Analyzer<'_, '_, '_>,
                    wasm: &mut WasmFunctionBuilder,
                    variants: &[TaggedUnionField],
                    local: LocalId,
                    tag: LocalId,

                    index: usize,
                ) {
                    let Some(mapping) = variants.get(index)
                    else {
                        wasm.block(|wasm, _| {
                            wasm.local_get(tag);
                            
                            let mut string = format_in!(anal.output, "br_table ");

                            for i in (0..variants.len()).rev() {
                                let _ = write!(string, "{} ", i);
                            }

                            let _ = write!(string, "{} ", 0);

                            wasm.raw(&string);
                        });

                        return;
                    };

                    wasm.block(|wasm, _| {
                        do_mapping(anal, wasm, variants, local, tag, index + 1);
                        
                        wasm.local_get(local);
                        anal.acquire(mapping.ty().unwrap_or(Type::Unit), wasm);
                        wasm.pop();
                    });
                }


                let TypeEnumKind::TaggedUnion(e) = e.kind()
                // nothing to drop
                else { wasm.pop(); return };

                let base_ptr = wasm.local(WasmType::Ptr { size: ty.size() });
                wasm.local_set(base_ptr);

                let tag = wasm.i32_temp();
                wasm.local_get(base_ptr);
                wasm.u32_read();

                let union = wasm.local(WasmType::Ptr { size: ty.size() - e.union_offset() as usize } );
                wasm.local_get(base_ptr);
                wasm.u32_const(e.union_offset());
                wasm.i32_add();
                wasm.local_set(union);

                do_mapping(self, wasm, e.fields(), union, tag, 0);

                wasm.local_get(base_ptr);
            },

            TypeKind::Error => {},
        }
    }


    /*
    ///
    /// If the value is a **NOT** Rc: Do nothing
    /// Else if the counter of the Rc is **NOT** 1: Clone the value
    /// Else: do nothing
    fn make_mut(
        &mut self,
        ty: Type,
        wasm: &mut WasmFunctionBuilder,
    ) -> Option<LocalId> {
        let Type::Custom(ty_id) = ty 
        // No drop function
        else { return None };

        let ty = self.types.get(ty_id);
        let TypeKind::Struct(sym) = ty.kind()
        else { return None };

        if sym.status != TypeStructStatus::Ptr { return None };

        let local = wasm.local(WasmType::I32);
        wasm.local_set(local);

        // Read the counter
        wasm.local_get(local);
        wasm.i64_read();

        // Eq 1
        wasm.i64_const(1);
        wasm.i64_ne();

        wasm.ite(&mut (),
            |_, wasm| {
                let ns = self.namespaces.get_type(Type::Custom(ty_id), &self.types);
                let ns = self.namespaces.get(ns);
                let func = ns.get_func(StringMap::NEW).unwrap();
                
                let field = sym.fields[1];

                // read the value
                wasm.local_get(local);
                wasm.u32_const(field.1 as u32);
                wasm.i32_add();

                wasm.read(field.0.ty.to_wasm_ty(&self.types));

                // drop the ptr
                wasm.local_get(local);
                self.drop_value(Type::Custom(ty_id), wasm);

                self.call_func(func,
                               &[(field.0.ty, SourceRange::ZERO, None)],
                               false, 
                               SourceRange::ZERO,
                               ScopeMap::ROOT,
                               wasm).unwrap();
                self.acquire(Type::Custom(ty_id), wasm);
                wasm.local_set(local);

                (local, ())
            },
            |_, _| { () }
        );
        Some(local)
    }
    */


    ///
    /// Calls the drop function on the value at the 
    /// top of the stack
    ///
    /// Value on top of the stack must be of type `ty`
    /// `ty` => ()
    ///
    fn drop_value(
        &mut self,
        ty: Type,
        
        wasm: &mut WasmFunctionBuilder,
    ) {
        let Type::Custom(ty_id) = ty 
        // No drop function
        else { wasm.pop(); return };

        let ty = self.types.get(ty_id);

        match ty.kind() {
            TypeKind::Struct(strct) => {
                let local = wasm.local(WasmType::Ptr { size: ty.size() });
                wasm.local_set(local);
                if strct.status == TypeStructStatus::Ptr {
                    // Read the counter
                    wasm.local_get(local);
                    wasm.i64_read();

                    // Subtract one
                    wasm.i64_const(1);
                    wasm.i64_sub();

                    let new_count = wasm.i64_temp();
                    wasm.local_tee(new_count);
                    wasm.i64_eqz();

                    // If the counter is now 0, free it
                    // Elsewise, write it 
                    wasm.ite(
                        |wasm| {
                            for i in strct.fields {
                                wasm.local_get(local);
                                wasm.u32_const(i.1.try_into().unwrap());
                                wasm.i32_add();

                                self.drop_value(i.0.ty, wasm)
                            }

                            wasm.local_get(local);
                            wasm.call_template("free");
                        },

                        |wasm| {
                            wasm.local_get(new_count);
                            wasm.local_get(local);
                            wasm.i64_write();
                        }
                    );
                    wasm.pop();

                    return
                } else {
                    for i in strct.fields {
                        wasm.local_get(local);
                        wasm.u32_const(i.1.try_into().unwrap());
                        wasm.i32_add();

                        wasm.read(i.0.ty.to_wasm_ty(&self.types));

                        self.drop_value(i.0.ty, wasm);
                    }
                }
            },


            TypeKind::Enum(e) => {
                fn do_mapping<'ast>(
                    anal: &mut Analyzer<'_, '_, '_>,
                    wasm: &mut WasmFunctionBuilder,
                    variants: &[TaggedUnionField],
                    local: LocalId,
                    tag: LocalId,

                    index: usize,
                ) {
                    let Some(mapping) = variants.get(index)
                    else {
                        wasm.block(|wasm, _| {
                            wasm.local_get(tag);
                            
                            let mut string = format_in!(anal.output, "br_table ");

                            for i in (0..variants.len()).rev() {
                                let _ = write!(string, "{} ", i);
                            }

                            let _ = write!(string, "{} ", 0);

                            wasm.raw(&string);
                        });

                        return;
                    };

                    wasm.block(|wasm, _| {
                        do_mapping(anal, wasm, variants, local, tag, index + 1);
                        
                        wasm.local_get(local);
                        anal.drop_value(mapping.ty().unwrap_or(Type::Unit), wasm)
                    });
                }


                let TypeEnumKind::TaggedUnion(e) = e.kind()
                // nothing to drop
                else { wasm.pop(); return };

                let base_ptr = wasm.local(WasmType::Ptr { size: ty.size() });
                wasm.local_set(base_ptr);

                let tag = wasm.i32_temp();
                wasm.local_get(base_ptr);
                wasm.u32_read();

                let union = wasm.local(WasmType::Ptr { size: ty.size() - e.union_offset() as usize } );
                wasm.local_get(base_ptr);
                wasm.u32_const(e.union_offset());
                wasm.i32_add();
                wasm.local_set(union);

                do_mapping(self, wasm, e.fields(), union, tag, 0);
            },

            TypeKind::Error => wasm.pop(),
        }
    }
    
    
    #[inline(never)]
    fn concat_path(&mut self, p1: StringIndex, p2: StringIndex) -> StringIndex {
        concat_path(self.output, self.string_map, p1, p2)
    }
}


fn concat_path(arena: &Arena, string_map: &mut StringMap, p1: StringIndex, p2: StringIndex) -> StringIndex {
    if p1 == StringMap::INVALID_IDENT { p2 }
    else {
        let str = format_in!(arena, "{}::{}", string_map.get(p1), string_map.get(p2));
        string_map.insert(&str)
    }
}


fn as_decl_iterator<'a>(iter: impl Iterator<Item=Node<'a>>) -> impl Iterator<Item=DeclarationNode<'a>> {
    iter
    .filter_map(|x| {
        match x {
            Node::Declaration(v) => Some(v),
            Node::Attribute(mut attr) => {
                loop {
                    match attr.node() {
                        Node::Declaration(v) => break Some(v),
                        Node::Attribute(v) => attr = v,
                        _ => break None
                    }
                }
            }
            _ => None
        }
    })
}


pub struct Builder<'a>(llvm_api::builder::Builder<'a>);

impl Builder<'_> {
    pub fn error(&mut self, error: ErrorId) {
        todo!()
    }
}

impl<'a> Deref for Builder<'a> {
    type Target = llvm_api::builder::Builder<'a>;

    fn deref(&self) -> &Self::Target {
       &self.0
    }
}

impl<'a> DerefMut for Builder<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
