use std::collections::{hash_map::Entry, HashMap};

use common::{copy_slice_in, string_map::StringIndex};
use llvm_api::{builder::Builder, values::IsValue};
use parser::nodes::{decl::{Declaration, DeclarationNode}, expr::{BinaryOperator, Expression, ExpressionNode, UnaryOperator}, stmt::{Statement, StatementNode}, Node};
use sti::arena::Arena;

use crate::{errors::Error, funcs::{FunctionArgument, FunctionSymbol}, namespace::{Namespace, NamespaceId}, scope::{Scope, ScopeId, ScopeKind, VariableScope}, types::{Generic, GenericKind, Structure, StructureField, Type, TypeSymbol, TypeSymbolId, TypeSymbolKind}, AnalysisResult, Analyzer};

impl<'me, 'out, 'ast, 'str> Analyzer<'me, 'out, 'ast, 'str> {
    pub fn block(&mut self, builder: &mut Builder, path: StringIndex, scope: ScopeId, body: &[Node<'ast>]) -> AnalysisResult {
        let scope = scope;
        let mut namespace = Namespace::new(path);

        // Collect type names
        self.collect_names(builder, path, &mut namespace, body);

        // Update the current scope so the following functions
        // are able to see the namespace
        let namespace = self.namespaces.push(namespace);

        let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(namespace));
        let mut scope = self.scopes.push(scope);

        // Compute types & functions
        self.compute_types(builder, path, scope, namespace, body);

        // Analyze all nodes
        let mut last_node = None;
        for node in body.iter() {
            dbg!(&node);
            let eval = self.node(builder, path, &mut scope, namespace, *node);
            last_node = Some(eval);
        }

        // Finalise
        let result = match last_node {
            Some(v) => v,
            None    => AnalysisResult::new(Type::UNIT, builder.unit(), true),
        };

        result
    }


    pub fn collect_names(&mut self, builder: &mut Builder, path: StringIndex, ns: &mut Namespace, nodes: &[Node]) {
        for n in nodes {
            let Node::Declaration(decl) = n
            else { continue };

            match decl.kind() {
                | Declaration::Enum { name, header, .. } 
                | Declaration::Struct { name, header, .. } => {
                    if ns.get_ty_sym(name).is_some() {
                        self.error(builder, Error::NameIsAlreadyDefined {
                            source: header, name });
                        continue
                    }

                    let pend = self.types.pending();
                    ns.add_sym(name, pend);
                },

                Declaration::Module { name, header, body } => {
                    if ns.get_ns(name).is_some() {
                        self.error(builder, Error::NameIsAlreadyDefined {
                            source: header, name });
                        continue
                    }

                    let path = self.string_map.concat(path, name);

                    let mut module_ns = Namespace::new(path);
                    self.collect_names(builder, path, &mut module_ns, body);
                    ns.add_ns(name, self.namespaces.push(module_ns));
                }

                _ => (),
            }
        }
    }


