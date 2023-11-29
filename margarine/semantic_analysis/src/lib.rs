pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use std::{task::Wake, any::TypeId};

use common::{source::SourceRange, string_map::StringMap};
use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Function};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{nodes::{Node, NodeKind, Expression, Declaration, BinaryOperator, UnaryOperator}, DataTypeKind, DataType};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind, FunctionDefinitionScope, VariableScope};
use types::{Type, TypeMap, FieldBlueprint};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::PackedOption, arena_pool::ArenaPool};

use crate::types::TypeBuilder;

#[derive(Debug)]
pub struct Analyzer<'out> {
    scopes: ScopeMap,
    namespaces: NamespaceMap,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,
    output: &'out Arena,

    pub module_builder: WasmModuleBuilder<'out>,
    pub errors: KVec<SemaError, Error>
}


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
        string_map: &mut StringMap,
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

        let main_name = string_map.insert("_init");
        let mut func = WasmFunctionBuilder::new(output, slf.module_builder.function_id());
        let scope = Scope::new(ScopeKind::Root, PackedOption::NONE);
        let scope = slf.scopes.push(scope);

        func.export(main_name);

        slf.block(&mut func, scope, nodes);
        func.pop();

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


        for (id, n) in nodes.iter().enumerate() {
            self.node(scope, builder, n);

            if id + 1 != nodes.len() {
                builder.pop();
            }
        }

        AnalysisResult { ty: Type::Unit, is_mut: true }
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
                    type_builder.add_ty(ty, name, header, false)
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

                            args.push((arg.name(), arg.is_inout(), ty));
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


    fn node(
        &mut self,
        scope: ScopeId,
        wasm: &mut WasmFunctionBuilder,

        node: &Node<'_>,
    ) -> AnalysisResult {
        match node.kind() {
            NodeKind::Declaration(decl) => {
                self.decl(decl, node.range(), scope);
                wasm.i64_const(0);
                AnalysisResult::new(Type::Unit, true)
            },

            NodeKind::Statement(_) => todo!(),
            NodeKind::Expression(expr) => self.expr(scope, expr, node.range(), wasm),
            NodeKind::Error(err) => {
                wasm.error(*err);
                wasm.i64_const(0);
                AnalysisResult::error()
            },
        }
    }


    fn decl(
        &mut self,
        decl: &Declaration,
        source: SourceRange,
        scope: ScopeId,
    ) {
        match decl {
            Declaration::Struct { kind, name, header, fields } => (),
            Declaration::Enum { name, header, mappings } => (),


            Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                let wasm = self.module_builder.function_id();
                let mut wasm = WasmFunctionBuilder::new(self.output, wasm);
                
                let func = self.scopes.get(scope).get_func(*name, &self.scopes, &self.namespaces).unwrap();
                let func = self.funcs.get(func);

                wasm.return_value(func.ret.to_wasm_ty());

                let scope = Scope::new(
                    ScopeKind::FunctionDefinition(
                        FunctionDefinitionScope::new(func.ret)
                    ),
                    scope.some()
                );

                let mut scope = self.scopes.push(scope);

                for a in func.args.iter() {
                    let wasm_ty = a.2.to_wasm_ty();

                    let local_id = wasm.param(wasm_ty);

                    let t = Scope::new(
                        ScopeKind::Variable(VariableScope::new(a.0, a.1, a.2, local_id)),
                        scope.some()
                    );

                    scope = self.scopes.push(t);
                }

                self.block(&mut wasm, scope, &body);

                self.module_builder.register(wasm);
            },


            Declaration::Impl { data_type, body } => todo!(),
            Declaration::Using { file } => todo!(),
            Declaration::Module { name, body } => todo!(),
            Declaration::Extern { file, functions } => todo!(),
        }
    }


    fn expr(
        &mut self,
        scope: ScopeId,
        expr: &Expression,
        source: SourceRange,

        wasm: &mut WasmFunctionBuilder,
    ) -> AnalysisResult {
        match expr {
            Expression::Unit => {
                wasm.i64_const(0);
                AnalysisResult::new(Type::Unit, true)
            },

            Expression::Literal(l) => {
                match l {
                    lexer::Literal::Integer(i) => {
                        wasm.i64_const(*i);
                        AnalysisResult::new(Type::Int, true)
                    },


                    lexer::Literal::Float(f) => {
                        wasm.f64_const(f.inner());
                        AnalysisResult::new(Type::Float, true)
                    },


                    lexer::Literal::String(_) => todo!(),
                    lexer::Literal::Bool(_) => todo!(),
                }
            },


            Expression::Identifier(ident) => {
                let Some(variable) = self.scopes.get(scope).get_var(*ident, &self.scopes)
                else {
                    wasm.error(self.error(Error::VariableNotFound { name: *ident, source }));
                    return AnalysisResult::error()
                };

                wasm.local_get(variable.local_id);
                AnalysisResult::new(variable.ty, variable.is_mutable)
            },


            Expression::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.node(scope, wasm, lhs);
                let rhs_anal = self.node(scope, wasm, rhs);

                let mut type_check = || {
                    if lhs_anal.ty.eq_lit(Type::Error)
                        || rhs_anal.ty.eq_lit(Type::Error)
                        {
                            return Err(())
                    }

                    if operator.is_arith() 
                        && !(lhs_anal.ty.is_number() && rhs_anal.ty.is_number()) {
                        wasm.error(self.error(Error::InvalidBinaryOp {
                            operator: *operator, lhs: lhs_anal.ty,
                            rhs: rhs_anal.ty, source 
                        }));

                        return Err(())
                    }

                    Ok(())
                };

                if type_check().is_err() {
                    wasm.pop();
                    wasm.pop();
                    wasm.i64_const(0);
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
                    (BinaryOperator::Add, Type::Int) => wfunc!(i64_add, Type::Int),
                    (BinaryOperator::Add, Type::UInt) => wfunc!(i64_add, Type::UInt),
                    (BinaryOperator::Add, Type::Float) => wfunc!(f64_add, Type::Float),

                    (BinaryOperator::Sub, Type::Int) => wfunc!(i64_sub, Type::Int),
                    (BinaryOperator::Sub, Type::UInt) => wfunc!(i64_sub, Type::Int),
                    (BinaryOperator::Sub, Type::Float) => wfunc!(f64_sub, Type::Float),

                    (BinaryOperator::Mul, Type::Int) => wfunc!(i64_mul, Type::Int),
                    (BinaryOperator::Mul, Type::UInt) => wfunc!(i64_mul, Type::Int),
                    (BinaryOperator::Mul, Type::Float) => wfunc!(f64_mul, Type::Float),

                    (BinaryOperator::Div, Type::Int) => wfunc!(i64_div, Type::Int),
                    (BinaryOperator::Div, Type::UInt) => wfunc!(u64_div, Type::Int),
                    (BinaryOperator::Div, Type::Float) => wfunc!(f64_div, Type::Int),

                    (BinaryOperator::Rem, Type::Int) => wfunc!(i64_rem, Type::Int),
                    (BinaryOperator::Rem, Type::UInt) => wfunc!(u64_rem, Type::Int),
                    (BinaryOperator::Rem, Type::Float) => wfunc!(f64_rem, Type::Int),

                    (BinaryOperator::BitshiftLeft, Type::Int) => wfunc!(i64_bw_left_shift, Type::Int),
                    (BinaryOperator::BitshiftLeft, Type::UInt) => wfunc!(i64_bw_left_shift, Type::UInt),

                    (BinaryOperator::BitshiftRight, Type::Int) => wfunc!(i64_bw_right_shift, Type::Int),
                    (BinaryOperator::BitshiftRight, Type::UInt) => wfunc!(u64_bw_right_shift, Type::Int),

                    (BinaryOperator::BitwiseAnd, Type::Int) => wfunc!(i64_bw_and, Type::Int),
                    (BinaryOperator::BitwiseAnd, Type::UInt) => wfunc!(i64_bw_and, Type::UInt),

                    (BinaryOperator::BitwiseOr, Type::Int) => wfunc!(i64_bw_or, Type::Int),
                    (BinaryOperator::BitwiseOr, Type::UInt) => wfunc!(i64_bw_or, Type::UInt),

                    (BinaryOperator::BitwiseXor, Type::Int) => wfunc!(i64_bw_xor, Type::Int),
                    (BinaryOperator::BitwiseXor, Type::UInt) => wfunc!(i64_bw_xor, Type::Int),

                    (BinaryOperator::Eq, Type::Int) => todo!(),
                    (BinaryOperator::Eq, Type::UInt) => todo!(),
                    (BinaryOperator::Eq, Type::Float) => todo!(),
                    (BinaryOperator::Eq, Type::Any) => todo!(),
                    (BinaryOperator::Eq, Type::Unit) => todo!(),
                    (BinaryOperator::Eq, Type::Never) => todo!(),
                    (BinaryOperator::Eq, Type::Error) => todo!(),
                    (BinaryOperator::Eq, Type::Custom(_)) => todo!(),
                    (BinaryOperator::Ne, Type::Int) => todo!(),
                    (BinaryOperator::Ne, Type::UInt) => todo!(),
                    (BinaryOperator::Ne, Type::Float) => todo!(),
                    (BinaryOperator::Ne, Type::Any) => todo!(),
                    (BinaryOperator::Ne, Type::Unit) => todo!(),
                    (BinaryOperator::Ne, Type::Never) => todo!(),
                    (BinaryOperator::Ne, Type::Error) => todo!(),
                    (BinaryOperator::Ne, Type::Custom(_)) => todo!(),
                    (BinaryOperator::Gt, Type::Int) => todo!(),
                    (BinaryOperator::Gt, Type::UInt) => todo!(),
                    (BinaryOperator::Gt, Type::Float) => todo!(),
                    (BinaryOperator::Gt, Type::Any) => todo!(),
                    (BinaryOperator::Gt, Type::Unit) => todo!(),
                    (BinaryOperator::Gt, Type::Never) => todo!(),
                    (BinaryOperator::Gt, Type::Error) => todo!(),
                    (BinaryOperator::Gt, Type::Custom(_)) => todo!(),
                    (BinaryOperator::Ge, Type::Int) => todo!(),
                    (BinaryOperator::Ge, Type::UInt) => todo!(),
                    (BinaryOperator::Ge, Type::Float) => todo!(),
                    (BinaryOperator::Ge, Type::Any) => todo!(),
                    (BinaryOperator::Ge, Type::Unit) => todo!(),
                    (BinaryOperator::Ge, Type::Never) => todo!(),
                    (BinaryOperator::Ge, Type::Error) => todo!(),
                    (BinaryOperator::Ge, Type::Custom(_)) => todo!(),
                    (BinaryOperator::Lt, Type::Int) => todo!(),
                    (BinaryOperator::Lt, Type::UInt) => todo!(),
                    (BinaryOperator::Lt, Type::Float) => todo!(),
                    (BinaryOperator::Lt, Type::Any) => todo!(),
                    (BinaryOperator::Lt, Type::Unit) => todo!(),
                    (BinaryOperator::Lt, Type::Never) => todo!(),
                    (BinaryOperator::Lt, Type::Error) => todo!(),
                    (BinaryOperator::Lt, Type::Custom(_)) => todo!(),
                    (BinaryOperator::Le, Type::Int) => todo!(),
                    (BinaryOperator::Le, Type::UInt) => todo!(),
                    (BinaryOperator::Le, Type::Float) => todo!(),
                    (BinaryOperator::Le, Type::Any) => todo!(),
                    (BinaryOperator::Le, Type::Unit) => todo!(),
                    (BinaryOperator::Le, Type::Never) => todo!(),
                    (BinaryOperator::Le, Type::Error) => todo!(),
                    (BinaryOperator::Le, Type::Custom(_)) => todo!(),

                    _ => unreachable!()
                };

                AnalysisResult::new(ty, true)
            },


            Expression::UnaryOp { operator, rhs } => {
                let rhs_anal = self.node(scope, wasm, rhs);
                
                let mut type_check = || {
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(())
                    }

                    if *operator == UnaryOperator::Not
                        && !rhs_anal.ty.eq_sem(self.types.bool()) {

                        wasm.error(self.error(Error::InvalidUnaryOp {
                            operator: *operator, rhs: rhs_anal.ty, source
                        }));

                        return Err(())

                    } else if *operator == UnaryOperator::Neg
                        && !rhs_anal.ty.is_number() {

                        wasm.error(self.error(Error::InvalidUnaryOp {
                            operator: *operator, rhs: rhs_anal.ty, source
                        }));

                        return Err(())
                    }

                    Ok(())
                };

                if type_check().is_err() {
                    wasm.pop();
                    wasm.i64_const(0);
                    return AnalysisResult::error();
                }

                match (operator, rhs_anal.ty) {
                    (UnaryOperator::Not, Type::Custom(_)) => todo!(),

                    (UnaryOperator::Neg, Type::Int) => {
                        // thanks wasm.
                        wasm.i64_const(-1);
                        wasm.i64_mul();
                    },

                    (UnaryOperator::Neg, Type::Float) => wasm.f64_neg(),

                    _ => unreachable!()
                }

                AnalysisResult::new(rhs_anal.ty, true)
            },


            Expression::If { condition, body, else_block } => todo!(),
            Expression::Match { value, taken_as_inout, mappings } => todo!(),
            Expression::Block { block } => todo!(),
            Expression::CreateStruct { data_type, fields } => todo!(),
            Expression::AccessField { val, field_name } => todo!(),
            Expression::CallFunction { name, is_accessor, args } => todo!(),
            Expression::WithinNamespace { namespace, namespace_source, action } => todo!(),
            Expression::WithinTypeNamespace { namespace, action } => todo!(),
            Expression::Loop { body } => todo!(),
            Expression::Return(_) => todo!(),
            Expression::Continue => todo!(),
            Expression::Break => todo!(),
            Expression::CastAny { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
        }
    }
}


