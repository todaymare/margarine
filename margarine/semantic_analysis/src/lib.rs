pub mod scope;
pub mod errors;
pub mod namespace;
pub mod types;
pub mod funcs;

use common::{source::SourceRange, string_map::{StringMap, StringIndex}};
use ::errors::{ErrorId, SemaError};
use errors::Error;
use funcs::{FunctionMap, Function};
use namespace::{Namespace, NamespaceMap, NamespaceId};
use parser::{nodes::{Node, NodeKind, Expression, Declaration, BinaryOperator, UnaryOperator}, DataTypeKind, DataType};
use scope::{ScopeId, ScopeMap, Scope, ScopeKind, FunctionDefinitionScope, VariableScope};
use types::{ty::Type, ty_map::TypeMap, ty_sym::{TypeEnum, TypeSymbolKind}};
use wasm::{WasmModuleBuilder, WasmFunctionBuilder, WasmType, FunctionId, StackPointer};
use sti::{vec::Vec, keyed::KVec, prelude::Arena, packed_option::PackedOption, arena_pool::ArenaPool, hash::HashMap};

use crate::types::{ty_map::TypeId, ty_builder::{TypeBuilder, TypeBuilderData, PartialStructField}, ty_sym::{StructField, TypeStruct}};

#[derive(Debug)]
pub struct Analyzer<'me, 'out, 'str> {
    scopes: ScopeMap,
    namespaces: NamespaceMap,
    pub types: TypeMap<'out>,
    pub funcs: FunctionMap<'out>,
    type_to_namespace: HashMap<Type, NamespaceId>,
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
            DataTypeKind::Float => Type::I64,
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
            type_to_namespace: HashMap::new(),
            string_map,
            options_map: HashMap::new(),
            results_map: HashMap::new(),
        };

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
        
        let scope = self.scopes.push(scope);

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
            ty = self.node(scope, builder, n).ty;

            if id + 1 != nodes.len() {
                builder.pop();
            } 

        }

        if nodes.is_empty() { builder.i64_const(0); }

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

                    type_builder.set_struct_fields(ty, fields)
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
                
                let func = self.scopes.get(scope).get_func(*name, &self.scopes, &self.namespaces).unwrap();
                let func = self.funcs.get(func);
                let mut wasm = WasmFunctionBuilder::new(self.output, func.wasm_id);

                wasm.return_value(func.ret.to_wasm_ty(&self.types));

                let scope = Scope::new(
                    ScopeKind::FunctionDefinition(
                        FunctionDefinitionScope::new(func.ret)
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
                        AnalysisResult::new(Type::I64, true)
                    },


                    lexer::Literal::Float(f) => {
                        wasm.f64_const(f.inner());
                        AnalysisResult::new(Type::I64, true)
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
                            wasm.write_i32_to_stack(ptr);
                        }

                        {
                            let ptr = alloc.add(strct.fields[1].offset);
                            wasm.i64_const(str.len() as i64);
                            wasm.write_i64_to_stack(ptr);
                        }
                        
                        AnalysisResult::new(Type::STR, true)
                    },

                    lexer::Literal::Bool(v) => {
                        let ty = Type::BOOL;
                        let name = if *v { StringMap::TRUE } else { StringMap::FALSE };

                        let func = self.namespaces.get_type(ty).get_func(name).unwrap();
                        let func = self.funcs.get(func);
                        
                        wasm.call(func.wasm_id);
                        AnalysisResult::new(Type::BOOL, true)
                    },
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
                    (BinaryOperator::Eq, Type::Custom(_)) => todo!(),

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
                    wasm.i64_const(0);
                    return AnalysisResult::error();
                }

                match (operator, rhs_anal.ty) {
                    (UnaryOperator::Not, Type::Custom(x)) if x == TypeId::BOOL => {
                        wasm.i64_const(1);
                        wasm.i64_bw_xor();
                    },

                    (UnaryOperator::Neg, Type::I64) => {
                        // thanks wasm.
                        wasm.i64_const(-1);
                        wasm.i64_mul();
                    },

                    (UnaryOperator::Neg, Type::I64) => wasm.f64_neg(),

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

                let (local, l, r) = wasm.ite(
                    self,
                    |slf, wasm| {
                        let body = slf.block(wasm, scope, body);
                        let wty = body.ty.to_wasm_ty(&slf.types);
                        let local = wasm.local(wty);
                        (local, Some((body, wasm.offset())))
                    },
                    |slf, wasm| {
                        if let Some(else_block) = else_block {
                            return Some((slf.node(scope, wasm, else_block), wasm.offset()))
                        }

                        None
                    }
                );

                let l = l.unwrap();

                if r.is_none() && !l.0.ty.eq_sem(Type::Unit) {
                    wasm.error(self.error(Error::IfMissingElse { body: (body.range(), l.0.ty) }));
                    wasm.insert_pop(l.1);
                    return AnalysisResult::error();
                }
                
                if r.is_none() {
                    wasm.insert_pop(l.1);

                } else if r.is_some() && !l.0.ty.eq_sem(r.as_ref().unwrap().0.ty) {

                    wasm.error(self.error(Error::IfBodyAndElseMismatch {
                        body: (body.range(), l.0.ty),
                        else_block: (else_block.unwrap().range(), r.as_ref().unwrap().0.ty)
                    }));

                    let i = wasm.insert_pop(l.1);
                    wasm.insert_pop(r.unwrap().1 + i);
                    return AnalysisResult::error()
                } else {

                    let i = wasm.insert_local_set(l.1, local);
                    if let Some(r) = r {
                        wasm.insert_local_set(r.1 + i, local);
                    }
                }

                l.0
            },


            Expression::Match { value, taken_as_inout, mappings } => todo!(),

            Expression::Block { block } => self.block(wasm, scope, block),

            Expression::CreateStruct { data_type, fields } => {
                let ty = match self.convert_ty(scope, *data_type) {
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

                    wasm.memcpy(ptr, node.ty.to_wasm_ty(&self.types))
                }

                wasm.sptr_const(alloc);
                AnalysisResult::new(ty, true)
            },
            
            
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

