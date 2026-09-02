use super::*;

impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub fn collect_names(&mut self, path: StringIndex, ns_id: NamespaceId, nodes: &[NodeId], gen_count: usize) {
        for n in nodes {
            let NodeId::Decl(decl) = n
            else { continue };

            let id = *decl;
            let mut ns = self.namespaces.get_ns_mut(ns_id);
            let decl = self.ast.decl(id);
            let range = self.ast.range(*n);
            match decl {
                | Decl::Enum { visibility, name, header, generics, .. } 
                | Decl::Struct { visibility, name, header, generics, .. }
                | Decl::Alias { visibility, name, header, gens: generics, .. }
                | Decl::Trait { visibility, name, header, generics, .. }
                | Decl::Function { visibility, sig: FunctionSignature { name, source: header, generics, .. }, .. }=> {
                    if let Some(sym) = ns.get_sym(name) {
                        let err = Error::NameIsAlreadyDefined {
                            source: header, name };
                        let err = self.error(*n, err);
                        ns = self.namespaces.get_ns_mut(ns_id);

                        if sym.is_ok() { ns.set_err_sym(name, err) }

                        continue
                    }

                    if matches!(name, StringMap::ITER_NEXT_FUNC)
                        && !matches!(decl, Decl::Function { .. }) {
                        self.error(*n, Error::NameIsReservedForFunctions { source: header });
                    }

                    let path = self.string_map.concat(path, name);
                    let pend = self.syms.pending(&mut self.namespaces, Some(ns_id), path, generics.len() + gen_count);
                    ns = self.namespaces.get_ns_mut(ns_id);

                    let result = ns.add_sym(
                        &mut self.errors, *n, range, name,
                        pend, visibility
                    );

                    if let Err(e) = result {
                        self.set_error(id, e);
                    }
                },




                Decl::Extern { functions, .. } => {
                    for f in functions {
                        if let Some(sym) = ns.get_sym(f.name()) {
                            let err = Error::NameIsAlreadyDefined {
                                source: f.range(), name: f.name() };

                            let err = self.error(*n, err);
                            ns = self.namespaces.get_ns_mut(ns_id);

                            if sym.is_ok() { ns.set_err_sym(f.name(), err) }

                            continue
                        }

                        let path = self.string_map.concat(path, f.name());
                        let pend = self.syms.pending(&mut self.namespaces, Some(ns_id), path, f.gens().len());
                        ns = self.namespaces.get_ns_mut(ns_id);


                        let result = ns.add_sym(
                            &mut self.errors, *n, range, f.name(),
                            pend, f.visibility()
                        );

                        if let Err(e) = result {
                            self.set_error(id, e);
                        }

                        ns = self.namespaces.get_ns_mut(ns_id);

                    }
                },


                Decl::Module { visibility, name, header, is_root, .. } => {
                    let path = self.string_map.concat(path, name);

                    let sym = self.syms.pending(&mut self.namespaces, Some(ns_id), path, 0);
                    self.syms.add_sym(sym, Symbol::new(name, &[], SymbolKind::Namespace));

                    let module_ns = self.syms.as_ns(sym);

                    {
                        let module_ns = self.namespaces.get_ns_mut(module_ns);

                        if is_root {
                            module_ns.add_sym(
                                &mut self.errors, *n, header, StringMap::ROOT,
                                sym, Visibility::Public
                            ).unwrap();
                        }
                    }
                    

                    let ns = self.namespaces.get_ns_mut(ns_id);
                    let result = ns.add_sym(
                        &mut self.errors, *n, header, name,
                        sym, visibility,
                    );

                    if let Err(e) = result {
                        self.set_error(id, e);
                    }


                },


                Decl::Attribute { attr, decl } => {
                    if matches!(attr.identifier(), Some(name) if self.string_map.get(name) == "silent") {
                        self.silent_ranges.push(self.ast.range(decl));
                    }
                    self.collect_names(path, ns_id, &[decl.into()], gen_count);
                },


                _ => (),
            }
        }
    }


    pub fn collect_impls(&mut self, path: StringIndex, scope: ScopeId, ns_id: NamespaceId, nodes: &[NodeId]) {
        for &n in nodes {
            let NodeId::Decl(id) = n
            else { continue };

            let decl = self.ast.decl(id);
            match decl {
                Decl::Module { .. } => (),


                Decl::Impl { data_type, gens, body } => {
                    let s = self.scopes.get(scope);
                    let gens =
                    match self.resolve_generics(scope, n, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.set_error(id, v);
                            continue;
                        },
                    };

                    let ty = self.dt_to_gen(id, s, data_type, gens);

                    let source = self.ast.range(n);
                    self.type_info.impls.insert(id, (ty, ty, gens));

                    let Some(sym) = ty.sym()
                    else {
                        self.error(n, Error::ImplOnGeneric(source));
                        continue;
                    };

                    let ns = self.syms.sym_ns(sym);

                    let path = self.namespaces.get_ns(ns).path;

                    self.collect_names(path, ns, &body, gens.len());
                    self.collect_impls(path, scope, ns, &body);
                }

                Decl::ImplTrait { trait_name, data_type, gens, body, .. } => {
                    let gens = match self.resolve_generics(scope, n, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.set_error(id, v);
                            continue;
                        },
                    };

                    let s = self.scopes.get(scope);
                    let trait_ty = self.dt_to_gen(id, s, trait_name, gens);

                    let ty = self.dt_to_gen(id, s, data_type, gens);

                    self.type_info.impls.insert(id, (trait_ty, ty, gens));

                    let source = self.ast.range(n);
                    let Some(trait_sym_id) = trait_ty.sym()
                    else {
                        self.error(n, Error::ImplOnGeneric(source));
                        return;
                    };

                    let Some(sym) = ty.sym()
                    else {
                        self.error(n, Error::ImplOnGeneric(source));
                        return;
                    };

                    let ns = Namespace::new(path);
                    let ns = self.namespaces.push(ns, Some(ns_id));
                    self.collect_names(path, ns, &*body, gens.len());

                    self.syms.traits(sym).entry(trait_sym_id).or_default().push(TraitImplEntry {
                        namespace: ns,
                        trait_ty,
                        receiver: ty,
                        generics: gens,
                        declaration: Some(id),
                        bound_error: None,
                    });
                },


                Decl::Attribute { decl, .. } => {
                    self.collect_impls(path, scope, ns_id, &[decl.into()]);
                },

                _ => (),
            }
        }
    }

    pub fn collect_uses(&mut self, scope_id: ScopeId, ns_id: NamespaceId, nodes: &[NodeId]) {
        let scope = self.scopes.get(scope_id);

        for n in nodes.iter().rev() {
            let NodeId::Decl(id) = *n
            else { continue; };

            if matches!(self.ast.decl(id), Decl::Module { .. }) {
                self.ensure_uses(self.blocks.block_id(NodeId::Decl(id)));
            }
        }

        for n in nodes {
            let NodeId::Decl(id) = *n
            else { continue; };

            match self.ast.decl(id) {
                Decl::Module { .. } => continue,

                Decl::Impl { body, .. } => {
                    let Some((ty, _, _)) = self.type_info.impls.get(&id)
                    else { continue };

                    let Some(sym) = ty.sym()
                    else { continue; };

                    let ns = self.syms.sym_ns(sym);
                    self.collect_uses(scope_id, ns, &body)
                }

                Decl::Using { visibility, item } => {
                    self.collect_use_item(*n, scope, ns_id, item, visibility)
                }

                Decl::Attribute { decl, .. } => {
                    self.collect_uses(scope_id, ns_id, &[decl.into()]);
                },

                _ => continue,
            }
        }
    }


    fn collect_use_item(&mut self, node: NodeId, scope: Scope, ns_id: NamespaceId, item: UseItem, visibility: Visibility) {
        let NodeId::Decl(_) = node
        else { return };

        match item.kind() {
            UseItemKind::List { list } => {
                let result = scope.find_sym_from(item.name(), &self.scopes, &mut self.syms, &self.namespaces, ns_id);
                let import_ns = self.convert_symbol_get_result(node, item.name(), item.range(), result);
                let import_ns = self.syms.sym_ns(import_ns);
                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(import_ns));
                for ui in list {
                    self.collect_use_item(node, scope, ns_id, *ui, visibility);
                }
            },

            UseItemKind::BringName(alias) => {
                let result = scope.find_sym_from(item.name(), &self.scopes, &mut self.syms, &self.namespaces, ns_id);
                let import_sym = self.convert_symbol_get_result(node, item.name(), item.range(), result);
                let ns = self.namespaces.get_ns_mut(ns_id);
                if let Err(e) = ns.add_sym(&mut self.errors, node, item.range(), alias, import_sym, visibility) {
                    ns.set_err_sym(item.name(), e);
                }
            },

            UseItemKind::All => {
                let result = scope.find_sym_from(item.name(), &self.scopes, &mut self.syms, &self.namespaces, ns_id);
                let import_ns = self.convert_symbol_get_result(node, item.name(), item.range(), result);
                let import_ns = self.syms.sym_ns(import_ns);
                if ns_id == import_ns { return };
                let (ns, import_ns) = self.namespaces.get_double(ns_id, import_ns);

                for s in import_ns.syms() {
                    if *s.0 == StringMap::ROOT { continue }
                    if s.1.visibility() == Visibility::Private {
                        continue;
                    }

                    if let Ok(v) = s.1.result() {
                        if let Err(e) = ns.add_sym(&mut self.errors, node, item.range(), *s.0, v, visibility) {
                            ns.set_err_sym(*s.0, e);
                        }
                    } else if let Err(e) = s.1.result() {
                        ns.set_err_sym(*s.0, e);
                    }
                }
            },
        };
    }
}
