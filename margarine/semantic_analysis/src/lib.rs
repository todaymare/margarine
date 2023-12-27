pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use std::fmt::Write;

use common::{source::SourceRange, string_map::{StringMap, StringIndex}};
use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Function};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{nodes::{Node, NodeKind, Expression, Declaration, BinaryOperator, UnaryOperator, Statement}, DataTypeKind, DataType};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind, FunctionDefinitionScope, VariableScope, LoopScope};
use types::{ty::Type, ty_map::TypeMap, ty_sym::{TypeEnum, TypeSymbolKind}};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType, FunctionId, StackPointer, LocalId};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::{PackedOption, Reserved}, arena_pool::ArenaPool, hash::HashMap};

use crate::types::{ty_map::TypeId, ty_builder::{TypeBuilder, TypeBuilderData, PartialStructField}, ty_sym::{StructField, TypeStruct}};

#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'str> {
    scopes: ScopeMap,
    namespaces: NamespaceMap,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,
    output: &'out Arena,
    pub string_map: &'me mut StringMap<'str>,

    pub module_builder: WasmModuleBuilder<'out, 'str>,
    pub errors: KVec<SemaError, Error>,

    options_map: HashMap<Type, TypeId>,
    results_map: HashMap<(Type, Type), TypeId>,
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


impl Analyzer<'_, '_, '_> {
     pub fn convert_ty(&mut self, scope: ScopeId, dt: DataType) -> Result<Type, Error> {
        let ty = match dt.kind() {
            DataTypeKind::Int => Type::I64,
            DataTypeKind::Bool => Type::BOOL,
            DataTypeKind::Float => Type::F64,
            DataTypeKind::Unit => Type::Unit,
            DataTypeKind::Any => todo!(),
            DataTypeKind::Never => Type::Never,
            DataTypeKind::Option(v) => {
                let inner_ty = self.convert_ty(scope, *v)?;
                if let Some(v) = self.options_map.get(&inner_ty) { return Ok(Type::Custom(*v)); }

                let tyid = {
                    let temp = ArenaPool::tls_get_temp();
                    let name = {
                        let mut str = sti::string::String::new_in(&*temp);
                        str.push(inner_ty.display(self.string_map, &self.types));
                        str.push_char('?');

                        self.string_map.insert(str.as_str())
                    };

                    let mut tyb = TypeBuilder::new(&temp);

                    let tyid = self.types.pending();
                    tyb.add_ty(tyid, name, dt.range());

                    let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
                    tyb.finalise(data, &mut self.errors);

                    tyid
                };

                self.options_map.insert(inner_ty, tyid);

                Type::Custom(tyid)
            },


            DataTypeKind::Result(v1, v2) => {
                let inner_ty1 = self.convert_ty(scope, *v1)?;
                let inner_ty2 = self.convert_ty(scope, *v2)?;
                if let Some(v) = self.results_map.get(&(inner_ty1, inner_ty2))
                    { return Ok(Type::Custom(*v)); }

                let tyid = {
                    let temp = ArenaPool::tls_get_temp();
                    let name = {
                        let mut str = sti::string::String::new_in(&*temp);
                        str.push(inner_ty1.display(self.string_map, &self.types));
                        str.push(" ~ ");
                        str.push(inner_ty2.display(self.string_map, &self.types));

                        self.string_map.insert(str.as_str())
                    };

                    let mut tyb = TypeBuilder::new(&temp);

                    let tyid = self.types.pending();
                    tyb.add_ty(tyid, name, dt.range());

                    let data = TypeBuilderData::new(&mut self.types, &mut self.namespaces, &mut self.funcs, &mut self.module_builder);
                    tyb.finalise(data, &mut self.errors);

                    tyid
                };

                self.results_map.insert((inner_ty1, inner_ty2), tyid);

                Type::Custom(tyid)
            },


            DataTypeKind::CustomType(v) => {
                if v == StringMap::STR { return Ok(Type::STR) }
                let scope = self.scopes.get(scope);
                let Some(ty) = scope.get_type(v, &self.scopes, &self.namespaces)
                else { return Err(Error::UnknownType(v, dt.range())) };

                Type::Custom(ty)
            },
        };

        Ok(ty)
    }
     

    pub fn error(&mut self, err: Error) -> ErrorId {
        ErrorId::Sema(self.errors.push(err))
    }
}


