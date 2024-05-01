use common::{copy_slice_in, string_map::StringIndex};
use llvm_api::values;
use parser::nodes::{decl::{Decl, DeclId}, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId};
use sti::arena::Arena;

use crate::{errors::Error, funcs::{FunctionArgument, FunctionSymbol}, namespace::{Namespace, NamespaceId}, scope::{GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, types::{Generic, GenericKind, Symbol, SymbolId, SymbolKind, Type}, AnalysisResult, TyChecker};

impl<'me, 'out, 'ast, 'str> TyChecker<'me, 'out, 'ast, 'str> {
    pub fn block(&mut self, path: StringIndex, scope: ScopeId, body: &[NodeId]) -> AnalysisResult {
        let scope = scope;
        let mut namespace = Namespace::new(path);

        // Collect type names
        self.collect_names(path, &mut namespace, body);

        // Update the current scope so the following functions
        // are able to see the namespace
        let namespace = self.namespaces.push(namespace);

        let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(namespace));
        let mut scope = self.scopes.push(scope);

        // Compute types & functions
        self.compute_types(path, scope, namespace, body);

        // Analyze all nodes
        let mut last_node = None;
        for node in body.iter() {
            let eval = self.node(path, &mut scope, namespace, *node);
            last_node = Some(eval);
        }

        // Finalise
        let result = match last_node {
            Some(v) => v,
            None    => AnalysisResult::new(Type::UNIT, true),
        };

        result
    }


    pub fn collect_names(&mut self, path: StringIndex, ns: &mut Namespace, nodes: &[NodeId]) {
        for n in nodes {
            let NodeId::Decl(decl) = n
            else { continue };

            let decl = self.ast.decl(*decl);
            match decl {
                | Decl::Enum { name, header, .. } 
                | Decl::Struct { name, header, .. } => {
                    if ns.get_ty_sym(name).is_some() {
                        self.error(*n, Error::NameIsAlreadyDefined {
                            source: header, name });
                        continue
                    }

                    let pend = self.types.pending();
                    ns.add_sym(name, pend);
                },

                Decl::Module { name, header, body } => {
                    if ns.get_ns(name).is_some() {
                        self.error(*n, Error::NameIsAlreadyDefined {
                            source: header, name });
                        continue
                    }

                    let path = self.string_map.concat(path, name);

                    let mut module_ns = Namespace::new(path);
                    self.collect_names(path, &mut module_ns, &*body);
                    ns.add_ns(name, self.namespaces.push(module_ns));
                }

                _ => (),
            }
        }
    }


    // `Self::collect_names` must be ran before this
    pub fn compute_types(&mut self, path: StringIndex, scope: ScopeId,
                         ns: NamespaceId, nodes: &[NodeId]) {
        for n in nodes {
            let NodeId::Decl(decl) = n
            else { continue };

            let decl = self.ast.decl(*decl);
            match decl {
                 Decl::Struct { name, fields, generics, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let mut structure_fields = sti::vec::Vec::with_cap_in(self.output, fields.len());
                    let tsi = ns.get_ty_sym(name).unwrap();

                    for f in fields {
                        let sym = self.dt_to_gen(self.scopes.get(scope), f.1, generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(*n, v);
                                Generic::new(f.1.range(), GenericKind::ERROR)
                            },
                        };

                        let field = (f.0.some(), sym);
                        structure_fields.push(field);
                    }

                    // finalise
                    let generics = copy_slice_in(self.output, generics);
                    let sym_name = self.string_map.concat(path, name);
                    let sym = Symbol::new(sym_name, generics,
                                          structure_fields.leak(), SymbolKind::Struct);
                    self.types.add_sym(tsi, sym);
                },


                Decl::Enum { .. } => todo!(),


                Decl::Function { sig, body, .. } => {
                    let mut args = sti::vec::Vec::with_cap_in(self.output, sig.arguments.len());

                    for a in sig.arguments {
                        let sym = self.dt_to_gen(self.scopes.get(scope), a.data_type(), sig.generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(*n, v);
                                Generic::new(a.data_type().range(), GenericKind::ERROR)
                            },
                        };

                        let arg = FunctionArgument::new(a.name(), sym, a.is_inout());
                        args.push(arg);
                    }


                    let ret = self.dt_to_gen( self.scopes.get(scope), sig.return_type, sig.generics);
                    let ret = match ret{
                        Ok(v) => v,
                        Err(v) => {
                            self.error(*n, v);
                            Generic::new(sig.return_type.range(), GenericKind::ERROR)
                        },
                    };


                    // Finalise
                    let generics = copy_slice_in(self.output, sig.generics);
                    let sym_name = self.string_map.concat(path, sig.name);
                    let func = FunctionSymbol::new(sym_name, ret, args.leak(), generics, body, sig.source);
                    let id = self.funcs.push(func);
                    let ns = self.namespaces.get_ns_mut(ns);
                    ns.add_func(sig.name, id);
                }


                Decl::Module { name, body, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let Some(module_ns) = ns.get_ns(name)
                    else { continue };

                    let scope = self.scopes.push(self.scopes.get(scope));
                    let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(module_ns));
                    let scope = self.scopes.push(scope);

                    let path = self.namespaces.get_ns(module_ns).path;
                    self.compute_types(path, scope, module_ns, &*body);

                }

                _ => (),
            }
        }
    }


    pub fn node(&mut self, path: StringIndex,
                scope: &mut ScopeId, ns: NamespaceId, node: NodeId) -> AnalysisResult {
        match node {
            NodeId::Decl(decl) => {
                self.decl(scope, ns, decl);
                AnalysisResult::new(Type::UNIT, true)
            },

            NodeId::Stmt(stmt) => {
                self.stmt(path, scope, stmt);
                AnalysisResult::new(Type::UNIT, true)
            },

            NodeId::Expr(expr) => self.expr(path, *scope, expr),

            NodeId::Err(_) => {
                AnalysisResult::new(Type::ERROR, true)
            },
        }
    }


    pub fn decl(&mut self, scope: &mut ScopeId, ns: NamespaceId, id: DeclId) {
        let decl = self.ast.decl(id);
        match decl {
            Decl::Struct { .. } => (),
            Decl::Enum { .. } => (),

            
            Decl::Function { sig, body, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(func) = ns.get_func(sig.name)
                else { return };

                // we need a scope that'd fake the generics
                let generics = self.funcs.get_sym(func).generics();
                let generics = {
                    let mut vec = sti::vec::Vec::new_in(&*self.output);
                    for gen in generics {
                        let ty = self.types.pending();
                        self.types.add_sym(ty, Symbol::new(*gen, &[], &[], SymbolKind::Struct));
                        vec.push((*gen, self.get_ty(ty, &[])));
                    }

                    vec
                };
                
                // fake args
                let generics = generics.leak();
                let gscope = GenericsScope::new(generics);
                let mut scope = Scope::new(scope.some(), ScopeKind::Generics(gscope));

                for a in self.funcs.get_sym(func).args() {
                    let ty = self.gen_to_ty(a.symbol, &generics);
                    let ty = match ty {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(id, v);
                            Type::ERROR
                        }
                    };

                    let vs = VariableScope::new(a.name, ty, a.inout);
                    scope = Scope::new(self.scopes.push(scope).some(), ScopeKind::VariableScope(vs))
                }

                let ret = self.gen_to_ty(self.funcs.get_sym(func).ret(), &generics);
                let ret = match ret {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(id, v);
                        Type::ERROR
                    }
                };

                let scope = self.scopes.push(scope);

                let anal = self.block(self.funcs.get_sym(func).name, scope, &*body);

                let anal_sym = match anal.ty.sym(&mut self.types) {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(id, v);
                        return;
                    },
                };

                let ret_sym = match ret.sym(&mut self.types) {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(id, v);
                        return;
                    },
                };

                if anal_sym != ret_sym {
                    self.error(id, Error::FunctionBodyAndReturnMismatch {
                        header: sig.source, item: body.range(),
                        return_type: ret, body_type: anal.ty });
                }
            },


            Decl::Impl { .. } => todo!(),
            Decl::Using { .. } => todo!(),

            Decl::Module { name, body, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(module_ns) = ns.get_ns(name)
                else { return };
                
                let scope = Scope::new(scope.some(), ScopeKind::ImplicitNamespace(module_ns));
                let mut scope = self.scopes.push(scope);

                let path = self.namespaces.get_ns(module_ns).path;
                for n in body.iter() {
                    self.node(path, &mut scope, module_ns, *n);
                }
            },

            Decl::Extern { .. } => todo!(),
        }
    }


    pub fn stmt(&mut self, path: StringIndex,
                scope: &mut ScopeId, id: StmtId) {
        let source = self.ast.range(id);
        let stmt = self.ast.stmt(id);
        match stmt {
            Stmt::Variable { name, hint, is_mut, rhs } => {
                let rhs_anal = self.expr(path, *scope, rhs);
                
                let place_dummy = |slf: &mut TyChecker<'_, 'out, '_, '_>, scope: &mut ScopeId| {
                    let vs = VariableScope::new(name, Type::ERROR, is_mut);
                    *scope = slf.scopes.push(Scope::new(scope.some(), ScopeKind::VariableScope(vs)));
                };

                // Validation
                if rhs_anal.ty.eq(&mut self.types, Type::ERROR) {
                    place_dummy(self, scope);
                    return;
                }

                if let Some(hint) = hint {
                    let hint = match self.dt_to_ty(*scope, hint) {
                        Ok(v)  => v,
                        Err(v) => {
                            place_dummy(self, scope);
                            self.error(id, v);
                            return
                        },
                    };

                    if rhs_anal.ty.eq(&mut self.types, Type::NEVER) && rhs_anal.ty.eq(&mut self.types, hint) {
                        place_dummy(self, scope);
                        self.error(id, Error::VariableValueAndHintDiffer {
                            value_type: rhs_anal.ty, hint_type: hint, source });
                        return
                    }
                }

                // finalise
                let vs = VariableScope::new(name, rhs_anal.ty, is_mut);
                *scope = self.scopes.push(Scope::new(scope.some(),
                                          ScopeKind::VariableScope(vs)));
            },


            Stmt::VariableTuple { .. } => todo!(),


            Stmt::UpdateValue { .. } => todo!(),


            Stmt::ForLoop { .. } => todo!(),
        }
    }


    pub fn expr(&mut self, path: StringIndex, scope: ScopeId, id: ExprId) -> AnalysisResult {
        let source = self.ast.range(id);
        let expr = self.ast.expr(id);
        let result = (|| Ok(match expr {
            Expr::Unit => AnalysisResult::new(Type::UNIT, true),


            Expr::Literal(lit) => {
                match lit {
                    lexer::Literal::Integer(_) => {
                        AnalysisResult::new(Type::I64, true)
                    },


                    lexer::Literal::Float(_) => {
                        AnalysisResult::new(Type::F64, true)
                    },


                    // TODO: Need Rc to work first
                    lexer::Literal::String(_) => todo!(),


                    // assumes 1 is true and 0 is false
                    lexer::Literal::Bool(_) => {
                        AnalysisResult::new(Type::BOOL, true)
                    },
                }
            },


            Expr::Identifier(ident) => {
                let Some(variable) = self.scopes.get(scope).find_var(ident, &self.scopes)
                else { return Err(Error::VariableNotFound { name: ident, source }) };
                AnalysisResult::new(variable.ty(), variable.is_mut())
            },


            Expr::Deref(_) => todo!(),


            Expr::Range { .. } => todo!(),


            Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.expr(path, scope, lhs);
                let rhs_anal = self.expr(path, scope, rhs);

                let lhs_sym = lhs_anal.ty.sym(&mut self.types)?;

                if lhs_sym == SymbolId::ERROR { return Ok(AnalysisResult::error()) }
                if lhs_sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                let rhs_sym = rhs_anal.ty.sym(&mut self.types)?;

                if rhs_sym == SymbolId::ERROR { return Ok(AnalysisResult::error()) }
                if rhs_sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                let mut validate = || {
                    if !lhs_anal.ty.eq(&mut self.types, rhs_anal.ty) { return Ok(false) }
                    let sym = match lhs_anal.ty.sym(&mut self.types) {
                        Ok(v) => v,
                        Err(v) => return Err(v),
                    };

                    Ok(if operator.is_arith() { sym.supports_arith() } else { true }
                    && if operator.is_bw() { sym.supports_bw() } else { true }
                    && if operator.is_ocomp() { sym.supports_ord() } else { true }
                    && if operator.is_ecomp() { sym.supports_eq() } else { true })
                };


                let validate = validate()?;

                if !validate {
                    return Err(Error::InvalidBinaryOp {
                        operator, lhs: lhs_anal.ty, rhs: rhs_anal.ty, source });
                }

                let result = match operator {
                      BinaryOperator::Add 
                    | BinaryOperator::Sub
                    | BinaryOperator::Mul
                    | BinaryOperator::Div
                    | BinaryOperator::Rem
                    | BinaryOperator::BitshiftLeft
                    | BinaryOperator::BitshiftRight
                    | BinaryOperator::BitwiseAnd 
                    | BinaryOperator::BitwiseOr 
                    | BinaryOperator::BitwiseXor => lhs_anal.ty,

                      BinaryOperator::Eq 
                    | BinaryOperator::Ne 
                    | BinaryOperator::Gt 
                    | BinaryOperator::Ge 
                    | BinaryOperator::Lt 
                    | BinaryOperator::Le => Type::BOOL
                };

                AnalysisResult::new(result, true)
            },


            Expr::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(path, scope, rhs);
                let sym = rhs_anal.ty.sym(&mut self.types)?;

                if sym == SymbolId::ERROR { return Ok(AnalysisResult::error()) }
                if sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                match operator {
                    UnaryOperator::Not if sym == SymbolId::BOOL => (),
                    UnaryOperator::Neg if sym.is_sint() => (),
                    
                    _ => return Err(Error::InvalidUnaryOp { operator, rhs: rhs_anal.ty, source })
                }

                AnalysisResult::new(rhs_anal.ty, true)
            },


            Expr::If { condition, body, else_block } => {
                let cond = self.expr(path, scope, condition);
                if cond.ty.eq(&mut self.types, Type::ERROR) { return Ok(AnalysisResult::error()) }
                if !cond.ty.eq(&mut self.types, Type::NEVER) 
                   && !cond.ty.eq(&mut self.types, Type::BOOL) {
                    let range = self.ast.range(condition);
                    return Err(Error::InvalidType {
                        source: range, found: cond.ty, expected: Type::BOOL })
                }

                let body_anal = self.expr(path, scope, body);
                let mut value = body_anal.ty;

                (|| {
                    let Some(el) = else_block
                    else { return };

                    let el_anal = self.expr(path, scope, el);

                    if value.eq(&mut self.types, Type::ERROR) {
                        value = el_anal.ty
                    } else if el_anal.ty.ne(&mut self.types, value) {
                        let body = self.ast.range(body);
                        let else_block = self.ast.range(el);
                        self.error(el, Error::IfBodyAndElseMismatch {
                            body: (body, value), else_block: (else_block, el_anal.ty) });
                        return
                    }
                })();

                if value.ne(&mut self.types, Type::UNIT) && else_block.is_none() {
                    let body = self.ast.range(body);
                    return Err(Error::IfMissingElse { body: (body, value) })
                }

                AnalysisResult::new(value, true)
            },


            Expr::Match { .. } => todo!(),


            Expr::Block { block } => self.block(path, scope, &*block),


            Expr::CreateStruct { data_type, fields  } => {
                let ty = self.dt_to_ty(scope, data_type)?;

                let sym = ty.sym(&mut self.types)?;

                let sym = self.types.sym(sym);

                let sym_fields = {
                    let mut vec = Vec::new();
                    let gens = ty.gens(&mut self.types);
                    for f in sym.fields {
                        vec.push(self.gen_to_ty(f.1, gens)?)
                    }

                    vec
                };

                for (f, g) in fields.iter().zip(sym_fields) {
                    let expr = self.expr(path, scope, f.2);

                    if !expr.ty.eq(&mut self.types, g) {
                        self.error(id, Error::InvalidType {
                            source: f.1, found: expr.ty, expected: g });
                    }
                }

                AnalysisResult::new(ty, true)
            },


            Expr::AccessField { val, field_name  } => todo!(),


            Expr::CallFunction { .. } => todo!(),
            Expr::WithinNamespace { .. } => todo!(),
            Expr::WithinTypeNamespace { .. } => todo!(),


            Expr::Loop { body } => {
                self.block(path, scope, &*body);

                AnalysisResult::new(Type::UNIT, true)
            },


            Expr::Return(ret) => {
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return Err(Error::ReturnOutsideOfAFunction { source }) };

                let ret_anal = self.expr(path, scope, ret);
                if ret_anal.ty.eq(&mut self.types, Type::ERROR) { return Ok(AnalysisResult::error()) }
                if ret_anal.ty.eq(&mut self.types, Type::NEVER) { return Ok(AnalysisResult::never()) }

                if ret_anal.ty.ne(&mut self.types, func.ret) {
                    return Err(Error::ReturnAndFuncTypDiffer {
                        source, func_source: func.ret_source,
                        typ: ret_anal.ty, func_typ: func.ret })
                }

                AnalysisResult::new(Type::NEVER, true)
            },


            Expr::Continue => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none() { 
                    return Err(Error::ContinueOutsideOfLoop(source)) 
                }

                AnalysisResult::new(Type::NEVER, true)
            },


            Expr::Break => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none() { 
                    return Err(Error::ContinueOutsideOfLoop(source)) 
                }

                AnalysisResult::new(Type::NEVER, true)
            },


            Expr::Tuple(_) => {
                todo!();
            },


            Expr::AsCast { .. } => todo!(),
            Expr::Unwrap(_) => todo!(),
            Expr::OrReturn(_) => todo!(),
        }))();

        match result {
            Ok(v) => {
                self.type_info.set_expr(id, v);
                v
            },

            Err(v) => {
                self.error(id, v);
                AnalysisResult::error()
            },
        }
    }
}

