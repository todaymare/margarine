use super::*;

impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    // `Self::compute_types` must be ran before this
    pub fn validate_types(&mut self, path: StringIndex, scope: ScopeId,
                         ns: NamespaceId, nodes: &[NodeId],
                         impl_block: Option<(SymbolId, &[BoundedGeneric<'out>], Option<StringIndex>)>) {
        for n in nodes {
            let NodeId::Decl(id) = n
            else { continue };

            let decl = self.ast.decl(*id);
            match decl {
                Decl::Alias { name, .. } => {
                    let Some(Ok(tsi)) = self.namespaces.get_ns(ns).get_sym(name)
                    else { continue };

                    let mut stack = vec![];
                    let mut item = tsi;

                    let is_recursive =
                    loop {
                        if stack.contains(&item) { break true }
                        stack.push(item);

                        match self.syms.sym(item).kind() {
                            SymbolKind::Alias(sym) => {
                                if let Some(sym) = sym.sym() {
                                    item = sym;
                                }
                            },

                            _ => break false,
                        }
                    };


                    if is_recursive {
                        let range = self.ast.range(*id);
                        let err = self.error(*id, Error::RecursiveAlias(range));
                        self.namespaces.get_ns_mut(ns).set_err_sym(name, err);
                        continue;
                    }
                }


                Decl::Module { .. } => (),


                Decl::Impl { body, .. } => {
                    let Some((ty, _, gens)) = self.type_info.impls.get(&id)
                    else { continue };

                    let Some(sym) = ty.sym()
                    else { continue; };


                    let ns = self.syms.sym_ns(sym);
                    let scope = self.scopes.push(Scope::new(scope, ScopeKind::AliasDecl(StringMap::SELF_TY, *ty)));

                    self.validate_types(path, scope, ns, &body, Some((sym, gens, None)));
                }



                Decl::Attribute { decl, .. } => {
                    self.validate_types(path, scope, ns, &[decl.into()], impl_block);
                },


                _ => (),

            }
        }
    }



    // `Self::collect_names` must be ran before this
    pub fn compute_types(&mut self, path: StringIndex, scope: ScopeId,
                         ns: NamespaceId, nodes: &[NodeId],
                         impl_block: Option<(SymbolId, &[BoundedGeneric<'out>], Option<StringIndex>)>) {
        for n in nodes {
            let NodeId::Decl(id) = n
            else { continue };

            let decl = self.ast.decl(*id);
            match decl {
                 Decl::Struct { name, fields, generics, .. } => {
                    let declared_generics =
                    match self.resolve_generics(scope, *n, generics) {
                        Ok(v) => v,
                        Err(e) => {
                            self.set_error(*id, e);
                            continue;
                        },
                    };
                    let impl_generics = impl_block.map(|(_, gens, _)| gens).unwrap_or(&[]);
                    let generics = self.blocks.all_generics(self.output, &self.scopes, scope, declared_generics, impl_generics);


                    let ns = self.namespaces.get_ns(ns);
                    let mut structure_fields = Buffer::new(self.output, fields.len());

                    let tsi = 
                    match ns.get_sym(name).unwrap() {
                        Ok(e) => e,
                        Err(e) => {
                            self.set_error(*id, e);
                            continue;
                        },
                    };

                    for f in fields {
                        let sym = self.dt_to_gen(*id, self.scopes.get(scope), f.1, generics);
                        let field = (f.0, sym);
                        structure_fields.push(field);
                    }

                    let sym_name = self.string_map.concat(path, name);
                    let cont = Container::new(structure_fields.leak(), ContainerKind::Struct);
                    let kind = SymbolKind::Container(cont);

                    let sym = Symbol::new(sym_name, generics, kind);
                    self.syms.add_sym(tsi, sym);
                },


                Decl::Alias { name, gens, data_type, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let Ok(tsi) = ns.get_sym(name).unwrap()
                    else { continue };

                    let declared_generics =
                    match self.resolve_generics(scope, *n, gens) {
                        Ok(v) => v,
                        Err(e) => {
                            self.set_error(*id, e);
                            continue;
                        },
                    };
                    let impl_generics = impl_block.map(|(_, gens, _)| gens).unwrap_or(&[]);
                    let generics = self.blocks.all_generics(self.output, &self.scopes, scope, declared_generics, impl_generics);
                    let sym = self.dt_to_gen(*id, self.scopes.get(scope), data_type, generics);

                    let sym_name = self.string_map.concat(path, name);

                    let kind = SymbolKind::Alias(sym);

                    let sym = Symbol::new(sym_name, generics, kind);
                    self.syms.add_sym(tsi, sym);
                }


                Decl::Enum { name, mappings, generics, .. } => {
                    let declared_generics =
                    match self.resolve_generics(scope, *n, generics) {
                        Ok(v) => v,
                        Err(e) => {
                            self.set_error(*id, e);
                            continue;
                        },
                    };
                    let impl_generics = impl_block.map(|(_, gens, _)| gens).unwrap_or(&[]);
                    let generics = self.blocks.all_generics(self.output, &self.scopes, scope, declared_generics, impl_generics);


                    let ns = self.namespaces.get_ns(ns);
                    let mut enum_mappings = Buffer::new(self.output, mappings.len());
                    let Ok(tsi) = ns.get_sym(name).unwrap()
                    else { continue };

                    for f in mappings {
                        let sym = self.dt_to_gen(*id, self.scopes.get(scope), *f.data_type(), generics);

                        let mapping = (f.name(), sym);
                        enum_mappings.push(mapping);
                    }



                    let name = self.string_map.concat(path, name);
                    let source = self.ast.range(*id);

                    self.syms.add_enum(&mut self.namespaces, self.string_map,
                                        tsi, source, name,
                                        enum_mappings.leak(), generics, Some(*id));
                },


                Decl::Function { sig, .. } => {
                    let declared_generics =
                    match self.resolve_generics(scope, (*id).into(), sig.generics) {
                        Ok(e) => e,
                        Err(e) => {
                            let ns = self.namespaces.get_ns_mut(ns);
                            ns.set_err_sym(sig.name, e.clone());
                            self.set_error(*id, e);
                            continue;
                        },
                    };
                    let impl_generics = impl_block.map(|(_, gens, _)| gens).unwrap_or(&[]);
                    let generics = self.blocks.all_generics(self.output, &self.scopes, scope, declared_generics, impl_generics);

                    let mut args = Buffer::new(self.output, sig.arguments.len());

                    for a in sig.arguments {
                        let sym = self.dt_to_gen(*id, self.scopes.get(scope), a.data_type(), generics);

                        let arg = FunctionArgument::new_inout(a.name(), sym, a.is_inout());
                        args.push(arg);
                    }


                    let ret = self.dt_to_gen(*id, self.scopes.get(scope), sig.return_type, generics);
                    let ns = self.namespaces.get_ns(ns);
                    let Some(Ok(fid)) = ns.get_sym(sig.name)
                    else { continue };
                    // Check for special functions
                    if impl_block.is_some() && sig.name == StringMap::ITER_NEXT_FUNC {
                        let validate_sig = || {
                            if sig.arguments.len() != 1 { return false }
                            let (impl_ty, _, _) = impl_block.unwrap_or((SymbolId::MAX, &[], None));
                            let Some(val) = args[0].symbol().sym()
                            else { return false };

                            if val != impl_ty { return false; }

                             if !sig.arguments[0].is_inout() { return false; }
                            if ret.sym() != Some(SymbolId::OPTION) { return false; }

                            true
                        };


                        if !validate_sig() {
                            self.error(*id, Error::IteratorFunctionInvalidSig(sig.source));
                        }
                    }


                    // Finalise
                    let sym_name = self.syms.sym_ns(fid);
                    let mut sym_name = self.namespaces.get_ns(sym_name).path;

                    if let Some((impl_ty, _, Some(trait_path))) = impl_block {
                        let method_path = self.string_map.concat(trait_path, sig.name);
                        sym_name = self.string_map.concat(self.syms.sym(impl_ty).name(), method_path);
                    }

                    let func = FunctionTy::new(args.leak(), ret, FunctionKind::UserDefined, Some(*id), declared_generics);
                    let func = Symbol::new(sym_name, generics, SymbolKind::Function(func));

                    self.syms.add_sym(fid, func);
                }


                Decl::Extern { functions, .. } => {
                    for f in functions {
                        let mut args = Buffer::new(self.output, f.args().len());

                        let gens =
                        match self.resolve_generics(scope, (*id).into(), f.gens()) {
                            Ok(e) => e,
                            Err(e) => {
                                self.set_error(*id, e);
                                continue;
                            },
                        };

                        for a in f.args() {
                            let sym = self.dt_to_gen(*id, self.scopes.get(scope), a.data_type(), gens);
                            let arg = FunctionArgument::new_inout(a.name(), sym, a.is_inout());
                            args.push(arg);
                        }


                        let ret = self.dt_to_gen(*id, self.scopes.get(scope), f.return_type(), gens);

                        let sym_name = self.string_map.concat(path, f.name());

                        let func = FunctionTy::new(args.leak(), ret, FunctionKind::Extern(f.path()), Some(*id), gens);
                        let func = Symbol::new(sym_name, gens, SymbolKind::Function(func));

                        let Ok(id) = self.namespaces.get_ns(ns).get_sym(f.name()).unwrap()
                        else { continue };

                        self.syms.add_sym(id, func);
                    }
                }


                Decl::Trait { name, generics, functions, header, .. } => {
                    let Some(Ok(sym)) = self.namespaces.get_ns(ns).get_sym(name)
                    else { continue };

                    let trait_gens =
                    match self.resolve_generics(scope, *n, generics) {
                        Ok(v) => v,
                        Err(v) => {
                            self.set_error(*n, v);
                            continue;
                        },
                    };

                    let mut scope = self.scopes.push(
                        Scope::new(
                            scope, 
                            ScopeKind::AliasDecl(
                                StringMap::SELF_TY, 
                                Generic::new(header, GenericKind::Generic(BoundedGeneric::new(StringMap::SELF_TY, &[]))))));

                    if !trait_gens.is_empty() {
                        let mut vec = Buffer::new(&*self.output, trait_gens.len());
                        for g in trait_gens {
                            let ty = self.syms.pending(&mut self.namespaces, None, g.name(), 0);
                            let kind = SymbolKind::Container(Container::new(&[], ContainerKind::Generic));
                            self.syms.add_sym(ty, Symbol::new(g.name(), &[], kind));
                            vec.push((*g, self.syms.get_ty(ty, &[])));
                        }
                        let gscope = GenericsScope::new(vec.leak());
                        scope = self.scopes.push(Scope::new(scope, ScopeKind::Generics(gscope)));
                    }

                    let mut funcs = sti::vec::Vec::with_cap_in(self.output, functions.len());
                    for f in functions {
                        let mut args = Buffer::new(self.output, f.arguments.len());

                        let gens = match self.resolve_generics(scope, *n, f.generics) {
                            Ok(v) => v,
                            Err(v) => {
                                self.set_error(*n, v);
                                continue;
                            },
                        };

                        let all_gens =
                        if trait_gens.is_empty() {
                            gens
                        } else if gens.is_empty() {
                            trait_gens
                        } else {
                            let mut combined = Buffer::new(&*self.output, trait_gens.len() + gens.len());
                            for g in trait_gens { combined.push(*g); }
                            for g in gens { combined.push(*g); }
                            combined.leak()
                        };

                        for a in f.arguments {
                            let sym = self.dt_to_gen(*id, self.scopes.get(scope), a.data_type(), all_gens);

                            let arg = FunctionArgument::new_inout(a.name(), sym, a.is_inout());
                            args.push(arg);
                        }

                        let ret = self.dt_to_gen(*id, self.scopes.get(scope), f.return_type, all_gens);

                        funcs.push((f.name, FunctionTy::new(args.leak(), ret, FunctionKind::Trait, None, gens)));
                    }

                    self.syms.add_sym(sym, Symbol::new(name, trait_gens, SymbolKind::Trait(Trait {
                        funcs: funcs.leak(),
                        synthesis: crate::syms::TraitSynthesis::None,
                    })));
                }


                Decl::Module { .. } => (),


                Decl::Impl { body, .. } => {
                    let Some((ty, _, gens)) = self.type_info.impls.get(&id)
                    else { continue };

                    let Some(sym) = ty.sym()
                    else { continue; };


                    let ns = self.syms.sym_ns(sym);
                    let scope = self.scopes.push(Scope::new(scope, ScopeKind::AliasDecl(StringMap::SELF_TY, *ty)));

                    self.compute_types(path, scope, ns, &body, Some((sym, gens, None)));
                }

                Decl::Attribute { decl, .. } => {
                    self.compute_types(path, scope, ns, &[decl.into()], impl_block);
                },


                _ => (),
            }
        }
    }
}
