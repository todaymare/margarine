pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use std::fmt::Write;

use common::{source::SourceRange, string_map::StringMap};
use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Function, FunctionKind, FuncId};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{DataType, DataTypeKind, nodes::{Node, decl::{Declaration, DeclarationNode, FunctionSignature}, stmt::{Statement, StatementNode}, expr::{Expression, ExpressionNode, BinaryOperator, UnaryOperator, MatchMapping}}};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind, FunctionDefinitionScope, VariableScope, LoopScope};
use types::{ty::Type, ty_map::TypeMap, ty_sym::{TypeEnum, TypeSymbolKind, TypeEnumKind, TypeEnumStatus, TypeStructStatus, TypeSymbol}};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType, LocalId};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::PackedOption, arena_pool::ArenaPool, hash::HashMap, traits::FromIn, string::String, format_in};

use crate::types::{ty_map::TypeId, ty_builder::{TypeBuilder, TypeBuilderData}, ty_sym::TypeStruct};

#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'str, 'ast> {
    scopes: ScopeMap<'out>,
    namespaces: NamespaceMap<'out>,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out, 'ast>,
    output: &'out Arena,
    pub string_map: &'me mut StringMap<'str>,

    pub module_builder: WasmModuleBuilder<'out, 'str>,
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
    is_mut: bool,
}

impl AnalysisResult {
    pub fn new(ty: Type, is_mut: bool) -> Self { Self { ty, is_mut } }

    pub fn error() -> Self {
        Self::new(Type::Error, true)
    }
}


impl<'out> Analyzer<'_, 'out, '_, '_> {
     pub fn convert_ty(&mut self, scope: ScopeId, dt: DataType) -> Result<Type, Error> {
        let ty = match dt.kind() {
            DataTypeKind::Int => Type::I64,
            DataTypeKind::Bool => Type::BOOL,
            DataTypeKind::Float => Type::F64,
            DataTypeKind::Unit => Type::Unit,
            DataTypeKind::Any => Type::Any,
            DataTypeKind::Never => Type::Never,
            DataTypeKind::Option(v) => {
                let inner_ty = self.convert_ty(scope, *v)?;
                let tyid = self.make_option(inner_ty);
                Type::Custom(tyid)
            },


            DataTypeKind::Result(v1, v2) => {
                let inner_ty1 = self.convert_ty(scope, *v1)?;
                let inner_ty2 = self.convert_ty(scope, *v2)?;
                Type::Custom(self.make_result(inner_ty1, inner_ty2))
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
                let Some(ns) = self.scopes.get(scope).get_ns(ns, &self.scopes, &mut self.namespaces)
                else {
                    return Err(Error::NamespaceNotFound { source: dt.range(), namespace: ns });
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), None.into());
                let scope = self.scopes.push(scope);
                self.convert_ty(scope, *dt)?
            }


            | DataTypeKind::RcConst(ty)
            | DataTypeKind::RcMut(ty) => {
                let ty = self.convert_ty(scope, *ty)?;
                if let Some(ty) = self.rc_map.get(&ty) { return Ok(Type::Custom(*ty)) }

                let tyid = {
                    let temp = ArenaPool::tls_get_temp();
                    let name = {
                        let mut str = sti::string::String::new_in(&*temp);
                        str.push_char('*');
                        str.push(ty.display(self.string_map, &self.types));

                        self.string_map.insert(str.as_str())
                    };

                    let mut tyb = TypeBuilder::new(&temp);

                    let tyid = self.types.pending();
                    tyb.add_ty(tyid, name, SourceRange::new(0, 0));
                    tyb.set_struct_fields(tyid, 
                        [
                            (StringMap::INT, Type::I64),
                            (StringMap::VALUE, ty),
                        ].iter().copied(),
                        if matches!(dt.kind(), DataTypeKind::RcConst(_)) { TypeStructStatus::Rc }
                        else { TypeStructStatus::RcMut },
                    );

                    let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
                    tyb.finalise(data, &mut self.errors);

                    tyid
                };

                self.rc_map.insert(ty, tyid);
                Type::Custom(tyid)
            },
        };

        Ok(ty)
    }

    
    pub fn make_tuple(&mut self, vec: Vec<Type, &Arena>, source: SourceRange) -> TypeId {
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

        let mut tyb = TypeBuilder::new(&temp);

        let tyid = self.types.pending();
        tyb.add_ty(tyid, name, source);
        tyb.set_struct_fields(tyid, vec.iter().enumerate().map(|(i, x)| {
            let mut str = sti::string::String::new_in(&*temp);
            let _ = write!(str, "{}", i);
            let id = self.string_map.insert(&str);
            (id, *x)
        }), TypeStructStatus::Tuple);

        let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
        tyb.finalise(data, &mut self.errors);

        self.tuple_map.insert(vec.move_into(self.output).leak(), tyid);

        tyid
    }


    pub fn make_result(&mut self, v1: Type, v2: Type) -> TypeId {
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

            let mut tyb = TypeBuilder::new(&temp);

            let tyid = self.types.pending();
            tyb.add_ty(tyid, name, SourceRange::new(0, 0));
            tyb.set_enum_fields(tyid, 
                [
                (self.string_map.insert("ok"), Some(v1)),
                (self.string_map.insert("err"),Some(v2)),
                ].iter().copied(),
                TypeEnumStatus::Result,
            );

            let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
            tyb.finalise(data, &mut self.errors);

            tyid
        };

        self.results_map.insert((v1, v2), tyid);

        tyid
    }


    fn make_option(&mut self, ty: Type) -> TypeId {
        if let Some(v) = self.options_map.get(&ty) { return *v; }

        let tyid = {
            let temp = ArenaPool::tls_get_temp();
            let name = {
                let mut str = sti::string::String::new_in(&*temp);
                str.push(ty.display(self.string_map, &self.types));
                str.push_char('?');

                self.string_map.insert(str.as_str())
            };

            let mut tyb = TypeBuilder::new(&temp);

            let tyid = self.types.pending();
            tyb.add_ty(tyid, name, SourceRange::new(0, 0));
            tyb.set_enum_fields(tyid, 
                [
                (self.string_map.insert("some"), Some(ty)),
                (self.string_map.insert("none"), None),
                ].iter().copied(),
                TypeEnumStatus::Option,
            );

            let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
            tyb.finalise(data, &mut self.errors);

            tyid
        };

        self.options_map.insert(ty, tyid);
        tyid
    }
     

    pub fn error(&mut self, err: Error) -> ErrorId {
        ErrorId::Sema(self.errors.push(err))
    }
}


