use crate::analysis::blocks::BlockId;

use super::*;

impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub fn eval_block(&mut self, id: BlockId, scope: ScopeId) -> AnalysisResult {
        self.ensure_validate(id);

        let path = self.blocks.block_state(id).path.unwrap();
        let ns = self.blocks.block_namespace(id).unwrap();
        let mut scope = 
        self.scopes.push(Scope::new(
            Some(scope),
            ScopeKind::ImplicitNamespace(ns),
        ));

        let mut last_node = None;
        let mut has_never = false;
        let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
        for index in 0..len {
            let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
            let eval = self.node(path, &mut scope, ns, node);
            if eval.ty.is_never(&mut self.syms) {
                has_never = true;
            }
            last_node = Some(eval);
        }

        match last_node {
            _ if has_never => AnalysisResult::new(Type::NEVER),
            Some(value) => value,
            None => AnalysisResult::new(Type::UNIT),
        }
    }


    pub fn block(&mut self, id: BlockId, scope: ScopeId) -> AnalysisResult {
        self.eval_block(id, scope)
    }

    pub fn node(&mut self, path: StringIndex,
                scope: &mut ScopeId, ns: NamespaceId, node: NodeId) -> AnalysisResult {
        match node {
            NodeId::Decl(decl) => {
                if let Decl::Error(e) = self.ast.decl(decl) {
                    self.set_error(decl, e);
                    return AnalysisResult::new(self.error_type(e));
                }

                self.decl(scope, ns, decl);
                AnalysisResult::new(Type::UNIT)
            },

            NodeId::Stmt(stmt) => {
                self.stmt(path, scope, stmt);
                AnalysisResult::new(Type::UNIT)
            },

            NodeId::Expr(expr) => self.expr(path, *scope, expr),

            NodeId::Err(err) => {
                AnalysisResult::new(self.error_type(err))
            },
        }
    }


    pub fn decl(&mut self, scope: &mut ScopeId, ns: NamespaceId, n: DeclId) {
        let decl = self.ast.decl(n);
        match decl {
            Decl::Struct { .. } => (),
            Decl::Enum { .. } => (),
            Decl::Alias { .. } => (),
            Decl::ImportFile { .. } => unreachable!(),
            Decl::ImportRepo { .. } => unreachable!(),
            Decl::Error(_) => unreachable!(),


            Decl::Trait { .. } => (),

            
            Decl::Function { sig, body, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(Ok(func_id)) = ns.get_sym(sig.name)
                else { return };

                let func = match self.syms.sym(func_id).kind() {
                    SymbolKind::Function(func) => func,
                    _ => return,
                };
                let block_id = self.blocks.block_id(NodeId::Decl(n));
                let Some(ty_scope) = self.blocks.block_ty_scope(block_id)
                else { return };

                let mut generics = std::vec::Vec::new();
                self.scopes.get(ty_scope).collect_generics(
                    &self.scopes,
                    &mut generics,
                );

                if sig.name == self.string_map.insert("main") {
                    self.startups.push(func_id);
                }

                let mut eval_scope = ty_scope;
                for arg in func.args() {
                    let ty = arg.symbol().to_ty(&generics, &mut self.syms);
                    let variable = VariableScope::new(arg.name(), ty, true);
                    eval_scope = self.scopes.push(Scope::new(
                        Some(eval_scope),
                        ScopeKind::VariableScope(variable),
                    ));
                }

                let ret = func.ret().to_ty(&generics, &mut self.syms);
                eval_scope = self.scopes.push(Scope::new(
                    Some(eval_scope),
                    ScopeKind::Function(FunctionScope::new(
                        ret,
                        sig.return_type.range(),
                    )),
                ));

                let flow = self.control_flow.suspend();
                let anal = self.eval_block(block_id, eval_scope);
                self.control_flow.restore(flow);

                if !anal.ty.eq(&mut self.syms, ret) {
                    let item =
                    body.last()
                        .map(|n| self.ast.range(*n))
                        .unwrap_or(body.range());

                    self.error(n, Error::FunctionBodyAndReturnMismatch {
                        header: sig.source, item,
                        return_type: ret, body_type: anal.ty });
                }
            },


            Decl::Impl { body, .. } => {
                let Some((ty, _, _)) = self.type_info.impls.get(&n)
                else { return; };
                
                let GenericKind::Sym(sym, _) = ty.kind()
                else { return; };

                let ns = self.syms.sym_ns(sym);

                let path = self.namespaces.get_ns(ns).path;
                let mut scope = self.scopes.push(Scope::new(*scope, ScopeKind::AliasDecl(StringMap::SELF_TY, *ty)));

                for n in body.iter() {
                    self.node(path, &mut scope, ns, *n);
                }

            },




            Decl::ImplTrait { body, header, data_type, .. } => {
                let Some(&(trait_ty, ty, _gens)) = self.type_info.impls.get(&n)
                else { return; };

                let Some(trait_sym_id) = trait_ty.sym()
                else {
                    return;
                };

                let Some(sym) = ty.sym()
                else {
                    return;
                };

                let trait_sym = self.syms.sym(trait_sym_id);
                let SymbolKind::Trait(tr) = trait_sym.kind()
                else {
                    self.error(n, Error::ImplTraitOnNonTrait(data_type.range()));
                    return;
                };

                let bound_error = self.validate_trait_bound(n.into(), trait_ty, &[]);
                if let Some(error) = bound_error {
                    if let Some(entry) = self.syms.traits(sym)
                        .get_mut(&trait_sym_id)
                        .and_then(|impls| impls.iter_mut().find(|entry| entry.declaration == Some(n)))
                    {
                        entry.bound_error = Some(error);
                    }
                }

                

                let path = trait_sym.name();

                let ns_id = self.syms.traits(sym).get(&trait_sym_id)
                    .and_then(|impls| impls.iter().find(|entry| entry.declaration == Some(n)))
                    .unwrap().namespace;
                let scope = Scope::new(*scope, ScopeKind::ImplicitNamespace(ns_id));
                let scope = self.scopes.push(scope);

                let scope = Scope::new(scope, ScopeKind::AliasDecl(StringMap::SELF_TY, ty));
                let mut scope = self.scopes.push(scope);



                for node in body.iter() {
                    self.node(path, &mut scope, ns_id, *node);
                }

                let mut ns = self.namespaces.get_ns_mut(ns_id);

                let mut missing = sti::vec![];

                for (name, ft) in tr.funcs.iter() {
                    let Some(sym) = ns.get_sym(*name)
                    else {
                        missing.push(*name);
                        continue;
                    };


                    let Ok(sym_id) = sym
                    else { continue };

                    let Some(sym) = self.syms.sym_ok(sym_id)
                    else {
                        self.error(n, Error::TraitMemberNotFunction {
                            source: header,
                            name: *name,
                        });
                        ns = self.namespaces.get_ns_mut(ns_id);
                        continue;
                    };

                    let SymbolKind::Function(f) = sym.kind()
                    else {
                        self.error(n, Error::TraitMemberNotFunction {
                            source: header,
                            name: *name,
                        });
                        ns = self.namespaces.get_ns_mut(ns_id);
                        continue;
                    };


                    if f.args().len() != ft.args().len() {
                        let decl = f.decl().unwrap();
                        let source = self.ast.range(decl);

                        self.error(decl, Error::FunctionArgsMismatch {
                            source, 
                            sig_len: ft.args().len(),
                            call_len: f.args().len() 
                        });

                        ns = self.namespaces.get_ns_mut(ns_id);

                        continue;
                    }

                    for (arg, trait_arg) in f.args().iter().zip(ft.args()) {
                        let arg_ty = arg.symbol().rec_replace(self.output, StringMap::SELF_TY, ty);
                        let mut farg = trait_arg.symbol().rec_replace(self.output, StringMap::SELF_TY, ty);

                        if let Some(trait_args) = trait_ty.gens() {
                            for (t_gen, t_arg) in trait_sym.generics().iter().zip(trait_args.iter()) {
                                farg = farg.rec_replace(self.output, t_gen.name(), *t_arg);
                            }
                        }

                        if arg.is_inout() != trait_arg.is_inout() || arg_ty != farg {
                            let decl = f.decl().unwrap();

                            self.error(decl, Error::InvalidArgument { source: arg_ty.range() });

                            ns = self.namespaces.get_ns_mut(ns_id);
                        }
                    }

                    let arg = f.ret().rec_replace(self.output, StringMap::SELF_TY, ty);
                    let mut farg = ft.ret().rec_replace(self.output, StringMap::SELF_TY, ty);

                    if let Some(trait_args) = trait_ty.gens() {
                        for (t_gen, t_arg) in trait_sym.generics().iter().zip(trait_args.iter()) {
                            farg = farg.rec_replace(self.output, t_gen.name(), *t_arg);
                        }
                    }

                    if arg != farg {
                        let decl = f.decl().unwrap();

                        self.error(decl, Error::InvalidArgument { source: arg.range() });

                        ns = self.namespaces.get_ns_mut(ns_id);
                    }


                }

                if !missing.is_empty() {
                    self.error(n, Error::MissingFuncs { source: header, fields: missing });
                }
            },




            Decl::Module { name, body, .. } => {
                let ns = self.namespaces.get_ns(ns);

                let Some(Ok(module_ns)) = ns.get_sym(name)
                else { return; };

                let module_ns = self.syms.as_ns(module_ns);


                let scope = Scope::new(*scope, ScopeKind::ImplicitNamespace(module_ns));
                let mut scope = self.scopes.push(scope);

                let path = self.namespaces.get_ns(module_ns).path;
                for n in body.iter() {
                    self.node(path, &mut scope, module_ns, *n);
                }
            },


            Decl::Using { .. } => (),
            Decl::Extern { .. } => (),

            Decl::Attribute { decl: decl_id, attr } => {
                let attr_name = attr.identifier();
                if matches!(attr_name, Some(name) if self.string_map.get(name) == "silent") {
                    for param in attr.params {
                        self.error(n, Error::UnknownAttrParam {
                            param: param.range, attr: attr.range,
                        });
                    }
                    self.decl(scope, ns, decl_id);
                    return;
                }

                self.decl(scope, ns, decl_id);

                match attr_name.map(|name| self.string_map.get(name)) {
                    Some("doc") => {
                        let valid = 
                           attr.params.len() == 1
                        && matches!(attr.params[0].value, AttributeValue::Literal(Literal::String(_)));

                        if !valid {
                            let value = attr.params.first().map(|param| param.range).unwrap_or(attr.range);
                            self.error(n, Error::InvalidValueForAttr {
                                attr: (attr.range, StringMap::DOC),
                                value,
                                expected: "'a string literal'",
                            });
                        }

                    },

                    Some("test") => {
                        let mut decl_id = decl_id;
                        while let Decl::Attribute { decl, .. } = self.ast.decl(decl_id) {
                            decl_id = decl;
                        }
                        let decl = self.ast.decl(decl_id);
                        let Decl::Function { 
                            sig: FunctionSignature {
                                name,
                                arguments: &[], 
                                generics: &[],
                                return_type: DataType { kind: DataTypeKind::Unit, .. },
                                ..
                            }, 
                            .. 
                        } = decl
                        else {
                            let range = self.ast.range(decl_id);
                            self.error(n, Error::InvalidValueForAttr {
                                attr: (attr.range, attr_name.unwrap()), value: range, expected: "'fn()'" });
                            return;
                        };

                        let Some(Ok(func)) = self.namespaces.get_ns(ns).get_sym(name)
                        else { return; };

                        let mut should_panic = false;
                        for p in attr.params {
                            match p.identifier().map(|name| self.string_map.get(name)) {
                                Some("should_panic") => should_panic = true,

                                _ => {
                                    self.error(n, Error::UnknownAttrParam {
                                        param: p.range, attr: attr.range,
                                    });
                                },
                            }
                        }

                        self.tests.push((func, should_panic));
                    },


                    Some("cached") => {
                        for p in attr.params {
                            self.error(n, Error::UnknownAttrParam {
                                param: p.range, attr: attr.range,
                            });
                        }

                        let decl = self.ast.decl(decl_id);
                        let Decl::Function { 
                            sig: FunctionSignature {
                                name,
                                ..
                            }, 
                            .. 
                        } = decl
                        else {
                            let range = self.ast.range(decl_id);
                            self.error(n, Error::InvalidValueForAttr {
                                attr: (attr.range, attr_name.unwrap()), value: range, expected: "'a function'" });
                            return;
                        };

                        let Some(Ok(func)) = self.namespaces.get_ns(ns).get_sym(name)
                        else { return; };

                        self.syms.cached_fn(func);
                    }

                    _ => {
                        self.error(n, Error::UnknownAttr(attr.range));
                    }
                }
            },
        }
    }


    pub fn resolve_pattern(
        &mut self, id: NodeId, scope: &mut ScopeId, 
        pattern: Pattern, rhs: AnalysisResult, rhs_range: SourceRange, mutable: bool
    ) -> Result<(), Error> {
        //
        // yes, I'm aware this is a very.. brave way of doing error handling
        // far as I can see it shouldn't cause any problems but it might
        // in the future.
        //
        // while writing it, the idea is that since for all of these we're
        // already creating a type and then doing an eq on it, it should be
        // able to infer as much as possible and leave the program in a
        // reasonable way.
        //
        // oh the lengths we go for graceful errors
        //
        let mut result = Once::new();

        (|| {
            match pattern.kind() {
                PatternKind::Variable(name) => {
                    let vs = VariableScope::new(name, rhs.ty, mutable);
                    let vs = Scope::new(*scope, ScopeKind::VariableScope(vs));
                    *scope = self.scopes.push(vs);
                },


                PatternKind::Tuple(items) => {
                    let syms = Vec::from_value_in(self.output, items.len(), None);

                    let tuple = self.tuple_sym(pattern.source(), &syms);
                    let gens = self.tuple_gens(items.len(), pattern.source(), id);

                    let ty = Type::Ty(tuple, gens);

                    if !ty.eq(&mut self.syms, rhs.ty) {
                        // if they're not equal, we need to check whether rhs
                        // is just not a tuple or it's a different sized tuple

                        match rhs.ty.sym(&mut self.syms) {
                            Ok(sym) => {
                                let sym = self.syms.sym(sym);

                                if let SymbolKind::Container(cont) = sym.kind()
                                && cont.kind() == ContainerKind::Tuple {
                                    result.set(Error::VariableTupleAndHintTupleSizeMismatch(
                                        pattern.source(),
                                        cont.fields().len(),
                                        items.len()
                                    ));
                                } else {
                                    result.set(Error::VariableValueNotTuple(rhs_range));
                                }
                            },


                            Err(e) => {
                                result.set(e);
                            },
                        }

                    }


                    let gens = ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);
                    for (&item, (_, ty)) in items.iter().zip(gens.iter()) {
                        let vs = VariableScope::new(item, *ty, mutable);
                        let vs = Scope::new(*scope, ScopeKind::VariableScope(vs));
                        *scope = self.scopes.push(vs);
                    }

                },
            }

        })();

        if let Some(err) = result.into_inner() {
            return Err(err)
        }

        Ok(())
    }


    pub fn stmt(&mut self, path: StringIndex,
                scope: &mut ScopeId, id: StmtId) {
        let source = self.ast.range(id);
        let stmt = self.ast.stmt(id);
        match stmt {
            Stmt::Variable { mutable, pat, hint, rhs } => {
                let mut rhs_anal = self.expr(path, *scope, rhs);

                let mut validate_hint = || {
                    if let Some(hint) = hint {
                        let hint = self.dt_to_ty(*scope, id, hint);
                        if let Some(err) = hint.as_err(&mut self.syms) {
                            rhs_anal.ty.eq(&mut self.syms, hint);
                            rhs_anal.ty = hint;
                            return Err(err);
                        }

                        if !rhs_anal.ty.eq(&mut self.syms, hint) {
                            rhs_anal.ty = hint;
                            return Err(self.error(id, Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source }))
                        }

                        // cute trick.
                        // 
                        // `err` and `!` types can coerce into whatever `hint`
                        // was. so if the equality check above passed we can
                        // just set it to `hint` to avoid some headache.
                        //
                        rhs_anal.ty = hint;
                    }

                    Ok(())
                };

                let validate_hint = validate_hint();

                let rhs_range = self.ast.range(rhs);

                let result = self.resolve_pattern(
                    id.into(), scope, pat, rhs_anal, rhs_range, mutable);

                if let Err(e) = validate_hint {
                    self.set_error(id, e);
                }

                if let Err(e) = result {
                    self.error(id, e);
                    return;
                }

            },


            Stmt::UpdateValue { lhs, rhs  } => {
                let lhs_anal = self.expr(path, *scope, lhs);
                let rhs_anal = self.expr(path, *scope, rhs);

                if !lhs_anal.ty.eq(&mut self.syms, rhs_anal.ty) {
                    self.error(id, Error::ValueUpdateTypeMismatch { lhs: lhs_anal.ty, rhs: rhs_anal.ty, source });
                }

                let range = self.ast.range(lhs);
                if lhs_anal.is_captured && self.is_assignable_place(lhs) {

                    self.error(id, Error::CannotMutateCapturedValue { source: range });
                } else if !lhs_anal.is_mut && self.is_assignable_place(lhs) {
                    let name = self.root_identifier(lhs)
                        .expect("assignable places always root at an identifier");
                    self.error(id, Error::AssignmentToImmutableVariable { name, source: range });
                } else if !lhs_anal.is_mut || !self.is_assignable_place(lhs) {
                    self.error(id, Error::AssignIsNotLHSValue { source: range });
                }
            },


            Stmt::ForLoop { binding, expr, body: _ } => {
                let iter_anal = self.expr(path, *scope, expr);

                // check if the exprs type is an iterable
                let Ok(sym) = iter_anal.ty.sym(&mut self.syms)
                else {
                    let range = self.ast.range(expr);

                    // The iterable's type is an unresolved inference variable;
                    // report it so the loop binding carries a real error, and
                    // resolve the variable so the post-pass doesn't repeat it.
                    let err = self.error(id, Error::UnableToInfer(range, None));
                    let err_ty = self.error_type(err);
                    iter_anal.ty.eq(&mut self.syms, err_ty);

                    let scope = Scope::new(*scope, ScopeKind::Loop);
                    let mut scope = self.scopes.push(scope);

                    let _ = self.resolve_pattern(
                        id.into(), &mut scope, binding, 
                        AnalysisResult::new(err_ty), range, true
                    );

                    self.control_flow.enter_loop();
                    let _ = self.eval_block(self.blocks.block_id(NodeId::Stmt(id)), scope);
                    self.control_flow.exit_loop();

                    return;
                };

                if iter_anal.ty.is_err(&mut self.syms) {
                    return;
                }

                let func = self.syms.sym_ns(sym);
                let ns = self.namespaces.get_ns(func);
                let Some(sym) = ns.get_sym(StringMap::ITER_NEXT_FUNC)
                else { 
                    let range = self.ast.range(expr);
                    let err = self.error(id, Error::ValueIsntAnIterator { ty: iter_anal.ty, range });
                    let err_ty = self.error_type(err);

                    let scope = Scope::new(*scope, ScopeKind::Loop);
                    let mut scope = self.scopes.push(scope);

                    let _ = self.resolve_pattern(
                        id.into(), &mut scope, binding, 
                        AnalysisResult::new(err_ty), range, true
                    );

                    self.control_flow.enter_loop();
                    let _ = self.eval_block(self.blocks.block_id(NodeId::Stmt(id)), scope);
                    self.control_flow.exit_loop();

                    return;
                };

                let Ok(sym) = sym 
                else { return };
                

                // check if the exprs type is a mutable iterable
                let binding_ty = self.syms.sym(sym);
                let SymbolKind::Function(binding_ty) = binding_ty.kind()
                else { unreachable!() };

                let gens = iter_anal.ty.gens(&mut self.syms);
                let gens = self.syms.get_gens(gens);

                let binding_ty = binding_ty.ret().to_ty(gens, &mut self.syms);

                // unwrap the option
                let binding_ty = binding_ty.gens(&mut self.syms);
                let binding_ty = self.syms.get_gens(binding_ty);
                if binding_ty.is_empty() { return; }
                let binding_ty = binding_ty[0].1;

                let scope = Scope::new(*scope, ScopeKind::Loop);
                let mut scope = self.scopes.push(scope);

                let _ = self.resolve_pattern(
                    id.into(), &mut scope, binding, 
                    AnalysisResult::new(binding_ty), source, true
                );


                self.control_flow.enter_loop();
                let _ = self.eval_block(self.blocks.block_id(NodeId::Stmt(id)), scope);
                self.control_flow.exit_loop();

            },


            Stmt::Attribute { attr, node } => {
                if matches!(attr.identifier(), Some(name) if self.string_map.get(name) == "silent") {
                    for param in attr.params {
                        self.error(id, Error::UnknownAttrParam {
                            param: param.range, attr: attr.range,
                        });
                    }
                    self.silent_ranges.push(self.ast.range(id));
                } else {
                    self.error(id, Error::UnknownAttr(attr.range));
                }

                match node {
                    NodeId::Stmt(stmt) => self.stmt(path, scope, stmt),
                    NodeId::Expr(expr) => { self.expr(path, *scope, expr); },
                    NodeId::Err(_) => (),
                    NodeId::Decl(_) => unreachable!(),
                }
            },
        }
    }


    fn is_assignable_place(&self, expr: ExprId) -> bool {
        match self.ast.expr(expr) {
            Expr::Identifier(_, _) => true,

            Expr::AccessField { val, .. }
            | Expr::IndexList { list: val, .. } => self.is_assignable_place(val),

            Expr::Unwrap(val)
            | Expr::OrReturn(val) => self.is_assignable_place(val),

            _ => false,
        }
    }

    fn root_identifier(&self, expr: ExprId) -> Option<StringIndex> {
        match self.ast.expr(expr) {
            Expr::Identifier(ident, _) => Some(ident),

            Expr::AccessField { val, .. }
            | Expr::IndexList { list: val, .. } => self.root_identifier(val),

            Expr::Unwrap(val)
            | Expr::OrReturn(val) => self.root_identifier(val),

            _ => None,
        }
    }

    fn trait_method_sym(
        &mut self,
        trait_id: SymbolId,
        trait_ty: Generic<'out>,
        func: FunctionTy<'out>,
        receiver: Generic<'out>,
        implementation_generics: &'out [BoundedGeneric<'out>],
    ) -> (SymbolId, &'out [BoundedGeneric<'out>]) {
        let method_generics = func.declared_generics();
        let mut all_gens_vec = std::vec::Vec::new();
        all_gens_vec.extend_from_slice(implementation_generics);
        receiver.collect_generics(&mut all_gens_vec);
        trait_ty.collect_generics(&mut all_gens_vec);
        for g in method_generics {
            if !all_gens_vec.iter().any(|existing| existing.name == g.name) {
                all_gens_vec.push(*g);
            }
        }

        let trait_sym = self.syms.sym(trait_id);
        if let Some(trait_args) = trait_ty.gens() {
            for (trait_generic, trait_arg) in trait_sym.generics().iter().zip(trait_args) {
                let GenericKind::Generic(argument_generic) = trait_arg.kind()
                else { continue; };
                let Some(index) = all_gens_vec.iter()
                    .position(|generic| generic.name == argument_generic.name)
                else { continue; };

                let mut bounds = sti::vec::Vec::with_cap_in(
                    self.output,
                    all_gens_vec[index].bounds.len() + trait_generic.bounds.len(),
                );
                bounds.extend_from_slice(all_gens_vec[index].bounds);
                for required_bound in trait_generic.bounds {
                    let mut required_bound = *required_bound;
                    for (generic, argument) in trait_sym.generics().iter().zip(trait_args) {
                        required_bound = required_bound.rec_replace(
                            self.output,
                            generic.name,
                            *argument,
                        );
                    }
                    bounds.push(required_bound);
                }

                all_gens_vec[index] = BoundedGeneric::new(argument_generic.name, bounds.leak());
            }
        }

        let all_generics = sti::vec::Vec::from_slice_in(self.output, &all_gens_vec).leak();

        let closure = self.syms.new_closure();
        let mut func_args = sti::vec::Vec::with_cap_in(self.output, func.args().len());
        for arg in func.args() {
            let mut symbol = arg
                .symbol()
                .rec_replace(self.output, StringMap::SELF_TY, receiver);

            if let Some(trait_args) = trait_ty.gens() {
                for (t_gen, t_arg) in trait_sym.generics().iter().zip(trait_args.iter()) {
                    symbol = symbol.rec_replace(self.output, t_gen.name(), *t_arg);
                }
            }

            func_args.push(FunctionArgument::new_inout(arg.name(), symbol, arg.is_inout()));
        }

        let mut ret = func
            .ret()
            .rec_replace(self.output, StringMap::SELF_TY, receiver);

        if let Some(trait_args) = trait_ty.gens() {
            for (t_gen, t_arg) in trait_sym.generics().iter().zip(trait_args.iter()) {
                ret = ret.rec_replace(self.output, t_gen.name(), *t_arg);
            }
        }

        let symbol = self.func_sym(
            closure,
            func_args.leak(),
            ret,
            all_generics,
            method_generics,
        );

        (symbol, all_generics)
    }

    pub(crate) fn validate_trait_bound(
        &mut self,
        node: NodeId,
        bound: Generic<'out>,
        bindings: &[(BoundedGeneric<'out>, Type)],
    ) -> Option<ErrorId> {
        let mut required_generics = std::vec::Vec::new();
        bound.collect_generics(&mut required_generics);
        if required_generics.iter().any(|generic| {
            !bindings.iter().any(|(binding, _)| binding.name == generic.name)
        }) {
            return None;
        }

        let bound_ty = bound.to_ty(bindings, &mut self.syms);
        let Some((index, actual, trait_id)) =
            self.syms.trait_argument_bound_failure(bound_ty)
        else {
            return None;
        };

        let source = bound.gens()
            .and_then(|generics| generics.get(index))
            .map(|generic| generic.range())
            .unwrap_or(bound.range());

        Some(self.error(node, Error::TypeDoesntImplTrait {
            source,
            ty: actual,
            tr: trait_id,
        }))
    }

    fn trait_method_entry_matches(
        syms: &mut SymbolMap<'out>,
        entry: TraitImplEntry<'out>,
        receiver_ty: Option<Type>,
        rejection: &mut Option<TraitMethodRejection>,
    ) -> bool {
        let mut bindings = std::vec::Vec::with_capacity(entry.generics.len());
        if let Some(receiver_ty) = receiver_ty {
            if !syms.match_impl_type(
                entry.receiver,
                receiver_ty,
                entry.generics,
                &mut bindings,
            ) {
                return false;
            }
        }

        if let Some(error) = entry.bound_error {
            // A declaration-time bound error outranks any call-site failure.
            if !matches!(rejection, Some(TraitMethodRejection::BoundError(_))) {
                *rejection = Some(TraitMethodRejection::BoundError(error));
            }
            return false;
        }

        let mut trait_generics = std::vec::Vec::new();
        entry.trait_ty.collect_generics(&mut trait_generics);
        if trait_generics.iter().all(|generic| {
            bindings.iter().any(|(binding, _)| binding.name == generic.name)
        }) {
            let trait_ty = entry.trait_ty.to_ty(&bindings, syms);
            if !syms.trait_arguments_satisfy_bounds(trait_ty) {
                if rejection.is_none()
                && let Some((_, actual, trait_id)) =
                    syms.trait_argument_bound_failure(trait_ty)
                {
                    *rejection = Some(TraitMethodRejection::BoundFailure(actual, trait_id));
                }
                return false;
            }
        }

        true
    }


    fn find_trait_method_candidate(
        &mut self,
        scope: ScopeId,
        id: ExprId,
        range: SourceRange,
        receiver_ty: Option<Type>,
        sym_id: SymbolId,
        method_name: StringIndex,
    ) -> Result<Option<(SymbolId, Generic<'out>, FunctionTy<'out>, Generic<'out>, &'out [BoundedGeneric<'out>])>, ErrorId> {
        let candidates = self.syms.traits(sym_id).clone();
        let mut candidate = None;
        let mut ambiguous_trait_method = false;
        let mut rejection = None;

        self.scopes.get(scope).over::<()>(&self.scopes, |scope| {
            let ScopeKind::ImplicitNamespace(ns) = scope.kind() else { return None };
            let ns = self.namespaces.get_ns(ns);
            for trait_id in ns.syms().values().filter_map(|sym| sym.result().ok()) {
                let Some(impls) = candidates.get(&trait_id) else { continue };
                for entry in impls {
                    let sym = self.syms.sym(trait_id);
                    let SymbolKind::Trait(tr) = sym.kind() else { continue };
                    let Some(ft) = tr.funcs.iter().find(|x| x.0 == method_name) else { continue };

                    if !Self::trait_method_entry_matches(
                        &mut self.syms,
                        *entry,
                        receiver_ty,
                        &mut rejection,
                    ) {
                        continue;
                    }

                    if candidate.is_none() {
                        candidate = Some((trait_id, entry.trait_ty, ft.1, entry.receiver, entry.generics));
                    } else {
                        ambiguous_trait_method = true;
                        return Some(());
                    }
                }
            }
            candidate.as_ref().map(|_| ())
        });

        if ambiguous_trait_method {
            return Err(self.error(id, Error::AmbiguousTraitMethod {
                source: range,
                name: method_name,
            }));
        }

        if candidate.is_none() {
            for (trait_id, impls) in candidates {
                for entry in impls {
                    if entry.namespace != NamespaceId::MAX { continue; }
                    let sym = self.syms.sym(trait_id);
                    let SymbolKind::Trait(tr) = sym.kind() else { continue };
                    let Some(ft) = tr.funcs.iter().find(|x| x.0 == method_name) else { continue };

                    if !Self::trait_method_entry_matches(
                        &mut self.syms,
                        entry,
                        receiver_ty,
                        &mut rejection,
                    ) {
                        continue;
                    }

                    if candidate.is_none() {
                        candidate = Some((trait_id, entry.trait_ty, ft.1, entry.receiver, entry.generics));
                    } else {
                        ambiguous_trait_method = true;
                        break;
                    }
                }
            }
        }

        if ambiguous_trait_method {
            return Err(self.error(id, Error::AmbiguousTraitMethod {
                source: range,
                name: method_name,
            }));
        }

        if candidate.is_none() {
            if let Some(rejection) = rejection {
                return Err(match rejection {
                    TraitMethodRejection::BoundError(error) => error,
                    TraitMethodRejection::BoundFailure(actual, trait_id) =>
                        self.error(id, Error::TypeDoesntImplTrait {
                            source: range,
                            ty: actual,
                            tr: trait_id,
                        }),
                });
            }
        }

        Ok(candidate)
    }


    pub fn expr(&mut self, path: StringIndex, scope: ScopeId, id: ExprId) -> AnalysisResult {
        let range = self.ast.range(id);

        let expr = self.ast.expr(id);
        let result = (|| -> Result<AnalysisResult, ErrorId> {Ok(match expr {
            Expr::Unit => AnalysisResult::new(Type::UNIT),


            Expr::Literal(lit) => {
                match lit {
                    lexer::Literal::Integer(_) => AnalysisResult::new(Type::I64),
                    lexer::Literal::Float(_)   => AnalysisResult::new(Type::F64),
                    lexer::Literal::String(_)  => AnalysisResult::new(Type::STR),
                    lexer::Literal::Bool(_)    => AnalysisResult::new(Type::BOOL),
                }
            },


            Expr::Paren(e) => self.expr(path, scope, e),


            Expr::Identifier(ident, gens) => {

                let mut variable = || {
                    let sym_id = self.scopes.get(scope).find_super(&self.scopes)?;
                    let candidate = match self.find_trait_method_candidate(scope, id, range, None, sym_id, ident) {
                        Ok(c) => c,
                        Err(e) => return Some(Err(Err(e))),
                    };

                    let Some((t, trait_ty, func, g, impl_generics)) = candidate
                    else { return None; };

                    self.type_info.set_acc(id, trait_ty);
                    self.type_info.set_ident(id, Some(sym_id));
                    let (sym, _) = self.trait_method_sym(t, trait_ty, func, g, impl_generics);

                    Some(Err(Ok(sym)))
                };


                let variable = 
                if let Some(v) = variable() { Some(v) }
                else {
                    Some(match self.scopes.get(scope)
                        .find_var(ident, &self.scopes, &self.namespaces, &mut self.syms) {
                        Ok(v) => Ok(v),
                        Err(SymbolGetResult::Symbol(sym)) => Err(Ok(sym)),
                        Err(SymbolGetResult::Errored(e)) => Err(Err(e)),
                        Err(SymbolGetResult::Private | SymbolGetResult::Undefined) => {
                            Err(Err(self.error(id, Error::VariableNotFound { name: ident, source: range })))
                        },
                    })
                };


                let Some(variable) = variable
                else { return Err(self.error(id, Error::VariableNotFound { name: ident, source: range })) };

                match variable {
                    Ok((variable, is_captured)) => {
                        if gens.is_some() {
                            return Err(self.error(id, Error::GenericLenMismatch { source: range, found: gens.map(|gs| gs.len()).unwrap_or(0), expected: 0 }))
                        }

                        return Ok(if is_captured {
                            AnalysisResult::captured(variable.ty())
                        } else {
                            let mut anal = AnalysisResult::new(variable.ty());
                            anal.is_mut = variable.is_mutable();
                            anal
                        })
                    },


                    Err(sym) => {
                        let sym_id = sym?;

                        let sym = self.syms.sym(sym_id);

                        match sym.kind() {
                            SymbolKind::Function(func) => {
                                self.type_info.set_ident(id, Some(sym_id));

                                if let Some(gens) = gens
                                && sym.generics().len() != gens.len() {
                                    return Err(self.error(id, Error::GenericLenMismatch { source: range, found: gens.len(), expected: sym.generics().len() }))
                                }

                                let mut vgens = sti::vec::Vec::with_cap_in(self.output, sym.generics().iter().len());

                                if let Some(gens) = gens {
                                    for (g, dt) in sym.generics().iter().zip(gens.iter()) {
                                        let sym = self.dt_to_ty(scope, id, *dt);
                                        if let Some(err) = sym.as_err(&mut self.syms) { return Err(err); }

                                        let g = BoundedGeneric::new(g.name(), &[]);
                                        vgens.push((g, sym));
                                    }

                                } else if let Some(qualified) =
                                self.scopes.get(scope)
                                    .find_qualified_type(&self.scopes)
                                    .map(|ty| ty.gens(&mut self.syms))
                                && self.syms.get_gens(qualified).len() == sym.generics().len() {
                                    for (g, (_, ty)) in sym.generics().iter().zip(self.syms.get_gens(qualified).iter()) {
                                        vgens.push((*g, *ty));
                                    }
                                } else {
                                    for g in sym.generics().iter() {
                                        let var = self.syms.new_var(id, g.name, range);
                                        vgens.push((*g, var));
                                    }
                                }

                                let gens = self.syms.add_gens(vgens.leak());

                                let mut anal = match func.kind() {
                                    FunctionKind::Closure(_) => AnalysisResult::new(Type::Ty(sym_id, gens)),
                                    _ => {
                                        let closure = self.syms.new_closure();

                                        let sym = self.func_sym(
                                            closure,
                                            func.args(),
                                            func.ret(),
                                            sym.generics(),
                                            func.declared_generics()
                                        );
                                        AnalysisResult::new(Type::Ty(sym, gens))
                                    }
                                };

                                anal.is_mut = true;
                                return Ok(anal)

                            },

                            _ => (),
                        }


                    },
                };

                return Err(self.error(id, Error::VariableNotFound { name: ident, source: range }))
            },


            Expr::Closure { args, body } => {
                let closure = self.syms.new_closure();
                let ret_var = self.syms.new_var(id, StringMap::RESULT, range);

                let closure_scope = self.scopes.push(Scope::new(Some(scope), ScopeKind::Function(FunctionScope { ret: ret_var, ret_source: range })));
                let closure_scope = self.scopes.push(Scope::new(Some(closure_scope), ScopeKind::Closure(closure)));
                let mut active_scope = closure_scope;


                // create generics for inference
                let mut sargs = sti::vec::Vec::new_in(self.syms.arena());
                for arg in args {
                    let ty =
                    if let Some(ty) = arg.1 { self.dt_to_ty(scope, id, ty) }
                    else { self.syms.new_var(id, arg.0, arg.3) };

                    active_scope =
                    self.scopes.push(Scope::new(
                        Some(active_scope),
                        ScopeKind::VariableScope(VariableScope::new(arg.0, ty, true))
                    ));

                    sargs.push((arg.0, ty, arg.2, arg.3));
                }


                // process the body
                let flow = self.control_flow.suspend();
                let ret = self.expr(path, active_scope, body);
                self.control_flow.restore(flow);


                if !ret.ty.eq(&mut self.syms, ret_var) {
                    let source = self.ast.range(body);
                    return Err(self.error(id, Error::InvalidType { source, found: ret.ty, expected: ret_var }));
                }


                let mut fargs = sti::vec::Vec::new_in(self.syms.arena());
                let mut gens = sti::vec::Vec::with_cap_in(self.syms.arena(), sargs.len() + 1);
                let mut gen_list = sti::vec::Vec::with_cap_in(self.syms.arena(), sargs.len() + 1);
                let t = BoundedGeneric::new(StringMap::INVALID_IDENT, &[]);
                let ret_ty = ret.ty;
                gens.push((t, ret_ty));
                gen_list.push(t);

                for (i, arg) in sargs.iter().enumerate() {
                    let g = self.string_map.num(i);
                    let g = BoundedGeneric::new(g, &[]);
                    gens.push((g, arg.1));
                    gen_list.push(g);
                    fargs.push(FunctionArgument::new_inout(
                        arg.0,
                        Generic::new(arg.3, GenericKind::Generic(g)),
                        arg.2
                    ));
                }

                let ret = Generic::new(range, GenericKind::Generic(t));

                let gen_list = gen_list.leak();
                let closure_ty = self.func_sym(
                    closure,
                    fargs.leak(),
                    ret,
                    gen_list,
                    gen_list
                );

                let gens = self.syms.add_gens(gens.leak());

                AnalysisResult::new(Type::Ty(closure_ty, gens))
            }


            Expr::Range { lhs, rhs  } => {
                let lhs_anal = self.expr(path, scope, lhs);
                let rhs_anal = self.expr(path, scope, rhs);

                if !lhs_anal.ty.is_int(&mut self.syms) {
                    let range = self.ast.range(lhs);
                    return Err(self.error(id, Error::InvalidRange { source: range, ty: lhs_anal.ty }));
                }


                if !rhs_anal.ty.is_int(&mut self.syms) {
                    let range = self.ast.range(rhs);
                    return Err(self.error(id, Error::InvalidRange { source: range, ty: rhs_anal.ty }));
                }


                AnalysisResult::new(Type::RANGE)
            },


            Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.expr(path, scope, lhs);
                let rhs_anal = self.expr(path, scope, rhs);

                lhs_anal.ty.eq(&mut self.syms, rhs_anal.ty);

                let lhs_sym = lhs_anal.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;

                if lhs_anal.ty.is_err(&mut self.syms) { return Ok(AnalysisResult::new(lhs_anal.ty)) }
                if lhs_sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                let rhs_sym = rhs_anal.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;

                if rhs_anal.ty.is_err(&mut self.syms) { return Ok(AnalysisResult::new(rhs_anal.ty)) }
                if rhs_sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                let mut validate = || {
                    if !lhs_anal.ty.eq(&mut self.syms, rhs_anal.ty) { return Ok(false) }

                    let sym = match lhs_anal.ty.sym(&mut self.syms) {
                        Ok(v) => v,
                        Err(v) => return Err(v),
                    };

                    Ok(if operator.is_arith() { sym.supports_arith() } else { true }
                    && if operator.is_bw() { sym.supports_bw() } else { true }
                    && if operator.is_ocomp() { sym.supports_ord() } else { true }
                    && if operator.is_ecomp() { sym.supports_eq() } else { true })
                };


                let validate = validate().map_err(|e| self.error(id, e))?;

                if validate {
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

                    return Ok(AnalysisResult::new(result))
                }



                let can_trait =
                   lhs_anal.ty.eq(&mut self.syms, rhs_anal.ty)
                && operator.is_ecomp();


                if !can_trait {
                    return Err(self.error(id, Error::InvalidBinaryOp {
                        operator, lhs: lhs_anal.ty, rhs: rhs_anal.ty, source: range }));

                }


                let lhs_sym = lhs_anal.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                let traits = self.syms.traits(lhs_sym);

                if traits.contains_key(&SymbolId::EQ_TRAIT) {
                    return Ok(AnalysisResult::new(Type::BOOL));
                }

                return Err(self.error(id, Error::TypeDoesntImplTrait { source: range, ty: lhs_anal.ty, tr: SymbolId::EQ_TRAIT }));
            },


            Expr::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(path, scope, rhs);
                let sym = rhs_anal.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;

                if rhs_anal.ty.is_err(&mut self.syms) { return Ok(AnalysisResult::new(rhs_anal.ty)) }
                if sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                match operator {
                    UnaryOperator::Not if sym == SymbolId::BOOL => (),
                    UnaryOperator::Neg if sym.is_num() && sym != SymbolId::BYTE => (),
                    
                    _ => return Err(self.error(id, Error::InvalidUnaryOp { operator, rhs: rhs_anal.ty, source: range }))
                }

                AnalysisResult::new(rhs_anal.ty)
            },


            Expr::If { condition, body, else_block } => {
                let cond = self.expr(path, scope, condition);

                if let Ok(sym) = cond.ty.sym(&mut self.syms) {
                    if self.syms.is_err_sym(sym) { return Ok(AnalysisResult::new(cond.ty)) }
                    if sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }
                }

                if !cond.ty.eq(&mut self.syms, Type::BOOL) {
                    let range = self.ast.range(condition);
                    return Err(self.error(id, Error::InvalidType {
                        source: range, found: cond.ty, expected: Type::BOOL }));
                }

                let body_anal = self.expr(path, scope, body);
                let mut value = body_anal.ty;

                (|| {
                    let Some(el) = else_block
                    else { return };

                    let el_anal = self.expr(path, scope, el);

                    if value.is_err(&mut self.syms) || value.is_never(&mut self.syms) {
                        value = el_anal.ty
                    } else if el_anal.ty.ne(&mut self.syms, value) {
                        let body = self.ast.range(body);
                        let else_block = self.ast.range(el);
                        self.error(el, Error::IfBodyAndElseMismatch {
                            body: (body, value), else_block: (else_block, el_anal.ty) });
                        return
                    }
                })();

                if else_block.is_none() && (value.is_err(&mut self.syms) | value.is_never(&mut self.syms)) {
                    value = Type::UNIT;
                }

                if value.ne(&mut self.syms, Type::UNIT) && else_block.is_none() {
                    let body = self.ast.range(body);
                    return Err(self.error(id, Error::IfMissingElse { body: (body, value) }))
                }

                AnalysisResult::new(value)
            },


            Expr::Match { value, mappings  } => {
                let anal = self.expr(path, scope, value);

                let sym = anal.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;

                if let Some(err) = anal.ty.as_err(&mut self.syms) {
                    return Err(err);
                }

                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else {
                    let range = self.ast.range(value);
                    return Err(self.error(id, Error::MatchValueIsntEnum { source: range, typ: anal.ty }));
                };

                // check if the value is an enum
                if !matches!(cont.kind(), ContainerKind::Enum) {
                    let range = self.ast.range(value);
                    return Err(self.error(id, Error::MatchValueIsntEnum { source: range, typ: anal.ty }));
                }

                // check the mapping names
                for (i, m) in mappings.iter().enumerate() {
                    let exists = cont.fields().iter().any(|x| {
                        let name = x.0;
                        m.variant() == name
                    });

                    if !exists {
                        return Err(self.error(id, Error::InvalidMatch {
                            name: m.variant(), range: m.range(), value: anal.ty }));
                    }

                    for o in mappings.iter().skip(i+1) {
                        if o.variant() == m.variant() {
                            return Err(self.error(id, Error::DuplicateMatch {
                                declared_at: m.range(), error_point: o.range() }));
                        }
                    }
                }

                
                let mut missings = Vec::new_in(self.temp);
                for sm in cont.fields().iter() {
                    let name = sm.0;
                    if !mappings.iter().any(|x| x.variant() == name) {
                        missings.push(name);
                    }
                }

                if !missings.is_empty() {
                    return Err(self.error(id, Error::MissingMatch { name: KVec::from_slice(&missings), range }));
                }


                // ty chck
                let ret_ty = self.syms.new_var(id, StringMap::RESULT, range);
                let mut errored = None;
                let mut has_value_branch = false;
                for (m, f) in mappings.iter().zip(cont.fields().iter()) {
                    let gens = anal.ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);
                    let vs = VariableScope::new(m.binding(), f.1.to_ty(gens, &mut self.syms), true);

                    let scope = Scope::new(Some(scope), ScopeKind::VariableScope(vs));
                    let scope = self.scopes.push(scope);

                    let anal = self.expr(path, scope, m.expr());
                    if anal.ty.is_err(&mut self.syms) {
                        if errored.is_none() {
                            errored = anal.ty.as_err(&mut self.syms);
                        }
                        continue;
                    }

                    if anal.ty.is_never(&mut self.syms) {
                        continue;
                    }

                    has_value_branch = true;
                    if !anal.ty.eq(&mut self.syms, ret_ty) {
                        let range = self.ast.range(m.expr());
                        self.error(m.expr(), Error::InvalidType {
                            source: range, found: anal.ty, expected: ret_ty });
                    }
                }

                if !has_value_branch {
                    let ty = 
                    if let Some(err) = errored {
                        self.error_type(err)
                    } else {
                        Type::NEVER
                    };

                    ret_ty.eq(&mut self.syms, ty);
                }

                AnalysisResult::new(ret_ty)
            },


            Expr::Block { .. } => self.eval_block(self.blocks.block_id(NodeId::Expr(id)), scope),


            Expr::CreateStruct { data_type, fields  } => {
                let ty = self.dt_to_ty(scope, id, data_type);
                if let Some(err) = ty.as_err(&mut self.syms) { return Err(err); }

                let sym = ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else { return Err(self.error(id, Error::StructCreationOnNonStruct { source: range, typ: ty })) };

                // check if the sym is a struct
                if !matches!(cont.kind(), ContainerKind::Struct) {
                    return Err(self.error(id, Error::StructCreationOnNonStruct { source: range, typ: ty }));
                }

                // check if the fields are valid
                for f in fields.iter() {
                    let exists = cont.fields().iter().any(|x| {
                        let name = x.0;
                        name == f.0
                    });

                    if !exists {
                        return Err(self.error(id, Error::FieldDoesntExist {
                            source: f.1, field: f.0, typ: ty, suggested: None }));
                    }
                }


                // check missing fields; an errored field is optional (its slot
                // is zeroed at codegen, and the field is unusable anyway)
                let mut missing_fields = Vec::new_in(self.temp);
                for f in cont.fields().iter() {
                    let name = f.0;

                    if !fields.iter().any(|x| x.0 == name) {
                        let gens = ty.gens(&mut self.syms);
                        let gens = self.syms.get_gens(gens);
                        if !f.1.to_ty(gens, &mut self.syms).is_err(&mut self.syms) {
                            missing_fields.push(name);
                        }
                    }
                }

                if !missing_fields.is_empty() {
                    return Err(self.error(id, Error::MissingFields { source: range, fields: missing_fields.clone_in(GlobalAlloc) }));
                }


                // type check the fields
                let sym_fields = {
                    let mut vec = Buffer::new(self.temp, cont.fields().len());
                    let gens = ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);

                    for f in cont.fields() {
                        vec.push((f.0, f.1.to_ty(gens, &mut self.syms)))
                    }

                    vec
                };


                //dbg!(sym);
                for f in fields.iter() {
                    let expr = self.expr(path, scope, f.2);
                    let g = sym_fields.iter().find(|x| x.0 == f.0).unwrap();
                    //dbg!(g);
                    //dbg!(expr);

                    if !expr.ty.eq(&mut self.syms, g.1) {
                        self.error(f.2, Error::InvalidType {
                            source: f.1, found: expr.ty, expected: g.1 });
                    }
                }

                AnalysisResult::new(ty)
            },


            Expr::AccessField { val, field_name, gens: expr_gens } => {
                let expr = self.expr(path, scope, val);

                if expr.ty.is_err(&mut self.syms) {
                    return Ok(AnalysisResult::new(expr.ty))
                }

                let sym_id = expr.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                let sym = self.syms.sym(sym_id);

                let field_check = 'b: {
                    let err = Error::FieldDoesntExist {
                        source: range, field: field_name, typ: expr.ty, suggested: None };
                    let SymbolKind::Container(cont) = sym.kind()
                    else { break 'b Err(err) };

                    let field = cont.fields().iter().enumerate().find(|(_, f)| {
                        let name = f.0;
                        field_name == name
                    });

                    let Some((_, field)) = field
                    else { break 'b Err(err)  };

                    Ok((field, cont))
                };

                // if its a normal field
                let e = 
                match field_check {
                    Ok((field, cont)) => {
                        let gens = expr.ty.gens(&mut self.syms);
                        let gens = self.syms.get_gens(gens);

                        let field_gen = field.1;
                        let field_ty = field_gen.to_ty(gens, &mut self.syms);

                        let ty = 
                        match cont.kind() {
                            ContainerKind::Struct => field_ty,

                            ContainerKind::Enum => {
                                let gens = self.output.alloc_new([(BoundedGeneric::T, field_ty)]);
                                Type::Ty(SymbolId::OPTION, self.syms.add_gens(gens))
                            },

                            ContainerKind::Tuple => field_ty,

                            ContainerKind::Generic => unreachable!(),
                        };

                        return Ok(AnalysisResult {
                            ty,
                            is_mut: expr.is_mut,
                            is_captured: expr.is_captured,
                        })
                    },


                    Err(e) => e,
                };



                let ns = self.syms.sym_ns(sym_id);
                let ns = self.namespaces.get_ns(ns);
                if let Some(sym) = ns.get_sym(field_name) {
                    let sym_id = sym?;

                    let sym = self.syms.sym(sym_id);

                    let mut vgens = sti::vec::Vec::with_cap_in(self.output, sym.generics().iter().len());

                    for g in sym.generics().iter() {
                        let var = self.syms.new_var(id, g.name, range);
                        vgens.push((*g, var));
                    }
                    
                    let expr_type_gens = expr.ty.gens(&mut self.syms);
                    let sym_gens = self.syms.get_gens(expr_type_gens);

                    //assert!(sym_gens.iter().zip(&vgens).all(|(a, b)| a.0 == b.1.0));

                    for ((n0, g0), (_, (n1, g1))) in sym_gens.iter().zip(&vgens) {
                        if n0 == n1 {
                            (*g0).eq(&mut self.syms, *g1);
                        }
                    }



                    if let Some(gens) = expr_gens {
                        for (g, (_, s)) in gens.iter().zip(vgens.iter().skip(sym_gens.len())) {
                            let ty = self.dt_to_ty(scope, id, *g);
                            if ty.is_err(&mut self.syms) { continue; }

                            if !ty.eq(&mut self.syms, *s) {
                                self.error(id, Error::InvalidType { source: range, found: *s, expected: ty });
                            }
                        }
                    }

                    let gens = self.syms.add_gens(vgens.leak());

                    let SymbolKind::Function(func) = sym.kind()
                    else { return Err(self.error(id, Error::CallOnNonFunction { source: range })) };


                    let anal = match func.kind() {
                        FunctionKind::Closure(_) => AnalysisResult::new(Type::Ty(sym_id, expr.ty.gens(&mut self.syms))),
                        _ => {
                            let closure = self.syms.new_closure();

                            let sym = self.func_sym(
                                closure,
                                func.args(),
                                func.ret(),
                                sym.generics(),
                                func.declared_generics()
                            );

                            AnalysisResult::new(Type::Ty(sym, gens))
                        }
                    };

                    return Ok(anal);
                }

                let candidate = self.find_trait_method_candidate(scope, id, range, Some(expr.ty), sym_id, field_name)?;

                let Some((t, trait_ty, func, g, impl_generics)) = candidate
                else {
                    let e = match e {
                        Error::FieldDoesntExist { source, field, typ, .. } => Error::FieldDoesntExist {
                            source, field, typ,
                            suggested: self.suggest_trait_imports(sym_id, expr.ty, field_name),
                        },
                        e => e,
                    };

                    return Err(self.error(id, e));
                };
                let (sym, all_generics) =
                    self.trait_method_sym(t, trait_ty, func, g, impl_generics);
                let mut vgens = sti::vec::Vec::with_cap_in(self.output, all_generics.len());
                for generic in all_generics {
                    let var = self.syms.new_var(id, generic.name, range);
                    vgens.push((*generic, var));
                }

                let gens = self.syms.add_gens(vgens.leak());
                let implementation_ty = g.to_ty(self.syms.get_gens(gens), &mut self.syms);
                assert!(expr.ty.eq(&mut self.syms, implementation_ty));

                if let Some(explicit_gens) = expr_gens {
                    let resolved_gens = self.syms.get_gens(gens);
                    for (g, (_, s)) in explicit_gens.iter().zip(
                        resolved_gens.iter().skip(impl_generics.len())
                    ) {
                        let ty = self.dt_to_ty(scope, id, *g);
                        if ty.is_err(&mut self.syms) { continue; }

                        if !ty.eq(&mut self.syms, *s) {
                            self.error(id, Error::InvalidType { source: range, found: *s, expected: ty });
                        }
                    }
                }

                self.type_info.set_acc(id, trait_ty);

                AnalysisResult::new(Type::Ty(sym, gens))
            },


            Expr::CallFunction { lhs: lhs_expr, args } => {
                let lhs = self.expr(path, scope, lhs_expr);
                let lhs_range = self.ast.range(lhs_expr);
                
                let pool = self.ast.arena;
                let mut is_accessor = false;
                let args_anals = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, args.len());
                    let mut err = Ok(());

                    if let Expr::AccessField { val, field_name, .. } = self.ast.expr(lhs_expr) {
                        let range = self.ast.range(val);
                        let anal = self.expr(path, scope, val);

                        // check if it's a field or not
                        let sym = anal.ty;
                        let sym = sym.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                        let sym = self.syms.sym(sym);

                        if let SymbolKind::Container(cont) = sym.kind()
                        && cont.fields().iter().find(|x| x.0 == field_name).is_some() {
                            err = Err(self.error(id, Error::CallOnField { source: lhs_range, field_name }))
                        } else {
                            is_accessor = true;
                            vec.push((range, anal, val, false));
                        }
                    }

                    for a in args {
                        let anal = self.expr(path, scope, a.expr);
                        vec.push((self.ast.range(a.expr), anal, a.expr, a.is_inout));
                    }

                    let _ = err?;

                    vec.leak()
                };


                if lhs.ty.is_err(&mut self.syms) {
                    return Ok(AnalysisResult::new(lhs.ty))
                }

                let sym_id = lhs.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;


                let sym = self.syms.sym(sym_id);
                let SymbolKind::Function(func) = sym.kind()
                else { return Err(self.error(id, Error::CallOnNonFunction { source: lhs_range })); };
                let f_gens = lhs.ty.gens(&mut self.syms);
                let gens = self.syms.get_gens(f_gens);

                // check arg len
                if func.args().len() != args_anals.len() {
                    return Err(self.error(id, Error::FunctionArgsMismatch {
                        source: range, sig_len: func.args().len(), call_len: args.len() + if is_accessor { 1 } else { 0 } }));
                }

                // find out the args
                let func_args = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, func.args().len());
                    for g in func.args() {
                        vec.push((g.symbol().to_ty(gens, &mut self.syms), g.is_inout()));
                    }

                    vec
                };

                let ret = func.ret().to_ty(gens, &mut self.syms);

                // ty check args
                for (i, ((source, anal, expr, explicit_inout), (fa, formal_inout))) in args_anals.iter().copied().zip(func_args.iter().copied()).enumerate() {
                    if !anal.ty.eq(&mut self.syms, fa) {
                        self.error(expr, Error::InvalidType {
                            source, found: anal.ty, expected: fa });
                    }

                    let is_inout = explicit_inout || (formal_inout && is_accessor && i == 0);
                    if is_inout && !formal_inout {
                        self.error(expr, Error::InOutValueWithoutInOutBinding { source });
                    } else if formal_inout && !is_inout {
                        self.error(expr, Error::InOutBindingWithoutInOutValue { source });
                    } else if is_inout && anal.is_captured && self.is_assignable_place(expr) {
                        self.error(expr, Error::CannotMutateCapturedValue { source });
                    } else if is_inout && (!anal.is_mut || !self.is_assignable_place(expr)) {
                        self.error(expr, Error::InOutValueIsNotAssignable { source });
                    }
                }

                for (sym_g, (func_g, value)) in sym.generics().iter().zip(gens.iter()) {
                    assert_eq!(sym_g.name(), func_g.name());

                    if sym_g.bounds.is_empty() { continue }

                    for bound in sym_g.bounds {
                        let Some(trait_id) = bound.sym()
                        else { continue };

                        if value.is_err(&mut self.syms)
                        || value.is_never(&mut self.syms) {
                            return Ok(AnalysisResult::new(*value))
                        }

                        if let SymbolKind::Error(error) = self.syms.sym(trait_id).kind() {
                            return Ok(AnalysisResult::new(self.error_type(error)))
                        }

                        let bound_trait_ty = bound.to_ty(gens, &mut self.syms);
                        if self.syms.type_implements_trait_generic(*value, bound_trait_ty) { continue }

                        let err = self.error(
                            lhs_expr,
                            Error::TypeDoesntImplTrait { 
                                source: range, ty: *value, tr: trait_id }
                        );
                        
                        return Ok(AnalysisResult::new(self.error_type(err)))
                    }
                }

                self.type_info.set_func_call(id, (sym_id, f_gens));
                AnalysisResult::new(ret)
            },


            Expr::WithinNamespace { namespace, namespace_source, action  } => {
                let sym = self.scopes.get(scope).find_sym(
                    namespace, &self.scopes, 
                    &mut self.syms, &self.namespaces
                );

                let sym = self.convert_symbol_get_result(id, namespace, namespace_source, sym);
                let gen_count = self.syms.sym_gens_size(sym);
                let mut generics = Buffer::new(&*self.output, gen_count);
                for index in 0..gen_count {
                    let name = self.syms.sym(sym).generics()[index].name();
                    generics.push(self.syms.new_var(id, name, namespace_source));
                }

                let ty = self.syms.get_ty(sym, &*generics);
                let sym = ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                if let SymbolKind::Error(err) = self.syms.sym(sym).kind() {
                    return Err(err);
                }
                let scope = Scope::new(scope, ScopeKind::QualifiedTypeNamespace(ty, None));
                let scope = self.scopes.push(scope);
                let scope = Scope::new(scope, ScopeKind::ImplicitTrait(sym));
                let scope = self.scopes.push(scope);

                self.expr(path, scope, action)
            },

            Expr::WithinTypeNamespace { namespace, action  } => {
                let ty = self.dt_to_ty(scope, id, namespace);
                let sym = ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                if let SymbolKind::Error(err) = self.syms.sym(sym).kind() {
                    return Err(err);
                }

                self.expr(path, scope, action)
            },


            Expr::Loop { body: _ } => {
                let scope = Scope::new(Some(scope), ScopeKind::Loop);
                let scope = self.scopes.push(scope);
                self.control_flow.enter_loop();
                self.eval_block(self.blocks.block_id(NodeId::Expr(id)), scope);

                let loop_context = self.control_flow.exit_loop();
                if loop_context.has_break {
                    AnalysisResult::new(Type::UNIT)
                } else {
                    AnalysisResult::new(Type::NEVER)
                }
            },


            Expr::Return(ret) => {
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return Err(self.error(id, Error::OutsideOfAFunction { source: range })) };
                let ret_anal = self.expr(path, scope, ret);
                if ret_anal.ty.is_err(&mut self.syms) { return Ok(AnalysisResult::new(ret_anal.ty)) }
                if ret_anal.ty.is_never(&mut self.syms) { return Ok(AnalysisResult::never()) }

                if ret_anal.ty.ne(&mut self.syms, func.ret) {
                    return Err(self.error(id, Error::ReturnAndFuncTypDiffer {
                        source: range, func_source: func.ret_source,
                        typ: ret_anal.ty, func_typ: func.ret }));
                }

                AnalysisResult::new(Type::NEVER)
            },


            Expr::Continue => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none() { 
                    return Err(self.error(id, Error::ContinueOutsideOfLoop(range)))
                }

                AnalysisResult::new(Type::NEVER)
            },


            Expr::Break => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none()
                || !self.control_flow.mark_break() {
                    return Err(self.error(id, Error::BreakOutsideOfLoop(range)))
                }

                AnalysisResult::new(Type::NEVER)
            },


            Expr::Tuple(values) => {
                let pool = self.ast.arena;

                let fields = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, values.len());
                    for _ in 0..values.len() {
                        vec.push(None);
                    }

                    vec.leak()
                };

                let sym = self.tuple_sym(range, fields);

                let gens = {
                    let mut vec = sti::vec::Vec::with_cap_in(self.output, values.len());
                    for (index, value) in values.iter().enumerate() {
                        let str = self.string_map.num(index);
                        let str = BoundedGeneric::new(str, &[]);
                        let ty = self.expr(path, scope, *value);
                        vec.push((str, ty.ty));
                    }

                    vec.leak()
                };

                let gens = self.syms.add_gens(gens);

                AnalysisResult::new(Type::Ty(sym, gens))
            },


            Expr::CreateList { exprs } => {
                let ty = self.syms.new_var(id, None, range);

                let mut errored = None;
                for e in exprs {
                    let expr = self.expr(path, scope, *e);
                    if !ty.eq(&mut self.syms, expr.ty) {
                        let range = self.ast.range(*e);
                        let e = self.error(*e, Error::InvalidType { source: range, found: expr.ty, expected: ty });
                        if errored.is_none() {
                            errored = Some(e);
                        }
                    }
                }

                let gens = self.syms.add_gens(self.output.alloc_new([(BoundedGeneric::T, ty)]));
                AnalysisResult::new(Type::Ty(SymbolId::LIST, gens))
            }


            Expr::IndexList { list, index } => {
                let list = self.expr(path, scope, list);
                let index = self.expr(path, scope, index);

                let sym = list.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;

                if sym == SymbolId::NEVER || self.syms.is_err_sym(sym) { return Ok(AnalysisResult::new(list.ty)) }

                if sym != SymbolId::LIST {
                    return Err(self.error(id, Error::IndexOnNonList(range, list.ty)));
                }

                if !index.ty.is_int(&mut self.syms) {
                    return Err(self.error(id, Error::InvalidType { source: range, found: index.ty, expected: Type::I64 }))
                }

                let gens = list.ty.gens(&mut self.syms);
                let (_, ty) = self.syms.get_gens(gens)[0];

                AnalysisResult {
                    ty,
                    is_mut: list.is_mut,
                    is_captured: list.is_captured,
                }
            },


            Expr::AsCast { lhs, data_type  } => {
                let anal = self.expr(path, scope, lhs);
                let ty = self.dt_to_ty(scope, id, data_type);

                if anal.ty.eq(&mut self.syms, ty) {
                    return Ok(AnalysisResult::new(ty))
                }

                if anal.ty.is_err(&mut self.syms)
                    || anal.ty.is_never(&mut self.syms)
                    || ty.is_err(&mut self.syms)
                    || ty.is_never(&mut self.syms)
                    || anal.ty.eq(&mut self.syms, ty) {
                    return Ok(AnalysisResult::new(ty))
                }

                match (anal.ty.sym(&mut self.syms), ty.sym(&mut self.syms)) {
                    (Ok(SymbolId::BOOL), Ok(SymbolId::I64)) => (),
                    _ => {
                        if !(anal.ty.is_num(&mut self.syms) && ty.is_num(&mut self.syms)) {
                            self.error(id, Error::InvalidCast {
                                range, from_ty: anal.ty, to_ty: ty });
                        }


                    }
                }
                AnalysisResult::new(ty)
            },


            Expr::Unwrap(val) => {
                let expr = self.expr(path, scope, val);
                let sym = expr.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                if self.syms.is_err_sym(sym) { return Ok(AnalysisResult::new(expr.ty)) }

                if sym != SymbolId::OPTION
                   && sym != SymbolId::RESULT {
                    return Err(self.error(id, Error::CantUnwrapOnGivenType(range, expr.ty)));
                }

                let gens = expr.ty.gens(&mut self.syms);
                let gens = self.syms.get_gens(gens);
                
                AnalysisResult {
                    ty: gens[0].1,
                    is_mut: expr.is_mut,
                    is_captured: expr.is_captured,
                }
            },


            Expr::OrReturn(val) => {
                let expr = self.expr(path, scope, val);
                let sym = expr.ty.sym(&mut self.syms).map_err(|e| self.error(id, e))?;
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return Err(self.error(id, Error::OutsideOfAFunction { source: range })) };
                if let Some(err) = expr.ty.as_err(&mut self.syms) {
                    return Err(err);
                }

                if sym == SymbolId::OPTION {
                    let func_sym = func.ret;
                    let opt_sym = {
                        let val = self.syms.new_var(id, StringMap::T, range);
                        let gens = self.output.alloc_new([(BoundedGeneric::T, val)]);
                        let gens = self.syms.add_gens(gens);

                        Type::Ty(SymbolId::OPTION, gens)
                    };

                    if !opt_sym.eq(&mut self.syms, func_sym) {
                        return Err(self.error(id, Error::FunctionDoesntReturnAnOption { source: range, func_typ: func.ret }));
                    }

                    let gens = expr.ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);

                    return Ok(AnalysisResult {
                        ty: gens[0].1,
                        is_mut: expr.is_mut,
                        is_captured: expr.is_captured,
                    });
                }

                
                if sym == SymbolId::RESULT {
                    let res_sym = {
                        let ok = self.syms.new_var(id, StringMap::T, range);
                        let err = self.syms.new_var(id, StringMap::A, range);
                        let gens = self.output.alloc_new([(BoundedGeneric::T, ok), (BoundedGeneric::A, err)]);
                        let gens = self.syms.add_gens(gens);

                        Type::Ty(SymbolId::RESULT, gens)
                    };

                    if !res_sym.eq(&mut self.syms, func.ret) {
                        return Err(self.error(id, Error::FunctionDoesntReturnAResult { source: range, func_typ: func.ret }));
                    }

                    let func_gens = func.ret.gens(&mut self.syms);
                    let func_gens = self.syms.get_gens(func_gens);

                    let gens = expr.ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);

                    debug_assert_eq!(func_gens.len(), 2);
                    debug_assert_eq!(gens.len(), 2);

                    if !func_gens[1].1.eq(&mut self.syms, gens[1].1) {
                        return Err(self.error(id, Error::FunctionReturnsAResultButTheErrIsntTheSame {
                            source: range, func_source: func.ret_source,
                            func_err_typ: func_gens[1].1, err_typ: gens[1].1 }));
                    }

                    return Ok(AnalysisResult {
                        ty: gens[0].1,
                        is_mut: expr.is_mut,
                        is_captured: expr.is_captured,
                    });
                }


                return Err(self.error(id, Error::CantTryOnGivenType(range, expr.ty)));


            },
        })})();


        match result {
            Ok(v) => {
                let already_err = self.type_info.exprs[id]
                    .map(|info| info.ty.is_err(&mut self.syms))
                    .unwrap_or(false);
                if !already_err {
                    self.type_info.set_expr(id, v.ty);
                }
                v
            },

            Err(v) => {
                self.set_error(id, v);
                AnalysisResult::new(self.error_type(v))
            },
        }
    }


    fn suggest_trait_imports(
        &mut self,
        sym_id: SymbolId,
        ty: Type,
        name: StringIndex,
    ) -> Option<sti::vec::Vec<StringIndex>> {
        let candidates = self.syms.traits(sym_id).clone();
        let mut suggested = std::vec::Vec::new();
        for (trait_id, impls) in candidates {
            for entry in impls {
                if entry.namespace == NamespaceId::MAX { continue; }

                let SymbolKind::Trait(tr) = self.syms.sym(trait_id).kind()
                else { continue; };

                if !tr.funcs.iter().any(|x| x.0 == name) {
                    continue;
                }

                let mut bindings = std::vec::Vec::with_capacity(entry.generics.len());
                if !self.syms.match_impl_type(
                    entry.receiver,
                    ty,
                    entry.generics,
                    &mut bindings,
                ) {
                    continue;
                }
                let trait_ty = entry.trait_ty.to_ty(&bindings, &mut self.syms);
                if !self.syms.type_implements_trait_generic(ty, trait_ty) {
                    continue;
                }

                suggested.push(trait_id);
            }
        }

        if suggested.is_empty() {
            return None;
        }

        suggested.sort_by(|a, b| {
            let a = self.string_map.get(self.syms.sym(*a).name());
            let b = self.string_map.get(self.syms.sym(*b).name());
            a.cmp(b)
        });

        let mut qualified = std::vec::Vec::with_capacity(suggested.len());
        for id in suggested {
            let trait_ns = self.syms.sym_ns(id);
            let qualified_name = self.namespaces.get_ns(trait_ns).path;
            qualified.push(qualified_name);
        }

        Some(sti::vec::Vec::from_slice_in(GlobalAlloc, &qualified))
    }
}
