pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::FunctionMap;
use namespace::{Namespace, NamespaceMap};
use parser::{nodes::{Node, NodeKind}, DataTypeKind, DataType};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind};
use types::{Type, TypeMap};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::PackedOption};

#[derive(Debug)]
pub struct Analyzer<'out> {
    scopes: ScopeMap,
    namespaces: NamespaceMap,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,

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
        let scope = {
            let mut namespace = Namespace::new(); 

            {
                let scope = self.scopes.get(scope);
                self.collect_names(builder, nodes, &mut namespace);
            }

            dbg!(&namespace);
            let namespace_id = self.namespaces.put(namespace);
            Scope::new(ScopeKind::ImplicitNamespace(namespace_id), scope.some())
        };

        AnalysisResult { ty: Type::Unit }
    }
}


impl Analyzer<'_> {
    pub fn collect_names(
        &mut self,
        builder: &mut WasmFunctionBuilder,
        nodes: &[Node],
        
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


                    /*
                    let placeholder = self.types.uninit();
                    namespace.add_type(name, placeholder);
                    */
                },


                parser::nodes::Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                    if namespace.get_func(name).is_some() {
                        builder.error(self.error(Error::NameIsAlreadyDefined { 
                           source: header, name }));

                        continue
                    }

                    todo!();
                },


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

        namespace: &mut Namespace
    ) {
        for node in nodes {
            let source = node.range();

            let NodeKind::Declaration(decl) = node.kind()
            else { continue };

            match decl {
                parser::nodes::Declaration::Struct { kind, name, header, fields } => {

                },
                parser::nodes::Declaration::Enum { name, header, mappings } => todo!(),
                parser::nodes::Declaration::Function { is_system, name, header, arguments, return_type, body } => todo!(),
                parser::nodes::Declaration::Impl { data_type, body } => todo!(),
                parser::nodes::Declaration::Using { file } => todo!(),
                parser::nodes::Declaration::Module { name, body } => todo!(),
                parser::nodes::Declaration::Extern { file, functions } => todo!(),
            }
       }

    }
}