    // `Self::collect_names` must be ran before this
    pub fn compute_types(&mut self, builder: &mut Builder, path: StringIndex,
                         scope: ScopeId, ns: NamespaceId, nodes: &[Node<'ast>]) {
        for n in nodes {
            let Node::Declaration(decl) = n
            else { continue };

            match decl.kind() {
                 Declaration::Struct { name, fields, generics, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let mut structure_fields = sti::vec::Vec::with_cap_in(self.output, fields.len());
                    let tsi = ns.get_ty_sym(name).unwrap();

                    for f in fields {
                        let sym = self.dt_to_gen(builder.ctx(), self.scopes.get(scope), f.1, generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(builder, v);
                                Generic::new(f.1.range(), GenericKind::Symbol {
                                    symbol: TypeSymbolId::ERROR, generics: &[] })
                            },
                        };

                        let field = StructureField::new(f.0.some(), sym);
                        structure_fields.push(field);
                    }

                    // finalise
                    let generics = copy_slice_in(self.output, generics);
                    let structure = Structure::new(false, structure_fields.leak());
                    let tsk = TypeSymbolKind::Structure(structure);
                    let sym_name = self.string_map.concat(path, name);
                    let sym = TypeSymbol::new(sym_name, generics, tsk);
                    self.types.add_sym(tsi, sym);
                },


                Declaration::Enum { .. } => todo!(),


                Declaration::Function { sig, body, .. } => {
                    let mut args = sti::vec::Vec::with_cap_in(self.output, sig.arguments.len());

                    for a in sig.arguments {
                        let sym = self.dt_to_gen(builder.ctx(), self.scopes.get(scope), a.data_type(), sig.generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(builder, v);
                                Generic::new(a.data_type().range(), GenericKind::Symbol {
                                    symbol: TypeSymbolId::ERROR, generics: &[] })
                            },
                        };

                        let arg = FunctionArgument::new(a.name(), sym, a.is_inout());
                        args.push(arg);
                    }


                    let ret = self.dt_to_gen(builder.ctx(), self.scopes.get(scope), sig.return_type, sig.generics);
                    let ret = match ret{
                        Ok(v) => v,
                        Err(v) => {
                            self.error(builder, v);
                            Generic::new(sig.return_type.range(), GenericKind::Symbol {
                                symbol: TypeSymbolId::ERROR, generics: &[] })
                        },
                    };


                    // Finalise
                    let generics = copy_slice_in(self.output, sig.generics);
                    let sym_name = self.string_map.concat(path, sig.name);
                    let func = FunctionSymbol::new(sym_name, ret, args.leak(), generics, body);
                    let id = self.funcs.push(func);
                    let ns = self.namespaces.get_ns_mut(ns);
                    ns.add_func(sig.name, id);

                }


                Declaration::Module { name, body, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let Some(module_ns) = ns.get_ns(name)
                    else { continue };

                    let scope = self.scopes.push(self.scopes.get(scope));
                    let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(module_ns));
                    let scope = self.scopes.push(scope);

                    let path = self.namespaces.get_ns(module_ns).path;
                    self.compute_types(builder, path, scope, module_ns, body);

                }

                _ => (),
            }
        }
    }


    pub fn node(&mut self, builder: &mut Builder, path: StringIndex, scope: &mut ScopeId, ns: NamespaceId, node: Node<'ast>) -> AnalysisResult {
        match node {
            Node::Declaration(decl) => {
                self.decl(builder, scope, ns, decl);
                self.empty_error(builder)
            },

            Node::Statement(stmt) => {
                self.stmt(builder, path, scope, stmt);
                self.empty_error(builder)
            },

            Node::Expression(expr) => self.expr(builder, path, *scope, expr),

            Node::Attribute(_) => todo!(),

            Node::Error(e) => {
                self.place_error(builder, e.id());
                self.empty_error(builder)
            },
        }
    }


    pub fn decl(&mut self, builder: &mut Builder, scope: &mut ScopeId, ns: NamespaceId, decl: DeclarationNode<'ast>) {
        let decl = decl.kind();
        match decl {
            Declaration::Struct { .. } => (),
            Declaration::Enum { .. } => (),

            
            Declaration::Function { sig, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(func) = ns.get_func(sig.name)
                else { return };

                let pool = Arena::tls_get_rec();

                // we need a scope that'd fake the generics
                let generics = self.funcs.get_sym(func).generics();
                let generics = {
                    let mut vec = sti::vec::Vec::new_in(&*pool);
                    for gen in generics {
                        let ty = self.types.pending();
                        self.types.add_sym(ty, TypeSymbol::new(*gen, &[], TypeSymbolKind::Structure(Structure::new(false, &[]))));
                        vec.push(self.get_ty(builder.ctx(), ty, &[]));
                    }

                    vec
                };
                
                // this function will automatically generate & tyck the function
                self.get_func(false, builder.ctx(), *scope, func, &*generics);
            },


            Declaration::Impl { .. } => todo!(),
            Declaration::Using { .. } => todo!(),

            Declaration::Module { name, body, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(module_ns) = ns.get_ns(name)
                else { return };
                
                let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(module_ns));
                let mut scope = self.scopes.push(scope);

                let path = self.namespaces.get_ns(module_ns).path;
                for n in body {
                    self.node(builder, path, &mut scope, module_ns, *n);
                }
            },

            Declaration::Extern { .. } => todo!(),
        }
    }


    pub fn stmt(&mut self, builder: &mut Builder, path: StringIndex, scope: &mut ScopeId, stmt: StatementNode<'ast>) {
        let source = stmt.range();
        let stmt = stmt.kind();
        match stmt {
            Statement::Variable { name, hint, is_mut, rhs } => {
                let rhs_anal = self.expr(builder, path, *scope, rhs);
                
                let place_dummy = |slf: &mut Analyzer<'_, 'out, '_, '_>, builder: &mut Builder<'_>, scope: &mut ScopeId| {
                    let ty = slf.types.get_ty_val(Type::ERROR).llvm_ty();
                    let local = builder.local(ty);
                    let vs = VariableScope::new(name, Type::ERROR, is_mut, local);
                    *scope = slf.scopes.push(Scope::new(scope.some(), ScopeKind::VariableScope(vs)));
                };

                // Validation
                if rhs_anal.ty.eq(Type::ERROR, &self.types) {
                    place_dummy(self, builder, scope);
                    return;
                }

                if let Some(hint) = hint {
                    let hint = match self.dt_to_ty(builder.ctx(), *scope, hint) {
                        Ok(v)  => v,
                        Err(v) => {
                            place_dummy(self, builder, scope);
                            self.error(builder, v);
                            return
                        },
                    };

                    if rhs_anal.ty.eq(Type::NEVER, &self.types) && rhs_anal.ty.eq(hint, &self.types) {
                        place_dummy(self, builder, scope);
                        self.error(builder, Error::VariableValueAndHintDiffer {
                            value_type: rhs_anal.ty, hint_type: hint, source });
                        return
                    }
                }

                // finalise
                let ty = self.types.get_ty_val(rhs_anal.ty).llvm_ty();
                let local = builder.local(ty);
                builder.local_set(local, rhs_anal.value);

                let vs = VariableScope::new(name, rhs_anal.ty, is_mut, local);
                *scope = self.scopes.push(Scope::new(scope.some(),
                                          ScopeKind::VariableScope(vs)));
            },


            Statement::VariableTuple { .. } => todo!(),


            Statement::UpdateValue { .. } => todo!(),


            Statement::ForLoop { .. } => todo!(),
        }
    }


    pub fn expr(&mut self, builder: &mut Builder, path: StringIndex, scope: ScopeId, expr: ExpressionNode<'ast>) -> AnalysisResult {
        let source = expr.range();
        let expr = expr.kind();
        match expr {
            Expression::Unit => AnalysisResult::new(Type::UNIT, builder.unit(), true),


            Expression::Literal(lit) => {
                match lit {
                    lexer::Literal::Integer(int) => {
                        let ty  = self.types.get_ty_val(Type::I64).llvm_ty();
                        let int = builder.constant(ty.as_signed_int(), int);
                        AnalysisResult::new(Type::I64, int.value(), true)
                    },


                    lexer::Literal::Float(float) => {
                        let ty    = self.types.get_ty_val(Type::F64).llvm_ty();
                        let float = builder.constant(ty.as_f64(), float.inner());
                        AnalysisResult::new(Type::F64, float.value(), true)
                    },


                    // TODO: Need Rc to work first
                    lexer::Literal::String(_) => todo!(),


                    // assumes 1 is true and 0 is false
                    lexer::Literal::Bool(b) => {
                        let ty   = self.types.get_ty_val(Type::BOOL).llvm_ty();
                        let bool = builder.constant(ty.as_signed_int(), if b { 1 } else { 0 });
                        AnalysisResult::new(Type::BOOL, bool.value(), true)
                    },
                }
            },


            Expression::Identifier(ident) => {
                let Some(variable) = self.scopes.get(scope).find_var(ident, &self.scopes)
                else { return self.error(builder, Error::VariableNotFound { name: ident, source }) };

                if variable.ty().eq(Type::ERROR, &self.types) { return self.empty_error(builder) }

                let local = variable.local();
                let value = builder.local_get(local);

                AnalysisResult::new(variable.ty(), value, variable.is_mut())
            },


            Expression::Deref(_) => todo!(),


            Expression::Range { .. } => todo!(),


            Expression::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.expr(builder, path, scope, *lhs);
                let rhs_anal = self.expr(builder, path, scope, *rhs);

                if lhs_anal.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if lhs_anal.ty.eq(Type::NEVER, &self.types) { return self.never(builder) }
                if rhs_anal.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if rhs_anal.ty.eq(Type::NEVER, &self.types) { return self.never(builder) }

                let validate = || {
                    lhs_anal.ty.eq(rhs_anal.ty, &self.types)
                    && if operator.is_arith() { lhs_anal.ty.supports_arith() } else { true }
                    && if operator.is_bw() { lhs_anal.ty.supports_bw() } else { true }
                    && if operator.is_ocomp() { lhs_anal.ty.supports_ord() } else { true }
                    && if operator.is_ecomp() { lhs_anal.ty.supports_eq() } else { true }
                };


                if !validate() {
                    return self.error(builder, Error::InvalidBinaryOp {
                        operator, lhs: lhs_anal.ty, rhs: rhs_anal.ty, source });
                }

                macro_rules! op {
                    ($f: ident, $as: ident) => {
                        builder.$f(lhs_anal.value.$as(), rhs_anal.value.$as()).value()
                    };
                }

                let value = match operator {
                    BinaryOperator::Add if lhs_anal.ty.is_sint() => op!(add, as_signed_int),
                    BinaryOperator::Add if lhs_anal.ty.is_uint() => op!(add, as_unsigned_int),
                    BinaryOperator::Add if lhs_anal.ty.is_f32()  => op!(add, as_f32),
                    BinaryOperator::Add if lhs_anal.ty.is_f64()  => op!(add, as_f64),

                    BinaryOperator::Sub if lhs_anal.ty.is_sint() => op!(sub, as_signed_int),
                    BinaryOperator::Sub if lhs_anal.ty.is_uint() => op!(sub, as_unsigned_int),
                    BinaryOperator::Sub if lhs_anal.ty.is_f32()  => op!(sub, as_f32),
                    BinaryOperator::Sub if lhs_anal.ty.is_f64()  => op!(sub, as_f64),

                    BinaryOperator::Mul if lhs_anal.ty.is_sint() => op!(mul, as_signed_int),
                    BinaryOperator::Mul if lhs_anal.ty.is_uint() => op!(mul, as_unsigned_int),
                    BinaryOperator::Mul if lhs_anal.ty.is_f32()  => op!(mul, as_f32),
                    BinaryOperator::Mul if lhs_anal.ty.is_f64()  => op!(mul, as_f64),

                    BinaryOperator::Div if lhs_anal.ty.is_sint() => op!(div, as_signed_int),
                    BinaryOperator::Div if lhs_anal.ty.is_uint() => op!(div, as_unsigned_int),
                    BinaryOperator::Div if lhs_anal.ty.is_f32()  => op!(div, as_f32),
                    BinaryOperator::Div if lhs_anal.ty.is_f64()  => op!(div, as_f64),

                    BinaryOperator::Rem if lhs_anal.ty.is_sint() => op!(rem, as_signed_int),
                    BinaryOperator::Rem if lhs_anal.ty.is_uint() => op!(rem, as_unsigned_int),
                    BinaryOperator::Rem if lhs_anal.ty.is_f32()  => op!(rem, as_f32),
                    BinaryOperator::Rem if lhs_anal.ty.is_f64()  => op!(rem, as_f64),

                    BinaryOperator::BitwiseAnd if lhs_anal.ty.is_sint() => op!(and, as_signed_int),
                    BinaryOperator::BitwiseAnd if lhs_anal.ty.is_uint() => op!(and, as_signed_int),

                    BinaryOperator::BitwiseOr if lhs_anal.ty.is_sint() => op!(or, as_signed_int),
                    BinaryOperator::BitwiseOr if lhs_anal.ty.is_uint() => op!(or, as_signed_int),

                    BinaryOperator::BitwiseXor if lhs_anal.ty.is_sint() => op!(xor, as_signed_int),
                    BinaryOperator::BitwiseXor if lhs_anal.ty.is_uint() => op!(xor, as_signed_int),

                    BinaryOperator::BitshiftLeft if lhs_anal.ty.is_sint() => op!(shl, as_signed_int),
                    BinaryOperator::BitshiftLeft if lhs_anal.ty.is_uint() => op!(shl, as_signed_int),

                    BinaryOperator::BitshiftRight if lhs_anal.ty.is_sint() => op!(shr, as_signed_int),
                    BinaryOperator::BitshiftRight if lhs_anal.ty.is_uint() => op!(shr, as_signed_int),

                    BinaryOperator::Eq if lhs_anal.ty.is_sint() => op!(eq, as_signed_int),
                    BinaryOperator::Eq if lhs_anal.ty.is_uint() => op!(eq, as_unsigned_int),
                    BinaryOperator::Eq if lhs_anal.ty.is_f32()  => op!(eq, as_f32),
                    BinaryOperator::Eq if lhs_anal.ty.is_f64()  => op!(eq, as_f64),

                    BinaryOperator::Ne if lhs_anal.ty.is_sint() => op!(ne, as_signed_int),
                    BinaryOperator::Ne if lhs_anal.ty.is_uint() => op!(ne, as_unsigned_int),
                    BinaryOperator::Ne if lhs_anal.ty.is_f32()  => op!(ne, as_f32),
                    BinaryOperator::Ne if lhs_anal.ty.is_f64()  => op!(ne, as_f64),

                    BinaryOperator::Gt if lhs_anal.ty.is_sint() => op!(gt, as_signed_int),
                    BinaryOperator::Gt if lhs_anal.ty.is_uint() => op!(gt, as_unsigned_int),
                    BinaryOperator::Gt if lhs_anal.ty.is_f32()  => op!(gt, as_f32),
                    BinaryOperator::Gt if lhs_anal.ty.is_f64()  => op!(gt, as_f64),

                    BinaryOperator::Ge if lhs_anal.ty.is_sint() => op!(eq, as_signed_int),
                    BinaryOperator::Ge if lhs_anal.ty.is_uint() => op!(eq, as_unsigned_int),
                    BinaryOperator::Ge if lhs_anal.ty.is_f32()  => op!(eq, as_f32),
                    BinaryOperator::Ge if lhs_anal.ty.is_f64()  => op!(eq, as_f64),

                    BinaryOperator::Lt if lhs_anal.ty.is_sint() => op!(lt, as_signed_int),
                    BinaryOperator::Lt if lhs_anal.ty.is_uint() => op!(lt, as_unsigned_int),
                    BinaryOperator::Lt if lhs_anal.ty.is_f32()  => op!(lt, as_f32),
                    BinaryOperator::Lt if lhs_anal.ty.is_f64()  => op!(lt, as_f64),

                    BinaryOperator::Le if lhs_anal.ty.is_sint() => op!(le, as_signed_int),
                    BinaryOperator::Le if lhs_anal.ty.is_uint() => op!(le, as_unsigned_int),
                    BinaryOperator::Le if lhs_anal.ty.is_f32()  => op!(le, as_f32),
                    BinaryOperator::Le if lhs_anal.ty.is_f64()  => op!(le, as_f64),

                    _ => unreachable!(),
                };

                
                AnalysisResult::new(lhs_anal.ty, value, true)
            },


            Expression::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(builder, path, scope, *rhs);
                if rhs_anal.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if rhs_anal.ty.eq(Type::NEVER, &self.types) { return self.never(builder) }

                let value = match operator {
                    UnaryOperator::Not if rhs_anal.ty.eq(Type::BOOL, &self.types) => builder.bool_not(rhs_anal.value.as_bool()).value(),
                    UnaryOperator::Neg if rhs_anal.ty.is_sint() => {
                        let ty = rhs_anal.value.ty();
                        let neg_one = builder.constant(ty.as_signed_int(), -1);
                        builder.mul(rhs_anal.value.as_signed_int(), neg_one).value()
                    },

                    _ => return self.error(builder,
                                           Error::InvalidUnaryOp { operator, rhs: rhs_anal.ty, source })
                };

                AnalysisResult::new(rhs_anal.ty, value, true)
            },


            Expression::If { condition, body, else_block } => {
                let mut cond = self.expr(builder, path, scope, *condition);
                if cond.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if cond.ty.eq(Type::NEVER, &self.types) {
                    let bool = builder.ctx().bool();
                    cond.value = builder.constant(bool, false).value();

                } else if cond.ty.eq(Type::BOOL, &self.types) {
                    return self.error(builder, Error::InvalidType {
                        source: condition.range(), found: cond.ty, expected: Type::BOOL })
                }

                let mut value = None;

                builder.ite(&mut (&mut *self, &mut value), cond.value.as_bool(),
                |builder, (slf, value)| {
                    let anal = slf.expr(builder, path, scope, *body);
                    if anal.ty.eq(Type::ERROR, &slf.types) { return }

                    let ty = slf.types.get_ty_val(anal.ty);
                    let local = builder.local(ty.llvm_ty());
                    **value = Some((anal.ty, local));

                    builder.local_set(local, anal.value);
                },


                |builder, (slf, value)| {
                    let Some(el) = else_block
                    else { return };

                    let el_anal = slf.expr(builder, path, scope, *el);
                    let Some((anal, local)) = value
                    else {
                        if el_anal.ty.eq(Type::ERROR, &slf.types) { return }

                        let ty = slf.types.get_ty_val(el_anal.ty);
                        let local = builder.local(ty.llvm_ty());                   
                        **value = Some((el_anal.ty, local));

                        builder.local_set(local, el_anal.value);
                        return
                    };

                    if el_anal.ty.ne(*anal, &slf.types) {
                        slf.error(builder, Error::IfBodyAndElseMismatch {
                            body: (body.range(), *anal), else_block: (el.range(), el_anal.ty) });
                        return
                    }

                    builder.local_set(*local, el_anal.value);
                });


                let Some(value) = value
                else { return self.empty_error(builder) };

                if value.0.ne(Type::UNIT, &self.types) && else_block.is_none() {
                    return self.error(builder, Error::IfMissingElse {
                        body: (body.range(), value.0) })
                }

                let val = builder.local_get(value.1);
                AnalysisResult::new(value.0, val, true)
            },


            Expression::Match { .. } => todo!(),


            Expression::Block { block } => self.block(builder, path, scope, &*block),


            Expression::CreateStruct { data_type, fields  } => {
                todo!()
            },


            Expression::AccessField { val, field_name } => {
                let ty = self.expr(builder, path, scope, *val);
                if ty.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if ty.ty.eq(Type::NEVER, &self.types) { return self.never(builder) }

                let ty_val = self.types.get_ty_val(ty.ty);
                let sym = self.types.get_sym(ty_val.symbol());

                match sym.kind() {
                    TypeSymbolKind::Structure(strct) => {
                        let Some(field) = strct.fields.iter().find(|x| x.name == field_name.some())
                        else { return self.error(builder, Error::FieldDoesntExist 
                                                    { source, field: field_name, typ: ty.ty }) };


                        todo!()
                    },
                    TypeSymbolKind::Enum(_) => todo!(),

                    _ => self.error(builder, Error::FieldAccessOnNonEnumOrStruct 
                                                    { source, typ: ty.ty })
                };

                todo!()
            },


            Expression::CallFunction { name, is_accessor, args } => todo!(),
            Expression::WithinNamespace { namespace, namespace_source, action } => todo!(),
            Expression::WithinTypeNamespace { namespace, action } => todo!(),


            Expression::Loop { body } => {
                builder.loop_indefinitely(
                |builder, l| {
                    let scope = Scope::new(scope.some(), ScopeKind::Loop(l));
                    let scope = self.scopes.push(scope);
                    self.block(builder, path, scope, &*body);
                });

                AnalysisResult::new(Type::UNIT, builder.unit(), true)
            },


            Expression::Return(ret) => {
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return self.error(builder, Error::ReturnOutsideOfAFunction { source }) };

                let ret_anal = self.expr(builder, path, scope, *ret);
                if ret_anal.ty.eq(Type::ERROR, &self.types) { return self.empty_error(builder) }
                if ret_anal.ty.eq(Type::NEVER, &self.types) { return self.never(builder) }

                if ret_anal.ty.ne(func.ret, &self.types) {
                    return self.error(builder, Error::ReturnAndFuncTypDiffer {
                        source, func_source: func.ret_source,
                        typ: ret_anal.ty, func_typ: func.ret })
                }

                AnalysisResult::new(Type::NEVER, builder.unit(), true)
            },


            Expression::Continue => {
                let Some(l) = self.scopes.get(scope).find_loop(&self.scopes)
                else { return self.error(builder, Error::ContinueOutsideOfLoop(source)) };

                builder.loop_continue(l);
                AnalysisResult::new(Type::NEVER, builder.unit(), true)
            },


            Expression::Break => {
                let Some(l) = self.scopes.get(scope).find_loop(&self.scopes)
                else { return self.error(builder, Error::ContinueOutsideOfLoop(source)) };

                builder.loop_break(l);
                AnalysisResult::new(Type::NEVER, builder.unit(), true)
            },


            Expression::Tuple(v) => {
                todo!();
            },


            Expression::AsCast { lhs, data_type } => todo!(),
            Expression::Unwrap(_) => todo!(),
            Expression::OrReturn(_) => todo!(),
        }
    }


    fn infer(&mut self, ty: TypeSymbolId, fields: &[Type]) -> HashMap<StringIndex, Option<Type>> {
        let sym = self.types.get_sym(ty);
        let mut generics : HashMap<StringIndex, Option<Type>> = HashMap::with_capacity(sym.generics().len());
        for g in sym.generics() {
            generics.insert(*g, None);
        }

        match sym.kind() {
            TypeSymbolKind::Structure(s) => {
                fn i(slf: &mut Analyzer<'_, '_, '_, '_>,
                     map: &mut HashMap<StringIndex, Option<Type>>,
                     generics: &[Generic], tys: &[Type]) {
                    for (g, f) in generics.iter().zip(tys.iter()) {
                        match g.kind {
                            GenericKind::Generic(v) => {
                                let Entry::Occupied(mut value) = map.entry(v)
                                else { return todo!() };

                                let ty = value.get_mut();
                                if ty.is_none() { *ty = Some(*f) }
                                else if let Some(ty) = ty {
                                    if ty.ne(*f, &slf.types) {
                                        todo!()
                                    }
                                }
                            },


                            GenericKind::Symbol { symbol, generics } => {
                                let ty_val = slf.types.get_ty_val(*f);
                                i(slf, map, generics, tys)
                            },
                        }
                    }
                }
            },
            TypeSymbolKind::Enum(_) => todo!(),
            TypeSymbolKind::BuiltIn => todo!(),
        };

        generics
    }
}