impl<'me, 'out, 'str, 'ast> Analyzer<'me, 'out, 'str, 'ast> {
    pub fn run(
        output: &'out Arena,
        string_map: &'me mut StringMap<'str>,
        nodes: &[Node<'ast>],
    ) -> Self {
        let mut slf = Self {
            scopes: ScopeMap::new(),
            namespaces: NamespaceMap::new(output),
            types: TypeMap::new(),
            funcs: FunctionMap::new(),
            module_builder: WasmModuleBuilder::new(output),
            errors: KVec::new(),
            output,
            string_map,
            options_map: HashMap::new(),
            results_map: HashMap::new(),
            tuple_map: HashMap::new(),
            rc_map: HashMap::new(),
            startup_functions: Vec::new(),
        };

        slf.module_builder.memory(64 * 1024);
        slf.module_builder.stack_size(32 * 1024);

        {
            let pool = ArenaPool::tls_get_temp();
            let mut type_builder = TypeBuilder::new(&pool);

            {
                let id = slf.types.pending();
                assert_eq!(TypeId::BOOL, id);

                type_builder.add_ty(TypeId::BOOL, StringMap::BOOL, SourceRange::new(0, 0));
                type_builder.set_enum_fields(
                    TypeId::BOOL,
                    [(StringMap::FALSE, None), (StringMap::TRUE, None)].into_iter(),
                    TypeEnumStatus::User,
                );
            }
            {
                let id = slf.types.pending();
                assert_eq!(TypeId::STR, id);

                type_builder.add_ty(TypeId::STR, StringMap::STR, SourceRange::new(0, 0));
                type_builder.set_struct_fields(
                    TypeId::STR,
                    [
                        (slf.string_map.insert("len"), Type::I64),
                        (slf.string_map.insert("ptr"), Type::I64),
                    ].into_iter(),
                    TypeStructStatus::User
                );
            }

            let data = TypeBuilderData::new(
                &mut slf.types,
                &mut slf.namespaces,
                &mut slf.funcs,
                &mut slf.module_builder,
            );

            type_builder.finalise(data, &mut slf.errors);
        }

        let mut func = WasmFunctionBuilder::new(output, slf.module_builder.function_id());
        let scope = Scope::new(ScopeKind::Root, PackedOption::NONE);
        let scope = slf.scopes.push(scope);

        func.export(StringMap::INIT_FUNC);

        slf.block(&mut func, scope, nodes);
        func.pop();

        for s in slf.startup_functions.iter() {
            let f = slf.funcs.get(*s);
            assert!(f.args.is_empty(), "resources are not ready yet");
            let ret_ty = f.ret.to_wasm_ty(&slf.types).stack_size();
            if ret_ty != 0 {
                let alloc = func.alloc_stack(ret_ty);
                func.sptr_const(alloc);
            }
            func.call(f.wasm_id);
        }

        slf.module_builder.register(func);

        slf
    }


    pub fn block(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        scope: ScopeId,
        nodes: &[Node<'ast>],
    ) -> (AnalysisResult, ScopeId) {
        let pool = ArenaPool::tls_get_rec();
        let mut ty_builder = TypeBuilder::new(&*pool); 
        let (scope, ns_id) = {
            let namespace = Namespace::new(self.output);
            let namespace = self.namespaces.put(namespace);

            self.collect_type_names(
                as_decl_iterator(nodes.iter().copied()),
                builder, &mut ty_builder, namespace
            );

            (Scope::new(ScopeKind::ImplicitNamespace(namespace), scope.some()), namespace)
        };
        
        let mut scope = self.scopes.push(scope);

        self.collect_impls(builder, &mut ty_builder, as_decl_iterator(nodes.iter().copied()), scope, ns_id);
        self.collect_uses(as_decl_iterator(nodes.iter().copied()), builder, scope, ns_id);
        self.resolve_names(as_decl_iterator(nodes.iter().copied()), builder, &mut ty_builder, scope, ns_id);
        
        {
            let err_len = self.errors.len();

            let data = TypeBuilderData::new(
                &mut self.types, &mut self.namespaces,
                &mut self.funcs, &mut self.module_builder
            );

            ty_builder.finalise(data, &mut self.errors);

            for i in err_len..self.errors.len() {
                builder.error(ErrorId::Sema(SemaError::new((err_len + i).try_into().unwrap()).unwrap()))
            }
        }
        
        self.resolve_functions(as_decl_iterator(nodes.iter().copied()), builder, scope, ns_id);

        let mut ty = Type::Unit;
        for (id, n) in nodes.iter().enumerate() {
            ty = self.node(&mut scope, builder, n).ty;

            if id + 1 != nodes.len() {
                builder.pop();
            } 

        }

        if nodes.is_empty() { builder.unit(); }

        (AnalysisResult { ty, is_mut: true }, scope)
    }
}


impl<'ast> Analyzer<'_, '_, '_, 'ast> {
    pub fn collect_type_names<'a>(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'a>>,
        
        builder: &mut WasmFunctionBuilder,
        type_builder: &mut TypeBuilder,
        namespace: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                | Declaration::Enum { name, header, .. }
                | Declaration::Struct { name, header, .. } => {
                    let namespace = self.namespaces.get_mut(namespace);
                    if namespace.get_type(name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: header, name }));