impl<'me, 'out, 'str> Analyzer<'me, 'out, 'str> {
    pub fn run(
        output: &'out Arena,
        string_map: &'me mut StringMap<'str>,
        nodes: &[Node],
    ) -> Self {
        let mut slf = Self {
            scopes: ScopeMap::new(),
            namespaces: NamespaceMap::new(),
            types: TypeMap::new(),
            funcs: FunctionMap::new(),
            module_builder: WasmModuleBuilder::new(output),
            errors: KVec::new(),
            output,
            string_map,
            options_map: HashMap::new(),
            results_map: HashMap::new(),
        };

        slf.module_builder.memory(64 * 1024);

        {
            let pool = ArenaPool::tls_get_temp();
            let mut type_builder = TypeBuilder::new(&pool);

            {
                let id = slf.types.pending();
                assert_eq!(TypeId::BOOL, id);

                type_builder.add_ty(TypeId::BOOL, StringMap::BOOL, SourceRange::new(0, 0));
                type_builder.set_enum_fields(
                    TypeId::BOOL,
                    [(StringMap::TRUE, None), (StringMap::FALSE, None)].into_iter()
                );
            }
            {
                let id = slf.types.pending();
                assert_eq!(TypeId::STR, id);

                type_builder.add_ty(TypeId::STR, StringMap::STR, SourceRange::new(0, 0));
                type_builder.set_struct_fields(
                    TypeId::STR,
                    [
                        (slf.string_map.insert("ptr"), Type::I32),
                        (slf.string_map.insert("len"), Type::I64),
                    ].into_iter(),
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
        
        let mut scope = self.scopes.push(scope);

        self.resolve_names(nodes, builder, &mut ty_builder, scope, ns_id);
        
        {
            let err_len = self.errors.len();

            let data = TypeBuilderData::new(
                &mut self.types, &mut self.namespaces,
                &mut self.funcs, &mut self.module_builder
            );

            ty_builder.finalise(data, &mut self.errors);

            for i in err_len..self.errors.len() {
                builder.error(ErrorId::Sema(SemaError::new((err_len + i) as u32).unwrap()))
            }
        }

        let mut ty = Type::Unit;
        for (id, n) in nodes.iter().enumerate() {
            ty = self.node(&mut scope, builder, n).ty;

            if id + 1 != nodes.len() {
                builder.pop();
            } 

        }

        if nodes.is_empty() { builder.unit(); }

        AnalysisResult { ty, is_mut: true }
    }
}


impl Analyzer<'_, '_, '_> {
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
                    type_builder.add_ty(ty, name, header);
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
                    let ty = self.namespaces.get(ns_id).get_type(*name).unwrap();
                    let fields = fields.iter()
                        .filter_map(|(name, ty, _)| {
                            let ty = self.convert_ty(scope, *ty);
                            match ty {
                                Ok(v) => return Some((*name, v)),
                                Err(e) => self.error(e),
                            };

                            None
                        });

                    type_builder.set_struct_fields(ty, fields);
                    dbg!(&type_builder);
                },


                parser::nodes::Declaration::Enum { name, header, mappings } => {
                    let ty = self.namespaces.get(ns_id).get_type(*name).unwrap();
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

                    type_builder.set_enum_fields(ty, mappings)
                },


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
                    let func = Function::new(*name, args.leak(), ret, self.module_builder.function_id());
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
        scope: &mut ScopeId,
        wasm: &mut WasmFunctionBuilder,

        node: &Node<'_>,
    ) -> AnalysisResult {
        match node.kind() {
            NodeKind::Declaration(decl) => {
                self.decl(decl, node.range(), scope);
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },

            NodeKind::Statement(stmt) => {
                if self.stmt(stmt, node.range(), scope, wasm).is_err() {
                    wasm.unit();
                    return AnalysisResult::error()
                }
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)

            },
            NodeKind::Expression(expr) => self.expr(expr, node.range(), scope, wasm),
            NodeKind::Error(err) => {
                wasm.error(*err);
                wasm.unit();
                AnalysisResult::error()
            },
        }
    }


    fn decl(
        &mut self,
        decl: &Declaration,
        source: SourceRange,
        scope: &mut ScopeId,
    ) {
        match decl {
            Declaration::Struct { .. } => (),
            Declaration::Enum { .. } => (),


            Declaration::Function { is_system, name, header, arguments, return_type, body } => {
                
                let func = self.scopes.get(*scope).get_func(*name, &self.scopes, &self.namespaces).unwrap();
                let func = self.funcs.get(func);
                let mut wasm = WasmFunctionBuilder::new(self.output, func.wasm_id);

                wasm.return_value(func.ret.to_wasm_ty(&self.types));

                let scope = Scope::new(
                    ScopeKind::FunctionDefinition(
                        FunctionDefinitionScope::new(func.ret, return_type.range())
                    ),
                    scope.some()
                );

                let mut scope = self.scopes.push(scope);

                for a in func.args.iter() {
                    let wasm_ty = a.2.to_wasm_ty(&self.types);

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


    fn stmt(
        &mut self,
        stmt: &Statement,
        source: SourceRange,

        scope: &mut ScopeId,
        wasm: &mut WasmFunctionBuilder,
    ) -> Result<(), ()> {
        match stmt {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let mut func = || -> Result<(), ()> {
                    let rhs_anal = self.node(scope, wasm, rhs);
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(());
                    }

                    if let Some(hint) = hint {
                        let hint = match self.convert_ty(*scope, *hint) {
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

                    let variable_scope = VariableScope::new(*name, *is_mut, rhs_anal.ty, local);
                    *scope = self.scopes.push(
                        Scope::new(ScopeKind::Variable(variable_scope), scope.some()));

                    Ok(())
                };

                if func().is_err() {
                    let dummy = VariableScope::new(*name, true, Type::Error, wasm.local(WasmType::I64));
                    *scope = self.scopes.push(Scope::new(ScopeKind::Variable(dummy), scope.some()));
                    return Err(());
                }
                
            },


            Statement::UpdateValue { lhs, rhs } => {
                let rhs_anal = self.node(scope, wasm, rhs);
                if let Err(e) = self.assign(wasm, *scope, lhs, rhs_anal.ty, 0) {
                    wasm.error(self.error(e));
                    return Err(());
                }
            },
        };
        Ok(())
   }


    fn expr(
        &mut self,
        expr: &Expression,
        source: SourceRange,

        scope: &mut ScopeId,
        wasm: &mut WasmFunctionBuilder,
    ) -> AnalysisResult {
        match expr {
            Expression::Unit => {
                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },

            Expression::Literal(l) => {
                match l {
                    lexer::Literal::Integer(i) => {
                        wasm.i64_const(*i);
                        AnalysisResult::new(Type::I64, true)
                    },


                    lexer::Literal::Float(f) => {
                        wasm.f64_const(f.inner());
                        AnalysisResult::new(Type::F64, true)
                    },


                    lexer::Literal::String(v) => {
                        let str = self.string_map.get(*v);
                        let ptr = self.module_builder.add_string(str);
                        wasm.ptr_const(ptr);
                        
                        let ty = self.types.get(TypeId::STR);
                        let TypeSymbolKind::Struct(strct) = ty.kind()
                        else { unreachable!() };

                        let alloc = wasm.alloc_stack(ty.size());

                        {
                            let ptr = alloc.add(strct.fields[0].offset);
                            wasm.sptr_const(ptr);
                            wasm.i32_write();
                        }

                        {
                            let ptr = alloc.add(strct.fields[1].offset);
                            wasm.i64_const(str.len() as i64);

                            wasm.sptr_const(ptr);
                            wasm.i64_write();
                        }
                        
                        AnalysisResult::new(Type::STR, true)
                    },

                    lexer::Literal::Bool(v) => {
                        let ty = Type::BOOL;
                        let name = if *v { StringMap::TRUE } else { StringMap::FALSE };

                        let func = self.namespaces.get_type(ty);
                        let func = self.namespaces.get(func).get_func(name).unwrap();
                        let func = self.funcs.get(func);
                        
                        wasm.call(func.wasm_id);
                        AnalysisResult::new(Type::BOOL, true)
                    },
                }
            },


            Expression::Identifier(ident) => {
                let Some(variable) = self.scopes.get(*scope).get_var(*ident, &self.scopes)
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

                    (BinaryOperator::Eq, Type::I64) => wfunc!(i64_eq, Type::BOOL),
                    (BinaryOperator::Eq, Type::F64) => wfunc!(f64_eq, Type::BOOL),

                    (BinaryOperator::Eq, Type::Any) => todo!(),
                    (BinaryOperator::Eq, Type::Unit) => wfunc!(i64_eq, Type::BOOL),
                    (BinaryOperator::Eq, Type::Never) => todo!(),
                    (BinaryOperator::Eq, Type::Error) => todo!(),
                    (BinaryOperator::Eq, Type::Custom(v)) => {
                        let ty = self.types.get(v);
                        todo!()
                    },

                    (BinaryOperator::Ne, Type::I64) => wfunc!(i64_ne, Type::BOOL),
                    (BinaryOperator::Ne, Type::F64) => wfunc!(f64_ne, Type::BOOL),

                    (BinaryOperator::Ne, Type::Any) => todo!(),
                    (BinaryOperator::Ne, Type::Unit) => todo!(),
                    (BinaryOperator::Ne, Type::Never) => todo!(),
                    (BinaryOperator::Ne, Type::Error) => todo!(),
                    (BinaryOperator::Ne, Type::Custom(_)) => todo!(),

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
                let rhs_anal = self.node(scope, wasm, rhs);
                
                let mut type_check = || {
                    if rhs_anal.ty.eq_lit(Type::Error) {
                        return Err(())
                    }

                    if *operator == UnaryOperator::Not
                        && !rhs_anal.ty.eq_sem(Type::BOOL) {

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
                let cond = self.node(scope, wasm, &condition);

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
                        let body = slf.block(wasm, **scope, body);
                        let wty = body.ty.to_wasm_ty(&slf.types);
                        let local = wasm.local(wty);
                        (local, Some((body, wasm.offset())))
                    },
                    |(slf, scope), wasm| {
                        if let Some(else_block) = else_block {
                            return Some((slf.node(scope, wasm, else_block), wasm.offset()))
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
                fn match_mapping(
                    anal: &mut Analyzer<'_, '_, '_>,
                    mut scope: ScopeId,
                    wasm: &mut WasmFunctionBuilder,
                    id: LocalId,
                    taken_as_inout: bool,
                    value_range: SourceRange,

                    index: usize,
                    mappings: &[parser::nodes::MatchMapping<'_>]
                ) -> Option<(Type, SourceRange, LocalId)> {

                    let Some(mapping) = mappings.get(index) else {
                        wasm.block(|wasm, _| {
                        
                            wasm.local_get(id);
                            let mut str = format!("br_table {} ", mappings.len());

                            for i in (0..mappings.len()).rev() {
                                let _ = write!(str, "{} ", i);
                            }
                            wasm.raw(&str);

                        });

                        return None;
                    };
                    
                    let mut result = None;
                    wasm.block(|wasm, _| {
                        result = match_mapping(anal, scope, wasm, id, taken_as_inout, value_range, index + 1, mappings);

                        if mapping.is_inout() && !taken_as_inout {
                            wasm.error(anal.error(Error::InOutBindingWithoutInOutValue {
                                value_range, binding_range: mapping.binding_range() }))
                        }

                        let analysis = anal.node(&mut scope, wasm, mapping.node());
                        if let Some(result) = result {
                            if analysis.ty.eq_sem(result.0) {
                                wasm.local_set(result.2);
                            } else {
                                wasm.error(anal.error(Error::MatchBranchesDifferInReturnType {
                                    initial_source: result.1, initial_typ: result.0,
                                    branch_source: mapping.node().range(), branch_typ: analysis.ty
                                }));
                                wasm.pop();
                            }
                        } else {
                            result = Some((
                                analysis.ty, mapping.range(), 
                                wasm.local(analysis.ty.to_wasm_ty(&anal.types))
                            ));

                            wasm.local_set(result.unwrap().2);

                        }
                    });
                    
                    result
                }
                
                let anal = self.node(scope, wasm, value);
                if *taken_as_inout && !anal.is_mut {
                    wasm.error(self.error(Error::InOutValueIsntMut(value.range())));
                    return AnalysisResult::error();
                }

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

                let local = wasm.local(WasmType::I32);
                match sym {
                    TypeEnum::TaggedUnion(_) => wasm.i32_read(),
                    TypeEnum::Tag(_) => (),
                }
                wasm.local_set(local);

                let result = match_mapping(self, *scope, wasm, local, *taken_as_inout, value.range(), 0, mappings);
                if let Some(result) = result {
                    wasm.local_get(result.2);
                    AnalysisResult::new(result.0, true)
                } else {
                    wasm.unit();
                    AnalysisResult::new(Type::Unit, true)
                }
            },

            Expression::Block { block } => self.block(wasm, *scope, block),

            Expression::CreateStruct { data_type, fields } => {
                let ty = match self.convert_ty(*scope, *data_type) {
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

                    let node = self.node(scope, wasm, &val.2);
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
                let value = self.node(scope, wasm, val);

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
                    if f.name == *field_name {
                        wasm.i32_const(f.offset.try_into().unwrap());
                        wasm.i32_add();
                        wasm.read(f.ty.to_wasm_ty(&self.types));
                        return AnalysisResult::new(f.ty, value.is_mut);
                    }
                }

                wasm.error(self.error(Error::FieldDoesntExist {
                    source, field: *field_name, typ: value.ty }));
                AnalysisResult::error()
            },


            Expression::CallFunction { name, is_accessor, args } => {
                let mut scope = *scope;
                let pool = ArenaPool::tls_get_rec();
                let args = {
                    let mut vec = Vec::new_in(&*pool);
                    
                    for a in args.iter() {
                        let mut scope = scope;
                        let anal = self.node(&mut scope, wasm, &a.0);
                        vec.push((anal, a.0.range(), a.1))
                    }

                    vec
                };

                if *is_accessor {
                    let ty = args[0].0.ty;
                    let ns = self.namespaces.get_type(ty);
                    scope = self.scopes.push(
                        Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some()));
                }

                let scope = self.scopes.get(scope);
                let Some(func) = scope.get_func(*name, &self.scopes, &self.namespaces)
                else {
                    wasm.error(self.error(Error::FunctionNotFound { source, name: *name }));
                    return AnalysisResult::error();
                };

                let func = self.funcs.get(func);
                
                if func.args.len() != args.len() {
                    wasm.error(self.error(Error::FunctionArgsMismatch {
                        source, sig_len: func.args.len(), call_len: args.len() }));

                    return AnalysisResult::error();
                }

                let mut errored = false;
                for (sig_arg, call_arg) in func.args.iter().zip(args.iter()) {
                    if !sig_arg.2.eq_sem(call_arg.0.ty) {
                        errored = true;
                        wasm.error(self.error(Error::InvalidType {
                            source: call_arg.1, found: call_arg.0.ty, expected: sig_arg.2 }));
                    }

                    if sig_arg.1 && !call_arg.2 {
                        errored = true;
                        wasm.error(self.error(Error::InOutValueWithoutInOutBinding {
                            value_range: call_arg.1 }))
                    }
                }

                if errored {
                    return AnalysisResult::error();
                }

                wasm.call(func.wasm_id);

                AnalysisResult::new(func.ret, true)
            },


            Expression::WithinNamespace { namespace, namespace_source, action } => {
                let Some(ns) = self.scopes.get(*scope).get_ns(*namespace, &self.scopes, &mut self.namespaces)
                else {
                    wasm.error(self.error(Error::NamespaceNotFound 
                                          { source: *namespace_source, namespace: *namespace }));
                    return AnalysisResult::error();
                };

                let scope = Scope::new(ScopeKind::ImplicitNamespace(ns), scope.some());
                let mut scope = self.scopes.push(scope);
                self.node(&mut scope, wasm, action)
            },


            Expression::WithinTypeNamespace { namespace, action } => {
                let namespace = self.convert_ty(*scope, *namespace);
                let namespace = match namespace {
                    Ok(v) => v,
                    Err(e) => {
                        wasm.error(self.error(e));
                        return AnalysisResult::error();
                    },
                };

                let namespace = self.namespaces.get_type(namespace);
                let scope = Scope::new(ScopeKind::ImplicitNamespace(namespace), scope.some());
                let mut scope = self.scopes.push(scope);
                self.node(&mut scope, wasm, action)
            },

            
            Expression::Loop { body } => {
                wasm.do_loop(|wasm, id| {
                    let nscope = LoopScope::new(id);
                    let nscope = Scope::new(ScopeKind::Loop(nscope), scope.some());
                    let nscope = self.scopes.push(nscope);

                    self.block(wasm, nscope, body);
                });

                wasm.unit();
                AnalysisResult::new(Type::Unit, true)
            },


            Expression::Return(v) => {
                let value = self.node(scope, wasm, v);

                let func_return = {
                    let scope = self.scopes.get(*scope);
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
                    let scope = self.scopes.get(*scope);
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
                    let scope = self.scopes.get(*scope);
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


            Expression::CastAny { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
        }
    }

    ///
    /// This function expects:
    /// - A value on the stack with type `val_ty`
    ///
    fn assign(
        &mut self, 
        wasm: &mut WasmFunctionBuilder,
        scope: ScopeId, 
        node: &Node,
        val_ty: Type,
        depth: usize
    ) -> Result<Type, Error> {
        match node.kind() {
            NodeKind::Expression(Expression::Identifier(ident)) => {
                let Some(val) = self.scopes.get(scope).get_var(*ident, &self.scopes)
                else {
                    return Err(Error::VariableNotFound { name: *ident, source: node.range() });
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

            
            NodeKind::Expression(Expression::AccessField { val, field_name }) => {
                let ty = self.assign(wasm, scope, val, val_ty, depth + 1)?;

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
                    if sf.name == *field_name {
                        wasm.i32_const(sf.offset.try_into().unwrap());
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
                    source: node.range(), field: *field_name, typ: ty })
            }

            NodeKind::Error(_) => return Err(Error::Bypass),
            _ => return Err(Error::AssignIsNotLHSValue { source: node.range() }),
        }
    }
}


