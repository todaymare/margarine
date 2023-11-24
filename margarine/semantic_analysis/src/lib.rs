pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use std::task::Wake;

use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Function};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{nodes::{Node, NodeKind}, DataTypeKind, DataType};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind};
use types::{Type, TypeMap, FieldBlueprint};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::PackedOption, arena_pool::ArenaPool};

use crate::types::TypeBuilder;

#[derive(Debug)]
pub struct Analyzer<'out> {
    scopes: ScopeMap,
    namespaces: NamespaceMap,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,
    output: &'out Arena,

    module_builder: WasmModuleBuilder<'out>,
    pub errors: KVec<SemaError, Error>
}


pub struct AnalysisResult {
    ty: Type,
}


impl Analyzer<'_> {
     pub fn convert_ty(&self, scope: ScopeId, dt: DataType) -> Result<Type, Error> {
        let ty = match dt.kind() {
            DataTypeKind::Int => Type::Int,
            DataTypeKind::Bool => todo!(),
            DataTypeKind::Float => Type::Float,
            DataTypeKind::Unit => Type::Unit,
            DataTypeKind::Any => todo!(),
            DataTypeKind::Never => Type::Never,
            DataTypeKind::Option(_) => todo!(),
            DataTypeKind::Result(_, _) => todo!(),
            DataTypeKind::CustomType(v) => {
                let scope = self.scopes.get(scope);
                let Some(ty) = scope.get_type(*v, &self.scopes, &self.namespaces)
                else { return Err(Error::UnknownType(*v, dt.range())) };

                Type::Custom(ty)
            },
        };

        Ok(ty)
    }


    pub fn error(&mut self, err: Error) -> ErrorId {
        ErrorId::Sema(self.errors.push(err))
    }
}


impl<'out> Analyzer<'out> {
    pub fn run(
        output: &'out Arena,
        nodes: &[Node],
    ) -> Self {
        let mut slf = Self {
            scopes: ScopeMap::new(),
            namespaces: NamespaceMap::new(),
            types: TypeMap::new(),
            funcs: FunctionMap::new(),
            module_builder: WasmModuleBuilder::new(),
            errors: KVec::new(),
            output,
        };

        let mut func = WasmFunctionBuilder::new(output, slf.module_builder.function_id());
        let scope = Scope::new(ScopeKind::Root, PackedOption::NONE);
        let scope = slf.scopes.push(scope);

        slf.block(&mut func, scope, nodes);

        slf.module_builder.register(func);

        slf
    }


    pub fn block(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        scope: ScopeId,
        nodes: &[Node],
    ) -> AnalysisResult {
        let pool = ArenaPool::tls_get_rec();
        let mut ty_builder = TypeBuilder::new(&*pool); 
        let (scope, ns_id) = {
            let mut namespace = Namespace::new();

            self.collect_type_names(builder, nodes, &mut ty_builder, &mut namespace);

            let namespace_id = self.namespaces.put(namespace);
            (Scope::new(ScopeKind::ImplicitNamespace(namespace_id), scope.some()), namespace_id)
        };
        
        let scope = self.scopes.push(scope);

        self.resolve_names(nodes, builder, &mut ty_builder, scope, ns_id);
        {
            let err_len = self.errors.len();

            ty_builder.finalise(self.output, &mut self.types, &mut self.errors);

            for i in err_len..self.errors.len() {
                builder.error(ErrorId::Sema(SemaError::new((err_len + i) as u32).unwrap()))
            }
        }

        AnalysisResult { ty: Type::Unit }
    }
}


impl Analyzer<'_> {
    pub fn collect_type_names(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        nodes: &[Node],
        
        type_builder: &mut TypeBuilder,
        namespace: &mut Namespace,
    ) {
        for node in nodes {
            let source = node.range();

            let NodeKind::Declaration(decl) = node.kind()
            else { continue };

            match *decl {
                | parser::nodes::Declaration::Enum { name, header, .. }
                | parser::nodes::Declaration::Struct { name, header, .. } => {
                    if namespace.get_type(name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: header, name }));

                        continue
                    }

                    let ty = self.types.pending();
                    namespace.add_type(name, ty);
                    type_builder.add_ty(ty, name, header)
                },


                parser::nodes::Declaration::Function { .. } => {},

                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
        }
    }


    pub fn resolve_names(
        &mut self,
        nodes: &[Node],

        builder: &mut WasmFunctionBuilder,
        type_builder: &mut TypeBuilder,
        scope: ScopeId,
        ns_id: NamespaceId,
    ) {
        for node in nodes {
            let source = node.range();

            let NodeKind::Declaration(decl) = node.kind()
            else { continue };

            match decl {
                parser::nodes::Declaration::Struct { kind, name, header, fields } => {
                    let fields = {
                        let mut vec = Vec::with_cap_in(type_builder.alloc(), fields.len());
                        
                        for (name, ty, _) in fields.iter() {
                            let ty = self.convert_ty(scope, *ty);
                            let ty = match ty {
                                Ok(v) => v,
                                Err(v) => {
                                    self.error(v);
                                    continue;
                                },
                            };

                            vec.push(FieldBlueprint::new(*name, ty))
                        }

                        vec.leak()
                    };

                    let ty = self.namespaces.get(ns_id).get_type(*name).unwrap();
                    type_builder.add_fields(ty, fields)
                },


                parser::nodes::Declaration::Enum { name, header, mappings } => todo!(),


                parser::nodes::Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                    let ns = self.namespaces.get(ns_id);
                    if ns.get_func(*name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: *header, name: *name }));

                        continue
                    }

                    let args = {
                        let mut args = Vec::with_cap_in(self.output, arguments.len());

                        for arg in arguments.iter() {
                            let ty = self.convert_ty(scope, arg.data_type());
                            let ty = match ty {
                                Ok(v) => v,
                                Err(e) => {
                                    builder.error(self.error(e));
                                    Type::Error
                                },
                            };

                            args.push(ty);
                        }

                        args
                    };

                    let ret = match self.convert_ty(scope, *return_type) {
                        Ok(v) => v,
                        Err(e) => {
                            builder.error(self.error(e));
                            Type::Error
                        },
                    };

                    let ns = self.namespaces.get_mut(ns_id);
                    let func = Function::new(*name, args.leak(), ret);
                    let func = self.funcs.put(func);
                    ns.add_func(*name, func);
                },


                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
       }

    }
}