                        continue
                    }

                    let ty = self.types.pending();
                    namespace.add_type(name, ty);
                    type_builder.add_ty(ty, name, header);
                },


                Declaration::Function { sig, .. } => {
                    let namespace = self.namespaces.get_mut(namespace);
                    if namespace.get_func(sig.name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: sig.source, name: sig.name }));

                        continue
                    }

                    namespace.add_func(sig.name, self.funcs.pending())
                },
                Declaration::Impl { .. } => (),

                Declaration::Using { .. } => (),

                Declaration::Module { name, .. } => {
                    let ns = self.namespaces.get_mut(namespace);
                    if ns.get_mod(name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: decl.range(), name }));

                        continue
                    }

                    let ns = Namespace::new(self.output);
                    let ns = self.namespaces.put(ns);

                    let namespace = self.namespaces.get_mut(namespace);
                    namespace.add_mod(name, ns);
                },

                Declaration::Extern { functions, .. } => {
                    for f in functions.iter() {
                        let namespace = self.namespaces.get_mut(namespace);
                        if namespace.get_func(f.name()).is_some() {
                            builder.error(self.error(Error::NameIsAlreadyDefined { 
                               source: f.range(), name: f.name() }));

                            continue
                        }

                        namespace.add_func(f.name(), self.funcs.pending())
                    }

                },
                Declaration::Trait { name, sigs } => todo!(),
            }
        }
    }


    pub fn collect_impls<'a>(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        type_builder: &mut TypeBuilder,
        decls: impl Iterator<Item=DeclarationNode<'a>>,
        
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Impl { data_type, body } => {
                    let ty = match self.convert_ty(scope, data_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            return;
                        },
                    };
                    
                    let ns_id = self.namespaces.get_type(ty);

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns_id), scope.some());
                    let scope = self.scopes.push(scope);

                    self.collect_type_names(body.iter().copied(), builder, type_builder, ns_id);
                    self.collect_impls(builder, type_builder, body.iter().copied(), scope, ns_id);
                }


                Declaration::Module { name, body } => {
                    let ns_id = self.namespaces.get(ns_id);
                    let ns = ns_id.get_mod(name).unwrap();

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);

                    self.collect_type_names(body.iter().copied(), builder, type_builder, ns);
                    self.collect_impls(builder, type_builder, body.iter().copied(), scope, ns);
                }


                Declaration::Using { .. } => (), 

                _ => continue
            }
        }

    }


    pub fn collect_uses<'a>(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'a>>,

        _builder: &mut WasmFunctionBuilder,
        _scope: ScopeId,
        _ns_id: NamespaceId,
    ) {
        for decl in decls {
            let Declaration::Using { item: _ } = decl.kind()
            else { continue };

            todo!()
        }
    }




    pub fn resolve_names<'a>(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'a>>,

        builder: &mut WasmFunctionBuilder,
        type_builder: &mut TypeBuilder,
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Struct { name, fields, .. } => {
                    let ty = self.namespaces.get(ns_id).get_type(name).unwrap();
                    let fields = fields.iter()
                        .filter_map(|(name, ty, _)| {
                            let ty = self.convert_ty(scope, *ty);
                            match ty {
                                Ok(v) => return Some((*name, v)),
                                Err(e) => self.error(e),
                            };

                            None
                        });

                    type_builder.set_struct_fields(ty, fields, TypeStructStatus::User);
                },


                Declaration::Enum { name, mappings, .. } => {
                    let ty = self.namespaces.get(ns_id).get_type(name).unwrap();
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
                        let Ok(ty) = self.convert_ty(scope, data_type)
                        else { continue };

                        self.namespaces.get_type(ty)
                    };

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);
                    
                    self.resolve_names(body.iter().copied(), builder, type_builder, scope, ns);
                },

                Declaration::Module { name, body } => {
                    let ns = self.namespaces.get(ns_id);
                    let ns = ns.get_mod(name).unwrap();

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);

                    self.resolve_names(body.iter().copied(), builder, type_builder, scope, ns)
                },

                Declaration::Using { .. } => (),

                _ => continue,
           }
        }
    }


    pub fn resolve_functions(
        &mut self,
        decls: impl Iterator<Item=DeclarationNode<'ast>>,

        builder: &mut WasmFunctionBuilder,
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for decl in decls {
            match decl.kind() {
                Declaration::Function { sig, body } => {
                    let mut nscope = scope;
                    if !sig.generics.is_empty() {
                        let mut ns = Namespace::new(self.output);
                        for g in sig.generics {
                            if ns.get_type(g.name()).is_some() {
                                builder.error(self.error(Error::GenericAlreadyDefined 
                                                         { source: g.range() }));
                                continue
                            }

                            let ty = TypeSymbol::new(g.name(), 0, 0, TypeSymbolKind::Generic);
                            let ty_id = self.types.pending();
                            self.types.put(ty_id, ty);

                            ns.add_type(g.name(), ty_id);
                        }

                        let ns = self.namespaces.put(ns);
                        let s = Scope::new(ScopeKind::ImplicitNamespace(ns), nscope.some());
                        nscope = self.scopes.push(s);
                    }

                    let args = {
                        let mut args = Vec::with_cap_in(self.output, sig.arguments.len());

                        for arg in sig.arguments.iter() {
                            let ty = self.convert_ty(nscope, arg.data_type());
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

                    let ret = match self.convert_ty(nscope, sig.return_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            Type::Error
                        },
                    };

                    let inout_ty_id = if args.iter().any(|a| a.1) {
                        let temp = ArenaPool::tls_get_temp();
                        let tuple = Vec::from_in(&*temp,
                                                   args.iter().map(|a| a.2));
                        Some(self.make_tuple(tuple, decl.range()))
                    } else { None };

                    let ns = self.namespaces.get_mut(ns_id);
                    let func_id = ns.get_func(sig.name).unwrap();
                    let func = if sig.generics.is_empty() { FunctionKind::UserDefined {
                        inout: inout_ty_id,  }
                    } else {
                        FunctionKind::Template {
                            body,
                            scope,
                        }
                    };
                    let func = Function::new(sig.name, args.leak(), ret, self.module_builder.function_id(), func);
                    self.funcs.put(func_id, func);
                },


                Declaration::Extern { file, functions  } => {
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
                        
                        let ns = self.namespaces.get_mut(ns_id);
                        let func_id = ns.get_func(f.name()).unwrap();
                        let func = FunctionKind::Extern { ty };
                        let func = Function::new(f.name(), args.leak(), ret, wfid, func);
                        self.funcs.put(func_id, func);
                    }
                },


                Declaration::Impl { data_type, body } => {
                    let ty = match self.convert_ty(scope, data_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            return;
                        },
                    };
                    
                    let ns_id = self.namespaces.get_type(ty);

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns_id), scope.some());
                    let scope = self.scopes.push(scope);


                    self.resolve_functions(body.iter().copied(), builder, scope, ns_id);
                }


                Declaration::Module { name, body } => {
                    let ns_id = self.namespaces.get_mut(ns_id);
                    let ns = ns_id.get_mod(name).unwrap();

                    let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                    let scope = self.scopes.push(scope);

                    self.resolve_functions(body.iter().copied(), builder, scope, ns)
                }

                _ => continue,
            }
        }

    }


    fn node(
        &mut self,
        scope: &mut ScopeId,
        wasm: &mut WasmFunctionBuilder,

        node: &Node<'ast>,
    ) -> AnalysisResult {
        match node {
            Node::Declaration(decl) => {
                self.decl(*decl, *scope);
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },

            Node::Statement(stmt) => {
                if self.stmt(*stmt, scope, wasm).is_err() {
                    wasm.unit();
                    return AnalysisResult::error()
                }
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)

            },

            Node::Expression(expr) => self.expr(*expr, *scope, wasm),

            Node::Error(err) => {
                wasm.error(err.id());
                wasm.unit();
                AnalysisResult::error()
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
                                wasm.error(self.error(Error::InvalidValueForAttr {
                                    attr: (attr.range(), attr.attr().name()),
                                    value: node.range(),
                                    expected: "a system function",
                                }));
                                wasm.unit();
                                return AnalysisResult::error();
                            }
                        };

                        self.decl(decl, *scope);

                        let Declaration::Function { sig, .. } = decl.kind()
                        else { unreachable!() };

                        let func = self.scopes.get(*scope).get_func(sig.name, &self.scopes, &self.namespaces).unwrap();

                        self.startup_functions.push(func);

                        wasm.unit();
                        AnalysisResult::new(Type::Unit, true)
                    }
                    _ => {
                        wasm.error(self.error(Error::UnknownAttr(attr.range(), attr.attr().name())));
                        wasm.unit();
                        AnalysisResult::error()
                    }
                }
            },
        }
    }


    fn decl(
        &mut self,
        decl: DeclarationNode<'ast>,
        scope: ScopeId,
    ) {
        match decl.kind() {
            Declaration::Struct { .. } => (),
            Declaration::Enum { .. } => (),


            Declaration::Function { sig, body } => {
                let func = self.scopes.get(scope).get_func(sig.name, &self.scopes, &self.namespaces).unwrap();
                let func = self.funcs.get(func);
                match func.kind {
                    FunctionKind::UserDefined { inout } => {
                        let mut wasm = WasmFunctionBuilder::new(self.output, func.wasm_id);

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
                            let TypeSymbolKind::Struct(val) = x else { unreachable!() };
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

                        let inout_param = inout.map(|inout| wasm.param(WasmType::Ptr { size: self.types.get(inout).size() }));
                        for i in ids.iter() {
                            wasm.write_to(&mut string, |wasm| {
                                let f = fields.unwrap().fields[counter];
                                let inout = inout_param.unwrap();
                                wasm.local_get(*i);

                                wasm.local_get(inout);
                                wasm.u32_const(f.offset.try_into().unwrap());
                                wasm.i32_add();

                                wasm.write(f.ty.to_wasm_ty(&self.types));

                                counter += 1;
                            });
                        }

                        let wasm_ret = func.ret.to_wasm_ty(&self.types);
                        if wasm_ret.stack_size() != 0 {
                            wasm.param(wasm_ret);
                        }

                        wasm.set_finaliser(string.clone_in(self.module_builder.arena));

                        let (anal, _) = self.block(&mut wasm, scope, &body);
                        if !anal.ty.eq_sem(func.ret) {
                            wasm.error(self.error(Error::FunctionBodyAndReturnMismatch {
                                header: sig.source, item: body.last().map(|x| x.range()).unwrap_or(body.range()),
                                return_type: func.ret, body_type: anal.ty }));
                        }

                        self.module_builder.register(wasm);
                    },


                    FunctionKind::Template { .. } => {
                        let pool = &*ArenaPool::tls_get_rec();
                        let mut wasm = WasmFunctionBuilder::new(&*pool, func.wasm_id);
                        let scope = Scope::new(
                            ScopeKind::FunctionDefinition(
                                FunctionDefinitionScope::new(func.ret, sig.return_type.range())
                            ),
                            scope.some()
                        );

                        let mut scope = self.scopes.push(scope);

                        for a in func.args.iter() {
                            let wasm_ty = a.2.to_wasm_ty(&self.types);

                            let local_id = wasm.param(wasm_ty);
                            let t = Scope::new(
                                ScopeKind::Variable(VariableScope::new(a.0, a.1, a.2, local_id)),
                                scope.some(),
                            );

                            scope = self.scopes.push(t);
                        }

                        let (anal, _) = self.block(&mut wasm, scope, &body);
                        if !anal.ty.eq_sem(func.ret) {
                            wasm.error(self.error(Error::FunctionBodyAndReturnMismatch {
                                header: sig.source, item: body.last().map(|x| x.range()).unwrap_or(body.range()),
                                return_type: func.ret, body_type: anal.ty }));
                        }
                    },


                    FunctionKind::Extern { .. } => unreachable!(),
                };
            },


            Declaration::Impl { data_type, body } => {
                let Ok(ns) = self.convert_ty(scope, data_type)
                else { return };

                let ns = self.namespaces.get_type(ns);
                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let scope = self.scopes.push(scope);

                for decl in body.iter() {
                    self.decl(*decl, scope);
                }
            },

            Declaration::Using { .. } => (),
            Declaration::Module { name, body } => {
                let ns = self.scopes.get(scope);
                let ns = ns.get_mod(name, &self.scopes, &self.namespaces).unwrap();
                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let scope = self.scopes.push(scope);

                for decl in body.iter() {
                    self.decl(*decl, scope);
                }
            },

            Declaration::Extern { .. } => (),
            Declaration::Trait { name, sigs } => todo!(),
        }
    }


    fn stmt(
        &mut self,
        stmt: StatementNode<'ast>,

        scope: &mut ScopeId,
        wasm: &mut WasmFunctionBuilder,
    ) -> Result<(), ()> {
        let source = stmt.range();
        match stmt.kind() {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let mut func = || -> Result<(), ()> {
                    let rhs_anal = self.expr(rhs, *scope, wasm);
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(());
                    }

                    if let Some(hint) = hint {
                        let hint = match self.convert_ty(*scope, hint) {
                            Ok(v) => v,
                            Err(e) => {
                                wasm.error(self.error(e));
                                return Err(());
                            }
                        };

                        if !hint.eq_sem(rhs_anal.ty) {
                            wasm.error(self.error(Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source }));
                            return Err(())
                        }
                    }

                    let local = wasm.local(rhs_anal.ty.to_wasm_ty(&self.types));
                    wasm.local_set(local);

                    let variable_scope = VariableScope::new(name, is_mut, rhs_anal.ty, local);
                    *scope = self.scopes.push(
                        Scope::new(ScopeKind::Variable(variable_scope), scope.some()));

                    Ok(())
                };

                if func().is_err() {
                    let dummy = VariableScope::new(name, is_mut, Type::Error, wasm.local(WasmType::I64));
                    *scope = self.scopes.push(Scope::new(ScopeKind::Variable(dummy), scope.some()));
                    return Err(());
                }
                
            },


            Statement::VariableTuple { names, hint, rhs } => {
                let mut func = || -> Result<(), ()> {
                    let rhs_anal = self.expr(rhs, *scope, wasm);
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(());
                    }

                    if let Some(hint) = hint {
                        let hint = match self.convert_ty(*scope, hint) {
                            Ok(v) => v,
                            Err(e) => {
                                wasm.error(self.error(e));
                                return Err(());
                            }
                        };

                        if !hint.eq_sem(rhs_anal.ty) {
                            wasm.error(self.error(Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source }));
                            return Err(())
                        }
                    }

                    let rty = match rhs_anal.ty {
                        Type::Custom(v) => self.types.get(v),
                        _ => {
                            wasm.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                            return Err(());
                        }
                    };

                    let TypeSymbolKind::Struct(sym) = rty.kind()
                    else {
                        wasm.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                        return Err(());
                    };

                    if sym.status != TypeStructStatus::Tuple {
                        wasm.error(self.error(Error::VariableValueNotTuple(rhs.range())));
                        return Err(());
                    }

                    let ptr = wasm.local(rhs_anal.ty.to_wasm_ty(&self.types));
                    wasm.local_set(ptr);

                    for (binding, sym) in names.iter().zip(sym.fields.iter()) {
                        wasm.local_get(ptr);
                        wasm.u32_const(sym.offset.try_into().unwrap());
                        wasm.i32_add();


                        let sym_ty = sym.ty.to_wasm_ty(&self.types);
                        wasm.read(sym_ty);

                        let local = wasm.local(sym_ty);
                        wasm.local_set(local);

                        let variable_scope = VariableScope::new(binding.0, binding.1, sym.ty, local);
                        *scope = self.scopes.push(
                            Scope::new(ScopeKind::Variable(variable_scope), scope.some()));
                    }

                    Ok(())
                };

                if func().is_err() {
                    for binding in names.iter() {
                        let dummy = VariableScope::new(binding.0, binding.1, Type::Error, wasm.local(WasmType::I64));
                        *scope = self.scopes.push(Scope::new(ScopeKind::Variable(dummy), scope.some()));
                    }
                    return Err(());
                }

            },


            Statement::UpdateValue { lhs, rhs } => {
                let rhs_anal = self.expr(rhs, *scope, wasm);
                if let Err(e) = self.assign(wasm, *scope, lhs, rhs_anal.ty, 0) {
                    wasm.error(self.error(e));
                    return Err(());
                }
            },


            Statement::ForLoop { binding, expr, body } => {
                let expr_anal = self.expr(expr.1, *scope, wasm);

                if !expr_anal.is_mut && expr.0 {
                    wasm.error(self.error(Error::InOutValueIsntMut(expr.1.range())));
                    return Err(());
                }

                if binding.is_inout() && !expr.0 {
                    wasm.error(self.error(Error::InOutBindingWithoutInOutValue {
                        value_range: expr.1.range(), binding_range: binding.range() }));
                    return Err(());
                }

                if !binding.is_inout() && expr.0 {
                    wasm.error(self.error(Error::InOutValueWithoutInOutBinding { value_range: expr.1.range() }));
                    return Err(());
                }

                let ty_ns = self.namespaces.get_type(expr_anal.ty);
                let ty_ns = self.namespaces.get(ty_ns);
                
                let Some(func) = ty_ns.get_func(StringMap::ITER_NEXT_FUNC)
                else {
                    wasm.error(self.error(Error::ValueIsntAnIterator {
                        ty: expr_anal.ty, range: expr.1.range() }));
                    return Err(())
                };

                let func = self.funcs.get(func);

                todo!()
            },
        };
        Ok(())
   }


    fn expr(
        &mut self,
        expr: ExpressionNode<'ast>,

        scope: ScopeId,
        wasm: &mut WasmFunctionBuilder,
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
                        let TypeSymbolKind::Struct(strct) = ty.kind()
                        else { unreachable!() };

                        let alloc = wasm.alloc_stack(ty.size());

                        // len
                        {

                            let ptr = alloc.add(strct.fields[0].offset);
                            wasm.i64_const(str.len() as i64);
                            wasm.sptr_const(ptr);
                            wasm.i64_write();
                        }

                        // ptr
                        {
                            let ptr = alloc.add(strct.fields[1].offset);
                            wasm.string_const(string_ptr);
                            wasm.sptr_const(ptr);
                            wasm.i64_write();
                        }

                        wasm.sptr_const(alloc);
                        
                        AnalysisResult::new(Type::STR, true)
                    },

                    lexer::Literal::Bool(v) => {
                        let ty = Type::BOOL;
                        let name = if v { StringMap::TRUE } else { StringMap::FALSE };

                        let func = self.namespaces.get_type(ty);
                        let func = self.namespaces.get(func).get_func(name).unwrap();
                        let func = self.funcs.get(func);
                        
                        wasm.call(func.wasm_id);
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
                let lhs_anal = self.expr(*lhs, scope, wasm);
                let rhs_anal = self.expr(*rhs, scope, wasm);

                let mut type_check = || {
                    if lhs_anal.ty.eq_lit(Type::Error)
                        || rhs_anal.ty.eq_lit(Type::Error)
                        {
                            return Err(())
                    }


                    if !lhs_anal.ty.eq_sem(rhs_anal.ty) {
                        wasm.error(self.error(Error::InvalidType {
                            source, found: rhs_anal.ty, expected: lhs_anal.ty }));
                            return Err(());
                    }

                    if operator.is_arith() 
                        && !(lhs_anal.ty.is_number() && rhs_anal.ty.is_number()) {
                        wasm.error(self.error(Error::InvalidBinaryOp {
                            operator, lhs: lhs_anal.ty,
                            rhs: rhs_anal.ty, source 
                        }));

                        return Err(())
                    }

                    Ok(())
                };

                if type_check().is_err() {
                    wasm.pop();
                    wasm.pop();
                    wasm.unit();
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
                    (BinaryOperator::Add, Type::I64) => wfunc!(i64_add, Type::I64),
                    (BinaryOperator::Add, Type::F64) => wfunc!(f64_add, Type::F64),

                    (BinaryOperator::Sub, Type::I64) => wfunc!(i64_sub, Type::I64),
                    (BinaryOperator::Sub, Type::F64) => wfunc!(f64_sub, Type::I64),

                    (BinaryOperator::Mul, Type::I64) => wfunc!(i64_mul, Type::I64),
                    (BinaryOperator::Mul, Type::F64) => wfunc!(f64_mul, Type::I64),

                    (BinaryOperator::Div, Type::I64) => wfunc!(i64_div, Type::I64),

                    (BinaryOperator::Rem, Type::I64) => wfunc!(i64_rem, Type::I64),
                    (BinaryOperator::Rem, Type::F64) => wfunc!(f64_rem, Type::I64),

                    (BinaryOperator::BitshiftLeft, Type::I64) => wfunc!(i64_bw_left_shift, Type::I64),

                    (BinaryOperator::BitshiftRight, Type::I64) => wfunc!(i64_bw_right_shift, Type::I64),

                    (BinaryOperator::BitwiseAnd, Type::I64) => wfunc!(i64_bw_and, Type::I64),

                    (BinaryOperator::BitwiseOr, Type::I64) => wfunc!(i64_bw_or, Type::I64),

                    (BinaryOperator::BitwiseXor, Type::I64) => wfunc!(i64_bw_xor, Type::I64),

                    (BinaryOperator::Eq, _) => {
                        wasm.eq(lhs_anal.ty.to_wasm_ty(&self.types));
                        Type::BOOL
                    }

                    (BinaryOperator::Ne, _) => {
                        wasm.ne(lhs_anal.ty.to_wasm_ty(&self.types));
                        Type::BOOL
                    }

                    (BinaryOperator::Gt, Type::I64)   => wfunc!(i64_gt, Type::BOOL),
                    (BinaryOperator::Gt, Type::F64) => wfunc!(f64_gt, Type::BOOL),
                    (BinaryOperator::Ge, Type::I64)   => wfunc!(i64_ge, Type::BOOL),
                    (BinaryOperator::Ge, Type::F64) => wfunc!(f64_ge, Type::BOOL),
                    (BinaryOperator::Lt, Type::I64)   => wfunc!(i64_lt, Type::BOOL),
                    (BinaryOperator::Lt, Type::F64) => wfunc!(f64_lt, Type::BOOL),
                    (BinaryOperator::Le, Type::I64)   => wfunc!(i64_le, Type::BOOL),
                    (BinaryOperator::Le, Type::F64) => wfunc!(f64_le, Type::BOOL),

                    _ => unreachable!()
                };

                AnalysisResult::new(ty, true)
            },


            Expression::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(*rhs, scope, wasm);
                
                let mut type_check = || {
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(())
                    }

                    if operator == UnaryOperator::Not
                        && !rhs_anal.ty.eq_sem(Type::BOOL) {

                        wasm.error(self.error(Error::InvalidUnaryOp {
                            operator, rhs: rhs_anal.ty, source
                        }));

                        return Err(())

                    } else if operator == UnaryOperator::Neg
                        && !rhs_anal.ty.is_number() {

                        wasm.error(self.error(Error::InvalidUnaryOp {
                            operator, rhs: rhs_anal.ty, source
                        }));

                        return Err(())
                    }

                    Ok(())
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

                    (UnaryOperator::Neg, Type::F64) => wasm.f64_neg(),

                    _ => unreachable!()
                }

                AnalysisResult::new(rhs_anal.ty, true)
            },


            Expression::If { condition, body, else_block } => {
                let cond = self.expr(*condition, scope, wasm);

                if !cond.ty.eq_sem(Type::BOOL) {
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
                let TypeSymbolKind::Enum(e) = ty.kind() else { panic!() };
                e.get_tag(wasm);

                let mut slf = self;
                let (local, l, r) = wasm.ite(
                    &mut (&mut slf, scope),
                    |(slf, scope), wasm| {
                        let body = slf.expr(*body, *scope, wasm);
                        let wty = body.ty.to_wasm_ty(&slf.types);
                        let local = wasm.local(wty);
                        (local, Some((body, wasm.offset())))
                    },
                    |((slf, scope), _), wasm| {
                        if let Some(else_block) = else_block {
                            return Some((slf.expr(*else_block, *scope, wasm), wasm.offset()))
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
                let anal = self.expr(*value, scope, wasm);
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
                let TypeSymbolKind::Enum(sym) = ty.kind()
                else { 
                    wasm.error(self.error(Error::MatchValueIsntEnum {
                        source: value.range(), typ: anal.ty }));
                    return AnalysisResult::error();
                };

                wasm.local_get(sym_local);

                let tag = wasm.local(WasmType::I32);
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
                        anal: &mut Analyzer<'_, '_, '_, 'ast>,
                        wasm: &mut WasmFunctionBuilder,
                        mappings: &[MatchMapping<'ast>],
                        enum_sym: TypeEnum,
                        sym_local: LocalId,
                        tag: LocalId,
                        taken_as_inout: bool,
                        value_range: SourceRange,
                        scope: ScopeId,

                        index: usize,
                    ) -> Option<(Type, LocalId, SourceRange)> {
                        let Some(mapping) = mappings.get(index)
                        else {
                            wasm.block(|wasm, _| {
                                wasm.local_get(tag);
                                
                                let mut string = format_in!(anal.output, "br_table {} ", mappings.len());

                                for i in (0..mappings.len()).rev() {
                                    let _ = write!(string, "{} ", i);
                                }

                                wasm.raw(&string);
                            });

                            return None;
                        };

                        let mut final_result = None;
                        wasm.block(|wasm, _| {
                            let result = do_mapping(anal, wasm, mappings, enum_sym, sym_local, tag,
                                                    taken_as_inout, value_range, scope, index + 1);

                            if mapping.is_inout() && !taken_as_inout {
                                wasm.error(anal.error(Error::InOutBindingWithoutInOutValue {
                                    value_range, binding_range: mapping.binding_range() }))
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

                            let analysis = anal.expr(mapping.node(), scope, wasm);

                            final_result = Some(if let Some((ty, local, src)) = result {
                                if analysis.ty.eq_sem(ty) {
                                    wasm.local_set(local);
                                } else {
                                    wasm.error(anal.error(Error::MatchBranchesDifferInReturnType {
                                        initial_source: src, initial_typ: ty,
                                        branch_source: mapping.range(), branch_typ: analysis.ty }));
                                    wasm.pop();
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
                               value.range(), scope, 0);

                    if let Some(result) = result {
                        wasm.local_get(result.1);
                        AnalysisResult::new(result.0, true)
                    } else {
                        wasm.unit();
                        AnalysisResult::new(Type::Unit, true)
                    }
                }
            },

            Expression::Block { block } => self.block(wasm, scope, &*block).0,

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


                let strct = self.types.get(tyid);
                let TypeSymbolKind::Struct(TypeStruct { fields: sfields, .. }) = strct.kind() 
                else {
                    wasm.error(self.error(Error::StructCreationOnNonStruct {
                        source, typ: ty }));
                    return AnalysisResult::error();
                };


                for f in fields.iter() {
                    if !sfields.iter().any(|x| x.name == f.0) {
                        wasm.error(self.error(Error::FieldDoesntExist {
                            source: f.1,
                            field: f.0,
                            typ: ty,
                        }));

                        return AnalysisResult::error();
                    }
                }
                
                let mut vec = Vec::new();
                for sf in sfields.iter() {
                    if !fields.iter().any(|x| x.0 == sf.name) {
                        vec.push(sf.name);
                    }
                }


                if !vec.is_empty() {
                    wasm.error(self.error(Error::MissingFields { source, fields: vec }));
                    return AnalysisResult::error();
                }

                
                let alloc = wasm.alloc_stack(strct.size());
                for sf in sfields.iter() {
                    let val = fields.iter().find(|x| x.0 == sf.name).unwrap();
                    let ptr = alloc.add(sf.offset);

                    let node = self.expr(val.2, scope, wasm);
                    if !node.ty.eq_sem(sf.ty) {
                        wasm.error(self.error(Error::InvalidType 
                            { source: val.1, found: node.ty, expected: sf.ty }));
                        return AnalysisResult::error();
                    }

                    let wty = sf.ty.to_wasm_ty(&self.types);
                    wasm.sptr_const(ptr);
                    wasm.write(wty);
                }

                wasm.sptr_const(alloc);
                AnalysisResult::new(ty, true)
            },
            
            
            Expression::AccessField { val, field_name } => {
                let value = self.expr(*val, scope, wasm);

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
                let TypeSymbolKind::Struct(TypeStruct { fields: sfields, .. }) = strct.kind() 
                else {
                    wasm.error(self.error(Error::FieldAccessOnNonEnumOrStruct {
                        source, typ: value.ty }));
                    return AnalysisResult::error();
                };
                
                for f in sfields.iter() {
                    if f.name == field_name {
                        wasm.u32_const(f.offset.try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(f.ty.to_wasm_ty(&self.types));
                        return AnalysisResult::new(f.ty, value.is_mut);
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
                    
                    for a in args.iter() {
                        let scope = scope_id;
                        let anal = self.expr(a.0, scope, wasm);
                        vec.push((anal.ty, a.0.range(), if a.1 { Some(a.0) } else { None }));
                    }

                    vec
                };

                if is_accessor {
                    let ty = aargs[0].0;
                    let ns = self.namespaces.get_type(ty);
                    scope_id = self.scopes.push(
                        Scope::new(ScopeKind::ImplicitNamespace(ns), scope_id.some()));
                }

                let scope = self.scopes.get(scope_id);
                let Some(func) = scope.get_func(name, &self.scopes, &self.namespaces)
                else {
                    wasm.error(self.error(Error::FunctionNotFound { source, name }));
                    return AnalysisResult::error();
                };

                let func = self.funcs.get(func);
               
                let _ = self.call_func(
                    func, 
                    aargs.leak(),
                    source,
                    scope_id,
                    wasm
                );

                drop(pool);
                
                AnalysisResult::new(func.ret, true)
            },


            Expression::WithinNamespace { namespace, namespace_source, action } => {
                let Some(ns) = self.scopes.get(scope).get_ns(namespace, &self.scopes, &mut self.namespaces)
                else {
                    wasm.error(self.error(Error::NamespaceNotFound 
                                          { source: namespace_source, namespace }));
                    return AnalysisResult::error();
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let scope = self.scopes.push(scope);
                self.expr(*action, scope, wasm)
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

                let namespace = self.namespaces.get_type(namespace);
                let scope = Scope::new(ScopeKind::ImplicitNamespace(namespace), scope.some());
                let scope = self.scopes.push(scope);
                self.expr(*action, scope, wasm)
            },

            
            Expression::Loop { body } => {
                wasm.do_loop(|wasm, id| {
                    let nscope = LoopScope::new(id);
                    let nscope = Scope::new(ScopeKind::Loop(nscope), scope.some());
                    let nscope = self.scopes.push(nscope);

                    self.block(wasm, nscope, &*body);
                });

                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },


            Expression::Return(v) => {
                let value = self.expr(*v, scope, wasm);

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
                    let anal = self.expr(*n, scope, wasm);
                    vec.push(anal.ty);
                }

                let ty_id = match self.tuple_map.get(&*vec) {
                    Some(v) => *v,
                    None => {
                        self.make_tuple(vec.clone_in(&*pool), source)
                    },
                };

                let ty = self.types.get(ty_id);
                let TypeSymbolKind::Struct(sym) = ty.kind() else { unreachable!() };
                let ptr = wasm.alloc_stack(ty.size());

                let mut errored = false;
                for ((sf, f), n) in sym.fields.iter().zip(vec.iter()).zip(v.iter()).rev() {
                    if !sf.ty.eq_sem(*f) {
                        errored = true;
                        wasm.pop();
                        wasm.error(self.error(Error::InvalidType {
                            source: n.range(), found: *f, expected: sf.ty }));
                        continue
                    }

                    let ptr = ptr.add(sf.offset);
                    wasm.sptr_const(ptr);
                    wasm.write(sf.ty.to_wasm_ty(&self.types));
                }

                if errored { return AnalysisResult::error() }

                wasm.sptr_const(ptr);
                AnalysisResult::new(Type::Custom(ty_id), true)
            },


            Expression::CastAny { lhs, data_type  } => {
                let lhs_anal = self.expr(*lhs, scope, wasm);
                let ty = self.convert_ty(scope, data_type);
                let ty = match ty {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error();
                    }
                };

                if !lhs_anal.ty.eq_sem(Type::Any) {
                    wasm.error(self.error(Error::InvalidType {
                        source, found: lhs_anal.ty, expected: Type::Any }));
                    return AnalysisResult::error();
                }

                let ptr = wasm.local(lhs_anal.ty.to_wasm_ty(&self.types));
                wasm.local_set(ptr);
               
                // read the type id
                {
                    wasm.local_get(ptr);
                    wasm.u32_read();
                }

                wasm.u32_const(ty.type_id().as_u32());
                wasm.i32_eq();

                let option_ty = self.make_option(ty);

                wasm.ite(
                    self,
                    |slf, wasm| {
                        let tuple = slf.make_tuple(
                            Vec::from_array_in(&*ArenaPool::tls_get_rec(), [Type::I32, ty]), 
                            source);

                        let TypeSymbolKind::Struct(sym) = slf.types.get(tuple).kind()
                        else { unreachable!() };

                        let field = sym.fields[1];

                        wasm.local_get(ptr);
                        wasm.u32_const(field.offset.try_into().unwrap());
                        wasm.i32_add();

                        wasm.read(ty.to_wasm_ty(&slf.types));

                        let func = slf.namespaces.get_type(Type::Custom(option_ty));
                        let func = slf.namespaces.get(func).get_func(StringMap::SOME);
                        let func = slf.funcs.get(func.unwrap());

                        let func_ret_wasm_ty = func.ret.to_wasm_ty(&slf.types);
                        if func_ret_wasm_ty.stack_size() != 0 {
                            let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                            wasm.sptr_const(alloc);
                            wasm.call(func.wasm_id);

                            wasm.sptr_const(alloc);
                        } else {
                            wasm.call(func.wasm_id);
                        }

                        let local = wasm.local(Type::Custom(tuple).to_wasm_ty(&slf.types));
                        wasm.local_set(local);

                        (local, ())
                    }, 


                    |(slf, local), wasm| {
                        let func = slf.namespaces.get_type(Type::Custom(option_ty));
                        let func = slf.namespaces.get(func).get_func(StringMap::NONE);
                        let func = slf.funcs.get(func.unwrap());

                        let func_ret_wasm_ty = func.ret.to_wasm_ty(&slf.types);
                        if func_ret_wasm_ty.stack_size() != 0 {
                            let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                            wasm.sptr_const(alloc);
                            wasm.call(func.wasm_id);

                            wasm.sptr_const(alloc);
                        } else {
                            wasm.call(func.wasm_id);
                        }

                        wasm.local_set(local);
                    }
                );
                

                AnalysisResult::new(Type::Custom(option_ty), true)
            },


            Expression::AsCast { lhs, data_type } => {
                let lhs_anal = self.expr(*lhs, scope, wasm);
                let ty = self.convert_ty(scope, data_type);
                let ty = match ty {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error()
                    }
                };


                match (lhs_anal.ty, ty) {
                    | (Type::Error, _)
                    | (_, Type::Error) => return AnalysisResult::error(),

                    (Type::I64, Type::I32) => wasm.i64_as_i32(),
                    (Type::I64, Type::F64) => wasm.i64_as_f64(),
                    (Type::I32, Type::I64) => wasm.i32_as_i64(),
                    (Type::I32, Type::F64) => wasm.i32_as_f64(),
                    (Type::F64, Type::I64) => wasm.f64_as_i64(),
                    (Type::F64, Type::I32) => wasm.f64_as_i32(),

                    | (Type::I64, Type::I64)
                    | (Type::I32, Type::I32)
                    | (Type::F64, Type::F64) => (),

                    (Type::I64, Type::BOOL) => {
                        wasm.i64_as_i32();
                        wasm.ite(
                            &mut (), 
                            |_, wasm| {
                                let local = wasm.local(WasmType::I32);
                                wasm.i32_const(1);
                                wasm.local_set(local);
                                (local, ())
                            }, 
                            |(_, local), wasm| {
                                wasm.i32_const(0);
                                wasm.local_set(local);
                            }
                        );
                    },

                    (Type::BOOL, Type::I64) => {
                        wasm.ite(
                            &mut (), 
                            |_, wasm| {
                                let local = wasm.local(WasmType::I64);
                                wasm.i64_const(1);
                                wasm.local_set(local);
                                (local, ())
                            }, 
                            |(_, local), wasm| {
                                wasm.i64_const(0);
                                wasm.local_set(local);
                            }
                        );
                    }


                    (_, Type::Any) => {
                        let tuple = self.make_tuple(
                            Vec::from_slice_in(
                                &*ArenaPool::tls_get_rec(),
                                &[Type::I64, lhs_anal.ty]
                            ), source);
                        
                        let tuple = self.types.get(tuple);
                        let TypeSymbolKind::Struct(sym) = tuple.kind()
                        else { unreachable!() };

                        let alloc = wasm.alloc_stack(tuple.size());

                        {
                            let field = sym.fields[0];

                            wasm.u32_const(lhs_anal.ty.type_id().as_u32().try_into().unwrap());

                            wasm.sptr_const(alloc);
                            wasm.u32_const(field.offset.try_into().unwrap());
                            wasm.i32_add();

                            wasm.i32_write();
                        }

                        {
                            let field = sym.fields[1];

                            wasm.sptr_const(alloc);
                            wasm.u32_const(field.offset.try_into().unwrap());
                            wasm.i32_add();

                            wasm.write(lhs_anal.ty.to_wasm_ty(&self.types));
                        }

                        wasm.sptr_const(alloc);
                    }

                    _ => {
                        wasm.error(self.error(Error::InvalidCast {
                            range: source, from_ty: lhs_anal.ty, to_ty: ty }))
                    }
                }

                AnalysisResult::new(ty, true)
            },


            Expression::Unwrap(v) => {
                let anal = self.expr(*v, scope, wasm);
                let ty = match anal.ty {
                    Type::Custom(v) => self.types.get(v),

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::CantUnwrapOnGivenType(v.range(), anal.ty)));
                        return AnalysisResult::error();
                    }
                };
                
                let TypeSymbolKind::Enum(e) = ty.kind()
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
                wasm.ite(&mut (), |_, wasm| {
                    // TODO: Error messages, add it once errors are properly made
                    wasm.unreachable();
                    let ty = match e.kind() {
                        TypeEnumKind::TaggedUnion(v) => v.fields()[0].ty().unwrap_or(Type::Unit),
                        TypeEnumKind::Tag(_) => Type::Unit,
                    };

                    ret_ty = ty;

                    (wasm.local(ty.to_wasm_ty(&self.types)), ())
                },
                |(_, local), wasm| {
                    match e.kind() {
                        TypeEnumKind::TaggedUnion(v) => {
                            let ty = v.fields()[0].ty().unwrap_or(Type::Unit);
                            if ty == Type::Unit {
                                return
                            }

                            wasm.local_get(dup);
                            wasm.u32_const(v.union_offset().try_into().unwrap());
                            wasm.i32_add();
                            wasm.read(ty.to_wasm_ty(&self.types));
                            wasm.local_set(local)
                        },

                        TypeEnumKind::Tag(_) => wasm.unit(),
                    }

                });

                AnalysisResult::new(ret_ty, true)
            },


            Expression::OrReturn(v) => {
                let anal = self.expr(*v, scope, wasm);
                let ty = match anal.ty {
                    Type::Custom(v) => self.types.get(v),

                    Type::Error => return AnalysisResult::error(),

                    _ => {
                        wasm.error(self.error(Error::CantTryOnGivenType(v.range(), anal.ty)));
                        return AnalysisResult::error();
                    }
                };
                
                let TypeSymbolKind::Enum(enum_sym) = ty.kind()
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

                let TypeSymbolKind::Enum(func_sym) = ty.kind()
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
                wasm.ite(self, |slf, wasm| {

                    match func_sym.status() {
                        TypeEnumStatus::Option => {
                            let some_val = match enum_sym.kind() {
                                TypeEnumKind::TaggedUnion(v) => v.fields()[0].ty().unwrap_or(Type::Unit),
                                TypeEnumKind::Tag(_) => Type::Unit,
                            };

                            ret_ty = some_val;
                            let local = wasm.local(some_val.to_wasm_ty(&slf.types));

                            // Check the functions return signature for failure
                            {
                                if func_sym.status() != TypeEnumStatus::Option {
                                    wasm.error(slf.error(Error::FunctionDoesntReturnAnOption {
                                        source, func_typ: func.return_type }));
                                    return (local, ());
                                }
                            }

                            // Codegen
                            {
                                let ns = slf.namespaces.get_type(func.return_type);
                                let ns = slf.namespaces.get(ns);
                                let call_func = ns.get_func(StringMap::NONE).unwrap();
                                let call_func = slf.funcs.get(call_func);
                                let FunctionKind::UserDefined { .. } = call_func.kind
                                else { unreachable!() };

                                let func_ret_wasm_ty = func.return_type.to_wasm_ty(&slf.types);
                                if func_ret_wasm_ty.stack_size() != 0 {
                                    let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                                    wasm.sptr_const(alloc);
                                    wasm.call(call_func.wasm_id);

                                    wasm.sptr_const(alloc);
                                    wasm.ret();
                                } else {
                                    wasm.call(call_func.wasm_id);
                                    wasm.ret();
                                }

                            }


                            (local, ())
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
                            let local = wasm.local(ok.to_wasm_ty(&slf.types));

                            // Check the functions return signature for failure
                            {
                                if func_sym.status() != TypeEnumStatus::Result {
                                    wasm.error(slf.error(Error::FunctionDoesntReturnAResult {
                                        source, func_typ: func.return_type }));
                                    return (local, ());
                                }

                                let ferr = match func_sym.kind() {
                                    TypeEnumKind::TaggedUnion(v) => 
                                        v.fields()[1].ty().unwrap_or(Type::Unit),
                                    TypeEnumKind::Tag(_) => Type::Unit,
                                };

                                if !ferr.eq_sem(err) {
                                    wasm.error(slf.error(Error::FunctionReturnsAResultButTheErrIsntTheSame { 
                                        source, func_source: func.return_source, 
                                        func_err_typ: ferr, err_typ: err }));
                                    return (local, ());
                                }
                            }

                            // Codegen
                            {
                                let ns = slf.namespaces.get_type(func.return_type);
                                let ns = slf.namespaces.get(ns);
                                let call_func = ns.get_func(StringMap::ERR).unwrap();
                                let call_func = slf.funcs.get(call_func);
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
                                            wasm.read(ty.to_wasm_ty(&slf.types));
                                        },

                                        TypeEnumKind::Tag(_) => wasm.unit(),
                                    }
                                }

                                let func_ret_wasm_ty = func.return_type.to_wasm_ty(&slf.types);
                                if func_ret_wasm_ty.stack_size() != 0 {
                                    let alloc = wasm.alloc_stack(func_ret_wasm_ty.stack_size());

                                    wasm.sptr_const(alloc);
                                    wasm.call(call_func.wasm_id);

                                    wasm.sptr_const(alloc);
                                    wasm.ret();
                                } else {
                                    wasm.call(call_func.wasm_id);
                                    wasm.ret();
                                }

                            }


                            (local, ())
                        },
                        
                        _ => unreachable!(),
                    }
                },

                |(slf, local), wasm| {
                    match enum_sym.kind() {
                        TypeEnumKind::TaggedUnion(v) => {
                            let ty = v.fields()[0].ty().unwrap_or(Type::Unit);
                            if ty == Type::Unit {
                                return
                            }

                            wasm.local_get(dup);
                            wasm.u32_const(v.union_offset().try_into().unwrap());
                            wasm.i32_add();
                            wasm.read(ty.to_wasm_ty(&slf.types));
                            wasm.local_set(local)
                        },

                        TypeEnumKind::Tag(_) => wasm.unit(),
                    }
                });

                AnalysisResult::new(ret_ty, true)
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
                let Some(val) = self.scopes.get(scope).get_var(ident, &self.scopes)
                else {
                    return Err(Error::VariableNotFound { name: ident, source: node.range() });
                };

                if !val.is_mutable {
                    return Err(Error::ValueUpdateNotMut { source: node.range() });
                }

                if depth == 0 {
                    wasm.local_set(val.local_id);

                    if !val.ty.eq_sem(val_ty) {
                        return Err(Error::ValueUpdateTypeMismatch 
                                   { lhs: val.ty, rhs: val_ty, source: node.range() })
                    }

                    return Ok(val.ty);
                }

                wasm.local_get(val.local_id);
                Ok(val.ty)
            }

            
            Expression::AccessField { val, field_name } => {
                let ty = self.assign(wasm, scope, *val, val_ty, depth + 1)?;

                let tyid = match ty {
                    Type::Custom(v) => v,

                    Type::Error => return Err(Error::Bypass),

                    _ => {
                        return Err(Error::FieldAccessOnNonEnumOrStruct {
                            source: node.range(), typ: ty });
                    }
                };


                let strct = self.types.get(tyid);
                let TypeSymbolKind::Struct(TypeStruct { fields: sfields, .. }) = strct.kind() 
                else {
                    return Err(Error::FieldAccessOnNonEnumOrStruct {
                        source: node.range(), typ: ty });
                };

                for sf in sfields.iter() {
                    if sf.name == field_name {
                        wasm.u32_const(sf.offset.try_into().unwrap());
                        wasm.i32_add();

                        if depth == 0 {
                            if !sf.ty.eq_sem(val_ty) {
                                return Err(Error::ValueUpdateTypeMismatch 
                                           { lhs: sf.ty, rhs: val_ty, source: node.range() })
                            }
                            wasm.write(val_ty.to_wasm_ty(&self.types));
                        }

                        return Ok(sf.ty);
                    }
                }

                Err(Error::FieldDoesntExist {
                    source: node.range(), field: field_name, typ: ty })
            }

            _ => return Err(Error::AssignIsNotLHSValue { source: node.range() }),
        }
    }


    fn call_func(
        &mut self,
        func: Function,
        args: &[(Type, SourceRange, Option<ExpressionNode>)],
        source: SourceRange,
        scope: ScopeId,

        wasm: &mut WasmFunctionBuilder,
    ) -> Result<(), ()> {
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
                for (sig_arg, call_arg) in func.args.iter().zip(args.iter()) {
                    if !sig_arg.2.eq_sem(call_arg.0) {
                        errored = true;
                        wasm.error(self.error(Error::InvalidType {
                            source: call_arg.1, found: call_arg.0, expected: sig_arg.2 }));
                    }

                    if sig_arg.1 && call_arg.2.is_none() {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }

                    if !sig_arg.1 && call_arg.2.is_some() {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }
                }

                if errored {
                    return Err(());
                }

                wasm.call(func.wasm_id);

                if let Some(sp) = ret {
                    wasm.sptr_const(sp);
                }

                if let Some((ty, sp)) = inout {
                    let TypeSymbolKind::Struct(sym) = ty.kind() else { unreachable!() };
                    let mut c = 0;
                    for (i, sig_arg) in func.args.iter().enumerate() {
                        if !sig_arg.1 { continue }

                        let field = sym.fields[c];
                        wasm.sptr_const(sp);
                        wasm.u32_const(field.offset.try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(sig_arg.2.to_wasm_ty(&self.types));

                        if let Err(e) = self.assign(
                                wasm, scope, args[i].2.unwrap(), sig_arg.2, 0) {
                            wasm.error(self.error(e));
                        }

                        c += 1;

                    }
                }
            },

            FunctionKind::Extern { ty } => {
                let mut errored = false;
                for (sig_arg, call_arg) in func.args.iter().zip(args.iter()) {
                    if !sig_arg.2.eq_sem(call_arg.0) {
                        errored = true;
                        wasm.error(self.error(Error::InvalidType {
                            source: call_arg.1, found: call_arg.0, expected: sig_arg.2 }));
                    }

                    if sig_arg.1 && call_arg.2.is_none() {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }

                    if !sig_arg.1 && call_arg.2.is_some() {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }
                }

                if errored {
                    return Err(());
                }

                let ty_sym = self.types.get(ty);
                let ptr = {
                    wasm.alloc_stack(ty_sym.size())
                };

                let TypeSymbolKind::Struct(sym) = ty_sym.kind() else { unreachable!() };
                
                for sym_arg in sym.fields.iter().rev().skip(1) {
                    wasm.sptr_const(ptr);
                    wasm.u32_const(sym_arg.offset.try_into().unwrap());
                    wasm.i32_add();

                    wasm.write(sym_arg.ty.to_wasm_ty(&self.types));
                }

                wasm.sptr_const(ptr);
                wasm.call(func.wasm_id);

                let mut c = 0;
                for (i, sig_arg) in func.args.iter().enumerate() {
                    if !sig_arg.1 { continue }

                    let field = sym.fields[c];
                    wasm.sptr_const(ptr);
                    wasm.u32_const(field.offset.try_into().unwrap());
                    wasm.i32_add();
                    wasm.read(sig_arg.2.to_wasm_ty(&self.types));

                    if let Err(e) = self.assign(
                            wasm, scope, args[i].2.unwrap(), sig_arg.2, 0) {
                        wasm.error(self.error(e));
                    }

                    c += 1;

                }

                {
                    let r = sym.fields.last().unwrap();
                    wasm.sptr_const(ptr);
                    wasm.u32_const(r.offset.try_into().unwrap());
                    wasm.i32_add();

                    wasm.read(r.ty.to_wasm_ty(&self.types));
                }
            },


            FunctionKind::Template { body, scope } => {
                todo!()
            },
        };

        Ok(())
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
