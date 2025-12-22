use common::{buffer::Buffer, source::SourceRange, string_map::{StringIndex, StringMap}, Once};
use parser::{dt::{DataType, DataTypeKind}, nodes::{decl::{DeclGeneric, Decl, DeclId, FunctionSignature, UseItem, UseItemKind}, expr::{BinaryOperator, Expr, ExprId, UnaryOperator}, stmt::{Stmt, StmtId}, NodeId, Pattern, PatternKind}};
use sti::{alloc::GlobalAlloc, ext::FromIn, key::Key, vec::{KVec, Vec}};

use crate::{errors::Error, namespace::{Namespace, NamespaceId}, scope::{FunctionScope, GenericsScope, Scope, ScopeId, ScopeKind, VariableScope}, syms::{containers::{Container, ContainerKind}, func::{FunctionArgument, FunctionKind, FunctionTy}, sym_map::{BoundedGeneric, GenListId, Generic, GenericKind, SymbolId}, ty::Type, Symbol, SymbolKind, Trait}, AnalysisResult, TyChecker};

impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub fn block(&mut self, path: StringIndex, scope: ScopeId, body: &[NodeId]) -> AnalysisResult {
        let scope = scope;
        let namespace = Namespace::new(path);
        let namespace = self.namespaces.push(namespace);

        // Collect type names
        self.collect_names(path, namespace, body, 0);

        // Update the current scope so the following functions
        // are able to see the namespace
        let scope = Scope::new(Some(scope), ScopeKind::ImplicitNamespace(namespace));
        let mut scope = self.scopes.push(scope);

        // Collect imports
        self.collect_uses(scope, namespace, body);

        // Collect impls
        self.collect_impls(path, scope, namespace, body);

        // Compute types & functions
        self.compute_types(path, scope, namespace, body, None);

        // Analyze all nodes
        let mut last_node = None;
        for node in body.iter() {
            let eval = self.node(path, &mut scope, namespace, *node);
            last_node = Some(eval);
        }

        // Finalise
        let result = match last_node {
            Some(v) => v,
            None    => AnalysisResult::new(Type::UNIT),
        };

        result
    }


    pub fn collect_names(&mut self, path: StringIndex, ns_id: NamespaceId, nodes: &[NodeId], gen_count: usize) {
        for n in nodes {
            let NodeId::Decl(decl) = n
            else { continue };

            let mut ns = self.namespaces.get_ns_mut(ns_id);
            let decl = self.ast.decl(*decl);
            let range = self.ast.range(*n);
            match decl {
                | Decl::Enum { name, header, generics, .. } 
                | Decl::Struct { name, header, generics, .. }
                | Decl::OpaqueType { name, header, gens: generics, .. }
                | Decl::Function { sig: FunctionSignature { name, source: header, generics, .. }, .. }=> {
                    if let Some(sym) = ns.get_sym(name) {
                        let err = Error::NameIsAlreadyDefined {
                            source: header, name };
                        if sym.is_ok() { ns.set_err_sym(name, err.clone()) }

                        self.error(*n, err);
                        continue
                    }

                    if matches!(name, StringMap::ITER_NEXT_FUNC)
                        && !matches!(decl, Decl::Function { .. }) {
                        self.error(*n, Error::NameIsReservedForFunctions { source: header });
                    }

                    let path = self.string_map.concat(path, name);
                    let pend = self.syms.pending(&mut self.namespaces, path, generics.len() + gen_count);
                    ns = self.namespaces.get_ns_mut(ns_id);

                    if let Err(e) = ns.add_sym(range, name, pend) {
                        self.error(*n, e);
                    }
                },


                Decl::Trait { name, header, .. } => {
                    if let Some(sym) = ns.get_sym(name) {
                        let err = Error::NameIsAlreadyDefined {
                            source: header, name };
                        if sym.is_ok() { ns.set_err_sym(name, err.clone()) }

                        self.error(*n, err);
                        continue
                    }


                    let path = self.string_map.concat(path, name);
                    let pend = self.syms.pending(&mut self.namespaces, path, gen_count);
                    ns = self.namespaces.get_ns_mut(ns_id);

                    if let Err(e) = ns.add_sym(range, name, pend) {
                        self.error(*n, e);
                    }

                }


                Decl::Extern { functions }=> {
                    for f in functions {
                        if let Some(sym) = ns.get_sym(f.name()) {
                            let err = Error::NameIsAlreadyDefined {
                                source: f.range(), name: f.name() };

                            if sym.is_ok() { ns.set_err_sym(f.name(), err.clone()) }

                            self.error(*n, err);

                            ns = self.namespaces.get_ns_mut(ns_id);
                            continue
                        }

                        let path = self.string_map.concat(path, f.name());
                        let pend = self.syms.pending(&mut self.namespaces, path, f.gens().len());
                        ns = self.namespaces.get_ns_mut(ns_id);

                        if let Err(e) = ns.add_sym(range, f.name(), pend) {
                            self.error(*n, e);
                            ns = self.namespaces.get_ns_mut(ns_id);
                        }

                    }
                },


                Decl::Module { name, header, body, .. } => {
                    let path = self.string_map.concat(path, name);

                    let sym = self.syms.pending(&mut self.namespaces, path, 0);
                    self.syms.add_sym(sym, Symbol::new(name, &[], SymbolKind::Namespace));

                    let module_ns = self.syms.as_ns(sym);

                    if let Err(e) = self.namespaces.get_ns_mut(ns_id).add_sym(header, name, sym) {
                        self.error(*n, e);
                        continue;
                    }


                    self.collect_names(path, module_ns, &*body, gen_count);
                },


                Decl::Attribute { decl, .. } => self.collect_names(path, ns_id, &[decl.into()], gen_count),

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
                Decl::Module { name, body, user_defined, .. } => {
                    let module_ns = self.namespaces.get_ns(ns_id);
                    let Some(Ok(module_ns)) = module_ns.get_sym(name)
                    else { continue; };

                    let module_ns = self.syms.as_ns(module_ns);

                    let scope = Scope::new(
                        if !user_defined { self.base_scope }
                        else { scope }, 

                        ScopeKind::ImplicitNamespace(module_ns),
                    );

                    let scope = self.scopes.push(scope);
                    self.collect_impls(path, scope, module_ns, &*body);
                }


                Decl::Impl { data_type, gens, body } => {
                    let s = self.scopes.get(scope);
                    let gens = match self.resolve_generics(scope, n, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            continue;
                        },
                    };

                    let ty = match self.dt_to_gen(s, data_type, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            continue;
                        },
                    };

                    let source = self.ast.range(n);
                    let Some(sym) = ty.sym()
                    else {
                        self.error(n, Error::ImplOnGeneric(source));
                        continue;
                    };


                    self.type_info.impls.insert(id, (ty, ty, gens));

                    let ns = self.syms.sym_ns(sym);

                    let path = self.namespaces.get_ns(ns).path;

                    self.collect_names(path, ns, &body, gens.len());
                    self.collect_impls(path, scope, ns, &body);
                }

                Decl::ImplTrait { trait_name, data_type, gens, body, header } => {
                    let gens = match self.resolve_generics(scope, n, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            continue;
                        },
                    };

                    let s = self.scopes.get(scope);
                    let trait_ty = match self.dt_to_gen(s, trait_name, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            return;
                        },
                    };

                    let ty = match self.dt_to_gen(s, data_type, gens) {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            return;
                        },
                    };

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
                    let ns = self.namespaces.push(ns);

                    self.syms.traits(sym).insert(trait_sym_id, (ns, ty, gens));
                }





                Decl::Attribute { decl, .. } => self.collect_impls(path, scope, ns_id, &[decl.into()]),

                _ => (),
            }
        }
    }


    pub fn collect_uses(&mut self, scope_id: ScopeId, ns_id: NamespaceId, nodes: &[NodeId]) {
        let scope = self.scopes.get(scope_id);
        for n in nodes {
            let NodeId::Decl(id) = *n
            else { continue; };

            match self.ast.decl(id) {
                Decl::Module { name, body, .. } => {
                    let module_ns = self.namespaces.get_ns(ns_id);
                    let Some(Ok(module_ns)) = module_ns.get_sym(name)
                    else { continue; };

                    let module_ns = self.syms.as_ns(module_ns);

                    let scope = Scope::new(Some(scope_id), ScopeKind::ImplicitNamespace(module_ns));
                    let scope = self.scopes.push(scope);
                    self.collect_uses(scope, module_ns, &body);
                }


                Decl::Impl { body, .. } => {
                    let Some((ty, _, _)) = self.type_info.impls.get(&id)
                    else { continue };

                    let Some(sym) = ty.sym()
                    else { continue; };


                    let ns = self.syms.sym_ns(sym);
                    self.collect_uses(scope_id, ns, &body)
                }


                Decl::Using { item } => {
                    self.collect_use_item(*n, scope, ns_id, item)
                }


                Decl::Attribute { decl, .. } => self.collect_uses(scope_id, ns_id, &[decl.into()]),

                _ => continue,
            }
        }
    }


    fn collect_use_item(&mut self, node: NodeId, scope: Scope, ns_id: NamespaceId, item: UseItem) {
        match item.kind() {
            UseItemKind::List { list } => {
                let Some(import_ns) = scope.find_sym(item.name(), &self.scopes, &mut self.syms, &self.namespaces)
                else {
                    self.error(node, Error::NamespaceNotFound { source: item.range(), namespace: item.name() });
                    return;
                };

                let Ok(import_ns) = import_ns
                // @todo: we should probably recursively go and mark everything imported as
                // "errored" as well
                else { return; };

                let import_ns = self.syms.sym_ns(import_ns);
                let scope = Scope::new(None, ScopeKind::ImplicitNamespace(import_ns));
                for ui in list {
                    self.collect_use_item(node, scope, ns_id, *ui);
                }
            },


            UseItemKind::BringName => {
                if let Some(import_sym) = scope.find_sym(item.name(), &self.scopes, &mut self.syms, &self.namespaces) {
                    let ns = self.namespaces.get_ns_mut(ns_id);

                    match import_sym {
                        Ok(v) => {
                            if let Err(e) = ns.add_sym(item.range(), item.name(), v) {
                                self.error(node, e);
                            }
                        },
                        Err(e) => ns.set_err_sym(item.name(), e),
                    };
                    return;
                };


                self.error(node, Error::NamespaceNotFound { source: item.range(), namespace: item.name() });
            },


            UseItemKind::All => {
                let Some(import_ns) = scope.find_sym(item.name(), &self.scopes, &mut self.syms, &self.namespaces)
                else {
                    self.error(node, Error::NamespaceNotFound { source: item.range(), namespace: item.name() });
                    return;
                };

                let Ok(import_ns) = import_ns
                // @todo: we should probably recursively go and mark everything imported as
                // "errored" as well
                else { return; };

                let import_ns = self.syms.sym_ns(import_ns);

                let (ns, import_ns) = self.namespaces.get_double(ns_id, import_ns);

                for s in import_ns.syms() {
                    match s.1 {
                        Ok(v) => {
                            if let Err(e) = ns.add_sym(item.range(), *s.0, *v) {
                                Self::error_ex(&mut self.errors, &mut self.type_info, node, e);
                            }
                        },
                        Err(e) => ns.set_err_sym(*s.0, e.clone()),
                    }
                }
            },
        };

    }


    // `Self::collect_names` must be ran before this
    pub fn compute_types(&mut self, path: StringIndex, scope: ScopeId,
                         ns: NamespaceId, nodes: &[NodeId], impl_block: Option<(SymbolId, &[BoundedGeneric<'out>])>) {
        for n in nodes {
            let NodeId::Decl(id) = n
            else { continue };

            let decl = self.ast.decl(*id);
            println!("computing for");
            match decl {
                 Decl::Struct { name, fields, generics, .. } => {
                    let generics = match self.resolve_generics(scope, *n, generics) {
                        Ok(v) => v,
                        Err(e) => {
                            self.error(*id, e);
                            continue;
                        },
                    };


                    let ns = self.namespaces.get_ns(ns);
                    let mut structure_fields = Buffer::new(self.output, fields.len());

                    let tsi = match ns.get_sym(name).unwrap() {
                        Ok(e) => e,
                        Err(e) => {
                            self.error(*id, e);
                            continue;
                        },
                    };

                    for f in fields {
                        let sym = self.dt_to_gen(self.scopes.get(scope), f.1, generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                let id = self.error(*id, v);
                                Generic::new(f.1.range(), GenericKind::ERROR, Some(id))
                            },
                        };

                        let field = (f.0, sym);
                        structure_fields.push(field);
                    }

                    // finalise
                    let generics = {
                        let (_, impl_gens) = impl_block.unwrap_or((SymbolId::MAX, &[]));
                        let mut vec = Buffer::new(self.output, impl_gens.len() + generics.len());
                        vec.extend_from_slice(impl_gens);
                        vec.extend_from_slice(generics);

                        vec.leak()
                    };


                    let sym_name = self.string_map.concat(path, name);
                    let cont = Container::new(structure_fields.leak(), ContainerKind::Struct);
                    let kind = SymbolKind::Container(cont);

                    let sym = Symbol::new(sym_name, generics, kind);
                    self.syms.add_sym(tsi, sym);
                },


                Decl::OpaqueType { name, gens, .. } => {
                    let ns = self.namespaces.get_ns(ns);
                    let Ok(tsi) = ns.get_sym(name).unwrap()
                    else { continue };

                    let gens = match self.resolve_generics(scope, *n, gens) {
                        Ok(v) => v,
                        Err(e) => {
                            self.error(*id, e);
                            continue;
                        },
                    };

                    let generics = {
                        let (_, impl_gens) = impl_block.unwrap_or((SymbolId::MAX, &[]));
                        let mut vec = Buffer::new(self.output, impl_gens.len() + gens.len());
                        vec.extend_from_slice(impl_gens);
                        vec.extend_from_slice(gens);

                        vec.leak()
                    };


                    let sym_name = self.string_map.concat(path, name);
                    let kind = SymbolKind::Opaque;

                    let sym = Symbol::new(sym_name, generics, kind);
                    self.syms.add_sym(tsi, sym);
                }


                Decl::Enum { name, mappings, generics, .. } => {
                    let generics = match self.resolve_generics(scope, *n, generics) {
                        Ok(v) => v,
                        Err(e) => {
                            self.error(*id, e);
                            continue;
                        },
                    };




                    let ns = self.namespaces.get_ns(ns);
                    let mut enum_mappings = Buffer::new(self.output, mappings.len());
                    let Ok(tsi) = ns.get_sym(name).unwrap()
                    else { continue };

                    for f in mappings {
                        let sym = self.dt_to_gen(self.scopes.get(scope), *f.data_type(), generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                let id = self.error(*id, v);
                                Generic::new(f.data_type().range(), GenericKind::ERROR, Some(id))
                            },
                        };

                        let mapping = (f.name(), sym);
                        enum_mappings.push(mapping);
                    }

                    // finalise
                    let generics = {
                        let (_, impl_gens) = impl_block.unwrap_or((SymbolId::MAX, &[]));
                        let mut vec = Buffer::new(self.output, impl_gens.len() + generics.len());
                        vec.extend_from_slice(impl_gens);
                        vec.extend_from_slice(generics);
                        vec.leak()
                    };


                    let name = self.string_map.concat(path, name);
                    let source = self.ast.range(*id);

                    self.syms.add_enum(tsi, &mut self.namespaces, self.string_map,
                                        source, name,
                                        enum_mappings.leak(), generics, Some(*id));
                },


                Decl::Function { sig, .. } => {
                    let gens =
                    match self.resolve_generics(scope, (*id).into(), sig.generics) {
                        Ok(e) => e,
                        Err(e) => {
                            println!("errored while resolving generics");
                            let ns = self.namespaces.get_ns_mut(ns);
                            ns.set_err_sym(sig.name, e.clone());
                            self.error(*id, e);
                            continue;
                        },
                    };

                    let generics = {
                        let (_, impl_gens) = impl_block.unwrap_or((SymbolId::MAX, &[]));
                        let mut vec = Buffer::new(self.output, impl_gens.len() + sig.generics.len());
                        vec.extend_from_slice(impl_gens);
                        vec.extend_from_slice(gens);
                        vec.leak()
                    };

                    let ns = self.namespaces.get_ns(ns);
                    let Some(Ok(fid)) = ns.get_sym(sig.name)
                    else { println!("symbol is errored"); continue };

                    let mut args = Buffer::new(self.output, sig.arguments.len());

                    for a in sig.arguments {
                        let sym = self.dt_to_gen(self.scopes.get(scope), a.data_type(), generics);
                        let sym = match sym {
                            Ok(v) => v,
                            Err(v) => {
                                let id = self.error(*id, v);
                                Generic::new(a.data_type().range(), GenericKind::ERROR, Some(id))
                            },
                        };

                        let arg = FunctionArgument::new(a.name(), sym);
                        args.push(arg);
                    }


                    let ret = self.dt_to_gen(self.scopes.get(scope), sig.return_type, generics);
                    let ret = match ret {
                        Ok(v) => v,
                        Err(v) => {
                            let id = self.error(*id, v);
                            Generic::new(sig.return_type.range(), GenericKind::ERROR, Some(id))
                        },
                    };


                    // Check for special functions
                    if impl_block.is_some() && sig.name == StringMap::ITER_NEXT_FUNC {
                        let validate_sig = || {
                            if sig.arguments.len() != 1 { return false }
                            let (impl_ty, _) = impl_block.unwrap_or((SymbolId::MAX, &[]));
                            let Some(val) = args[0].symbol().sym()
                            else { return false };

                            if val != impl_ty { return false; }

                            //if !sig.arguments[0].is_inout() { return false; }
                            if ret.sym() != Some(SymbolId::OPTION) { return false; }

                            true
                        };


                        if !validate_sig() {
                            self.error(*id, Error::IteratorFunctionInvalidSig(sig.source));
                        }
                    }


                    // Finalise
                    let sym_name = self.syms.sym_ns(fid);
                    let sym_name = self.namespaces.get_ns(sym_name).path;

                    let func = FunctionTy::new(args.leak(), ret, FunctionKind::UserDefined, Some(*id));
                    let func = Symbol::new(sym_name, generics, SymbolKind::Function(func));

                    self.syms.add_sym(fid, func);
                    println!("computed");
                }


                Decl::Extern { functions } => {
                    for f in functions {
                        let mut args = Buffer::new(self.output, f.args().len());

                        let gens =
                        match self.resolve_generics(scope, (*id).into(), f.gens()) {
                            Ok(e) => e,
                            Err(e) => {
                                self.error(*id, e);
                                continue;
                            },
                        };



                        for a in f.args() {
                            let sym = self.dt_to_gen(self.scopes.get(scope), a.data_type(), gens);
                            let sym = match sym {
                                Ok(v) => v,
                                Err(v) => {
                                    let id = self.error(*id, v);
                                    Generic::new(a.data_type().range(), GenericKind::ERROR, Some(id))
                                },
                            };

                            let arg = FunctionArgument::new(a.name(), sym);
                            args.push(arg);
                        }


                        let ret = self.dt_to_gen(self.scopes.get(scope), f.return_type(), gens);
                        let ret = match ret {
                            Ok(v) => v,
                            Err(v) => {
                                let id = self.error(*id, v);
                                Generic::new(f.return_type().range(), GenericKind::ERROR, Some(id))
                            },
                        };


                        // Finalise
                        let sym_name = self.string_map.concat(path, f.name());

                        let func = FunctionTy::new(args.leak(), ret, FunctionKind::Extern(f.path()), Some(*id));
                        let func = Symbol::new(sym_name, gens, SymbolKind::Function(func));

                        let Ok(id) = self.namespaces.get_ns(ns).get_sym(f.name()).unwrap()
                        else { continue };

                        self.syms.add_sym(id, func);
                    }
                }


                Decl::Trait { name, functions, header } => {
                    let Some(Ok(sym)) = self.namespaces.get_ns(ns).get_sym(name)
                    else { continue };

                    let scope = self.scopes.push(
                        Scope::new(
                            scope, 
                            ScopeKind::AliasDecl(
                                StringMap::SELF_TY, 
                                Generic::new(header, GenericKind::Generic(BoundedGeneric::new(StringMap::SELF_TY, &[])), 
                                None,
                    ))));


                    let mut funcs = sti::vec::Vec::with_cap_in(self.output, functions.len());
                    for f in functions {
                        let mut args = Buffer::new(self.output, f.arguments.len());

                        let gens = match self.resolve_generics(scope, *n, f.generics) {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(*n, v);
                                continue;
                            },
                        };


                        for a in f.arguments {
                            let sym = self.dt_to_gen(self.scopes.get(scope), a.data_type(), gens);
                            let sym = match sym {
                                Ok(v) => v,
                                Err(v) => {
                                    let id = self.error(*id, v);
                                    Generic::new(a.data_type().range(), GenericKind::ERROR, Some(id))
                                },
                            };

                            let arg = FunctionArgument::new(a.name(), sym);
                            args.push(arg);
                        }


                        let ret = self.dt_to_gen(self.scopes.get(scope), f.return_type, gens);
                        let ret = match ret {
                            Ok(v) => v,
                            Err(v) => {
                                let id = self.error(*id, v);
                                Generic::new(f.return_type.range(), GenericKind::ERROR, Some(id))
                            },
                        };


                        funcs.push((f.name, FunctionTy::new(args.leak(), ret, FunctionKind::Trait, None)));
                    }

                    self.syms.add_sym(sym, Symbol::new(name, &[], SymbolKind::Trait(Trait { funcs: funcs.leak() })));
                }


                Decl::Module { name, body, user_defined, .. } => {

                    let ns = self.namespaces.get_ns(ns);
                    let Some(Ok(module_ns)) = ns.get_sym(name)
                    else { continue; };

                    let module_ns = self.syms.as_ns(module_ns);

                    let scope = self.scopes.push(self.scopes.get(scope));
                    let scope = Scope::new(if !user_defined { self.base_scope } else { scope }, ScopeKind::ImplicitNamespace(module_ns));
                    let scope = self.scopes.push(scope);

                    let path = self.namespaces.get_ns(module_ns).path;
                    self.compute_types(path, scope, module_ns, &*body, None);
                }


                Decl::Impl { body, .. } => {
                    let Some((ty, _, gens)) = self.type_info.impls.get(&id)
                    else { continue };

                    let Some(sym) = ty.sym()
                    else { continue; };


                    let ns = self.syms.sym_ns(sym);
                    let scope = self.scopes.push(Scope::new(scope, ScopeKind::AliasDecl(StringMap::SELF_TY, *ty)));

                    self.compute_types(path, scope, ns, &body, Some((sym, gens)));
                }

                Decl::Attribute { decl, .. } => {
                    self.compute_types(path, scope, ns, &[decl.into()], impl_block);
                },


                _ => (),
            }
        }
    }


    pub fn node(&mut self, path: StringIndex,
                scope: &mut ScopeId, ns: NamespaceId, node: NodeId) -> AnalysisResult {
        match node {
            NodeId::Decl(decl) => {
                if let Decl::Error(e) = self.ast.decl(decl) {
                    self.type_info.set_decl(decl, e);
                    return AnalysisResult::new(Type::ERROR);
                }


                self.decl(scope, ns, decl);
                AnalysisResult::new(Type::UNIT)
            },

            NodeId::Stmt(stmt) => {
                self.stmt(path, scope, stmt);
                AnalysisResult::new(Type::UNIT)
            },

            NodeId::Expr(expr) => self.expr(path, *scope, expr),

            NodeId::Err(_) => {
                AnalysisResult::new(Type::ERROR)
            },
        }
    }


    pub fn decl(&mut self, scope: &mut ScopeId, ns: NamespaceId, n: DeclId) {
        let decl = self.ast.decl(n);
        match decl {
            Decl::Struct { .. } => (),
            Decl::Enum { .. } => (),
            Decl::OpaqueType { .. } => (),
            Decl::ImportFile { .. } => unreachable!(),
            Decl::ImportRepo { .. } => unreachable!(),
            Decl::Error(_) => unreachable!(),


            Decl::Trait { functions, .. } => (),

            
            Decl::Function { sig, body, .. } => {
                let ns = self.namespaces.get_ns(ns);
                let Some(Ok(func_id)) = ns.get_sym(sig.name)
                else { return };


                // we need a scope that'd fake the generics
                let sym = self.syms.sym(func_id);
                let SymbolKind::Function(func) = sym.kind()
                else { unreachable!() };

                let generics = sym.generics();
                let generics = {
                    let mut vec = Buffer::new(&*self.output, generics.len());
                    for g in generics {
                        let ty = self.syms.pending(&mut self.namespaces, g.name(), 0);
                        let kind = SymbolKind::Container(Container::new(&[], ContainerKind::Generic));

                        self.syms.add_sym(ty, Symbol::new(g.name(), &[], kind));
                        let hm = self.syms.traits(ty);

                        for b in g.bounds.iter() {
                            hm.insert(
                                *b, 
                                (
                                    NamespaceId::MAX,
                                    Generic::new(sig.source, GenericKind::Sym(ty, &[]), None),
                                    &[],
                                )
                            );
                        }

                        vec.push((*g, self.syms.get_ty(ty, &[])));
                    }

                    vec.leak()
                };

                if sig.name == self.string_map.insert("main") {
                    self.startups.push(func_id);
                }
                
                // fake args
                let gscope = GenericsScope::new(generics);
                let mut scope = Scope::new(*scope, ScopeKind::Generics(gscope));

                for a in func.args() {
                    let ty = a.symbol().to_ty(generics, &mut self.syms);
                    let ty = match ty {
                        Ok(v) => v,
                        Err(v) => {
                            self.error(n, v);
                            Type::ERROR
                        }
                    };

                    let vs = VariableScope::new(a.name(), ty);
                    scope = Scope::new(Some(self.scopes.push(scope)), ScopeKind::VariableScope(vs))
                }

                let ret = func.ret().to_ty(generics, &mut self.syms);
                let ret = match ret {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(n, v);
                        Type::ERROR
                    }
                };

                // func scope
                let fs = FunctionScope::new(ret, sig.return_type.range());
                scope = Scope::new(Some(self.scopes.push(scope)), ScopeKind::Function(fs));

                let scope = self.scopes.push(scope);

                // GO GO GO
                let anal = self.block(sym.name(), scope, &*body);

                if !anal.ty.eq(&mut self.syms, ret) {
                    self.error(n, Error::FunctionBodyAndReturnMismatch {
                        header: sig.source, item: body.range(),
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
                let Some((trait_ty, ty, gens)) = self.type_info.impls.get(&n)
                else { return; };

                let ty = *ty;

                let Some(trait_sym_id) = trait_ty.sym()
                else {
                    return;
                };


                let Some(sym) = ty.sym()
                else {
                    return;
                };

                let gens = *gens;

                let trait_sym = self.syms.sym(trait_sym_id);
                let SymbolKind::Trait(tr) = trait_sym.kind()
                else {
                    self.error(n, Error::ImplTraitOnNonTrait(data_type.range()));
                    return;
                };

                let path = trait_sym.name();

                let ns_id = self.syms.traits(sym).get(&trait_sym_id).unwrap().0;
                let scope = Scope::new(*scope, ScopeKind::ImplicitNamespace(ns_id));
                let scope = self.scopes.push(scope);

                let scope = Scope::new(scope, ScopeKind::AliasDecl(StringMap::SELF_TY, ty));
                let mut scope = self.scopes.push(scope);

                self.collect_names(path, ns_id, &body, gens.len());
                self.collect_impls(path, scope, ns_id, &body);
                self.compute_types(path, scope, ns_id, &body, Some((sym, gens)));


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

                    let sym = self.syms.sym(sym_id);

                    let SymbolKind::Function(f) = sym.kind()
                    else {
                        todo!();
                    };


                    if f.args().len() != ft.args().len() {
                        let decl = f.decl().unwrap();
                        let source = self.ast.range(decl);

                        let err = Error::FunctionArgsMismatch {
                            source, 
                            sig_len: ft.args().len(),
                            call_len: f.args().len() 
                        };

                        let err = self.error(decl, err);
                        self.syms.set_err(sym_id, err);

                        ns = self.namespaces.get_ns_mut(ns_id);

                        continue;
                    }

                    let args = f.args().iter().map(|x| x.symbol()).chain([f.ret()].into_iter());
                    let fargs = ft.args().iter().map(|x| x.symbol()).chain([f.ret()].into_iter());

                    for (arg, farg) in args.zip(fargs) {
                        let arg = arg.rec_replace(self.output, StringMap::SELF_TY, ty);
                        let farg = farg.rec_replace(self.output, StringMap::SELF_TY, ty);

                        if arg != farg {
                            let decl = f.decl().unwrap();

                            let err = Error::InvalidArgument { source: arg.range() };

                            let err = self.error(decl, err);
                            self.syms.set_err(sym_id, err);

                            ns = self.namespaces.get_ns_mut(ns_id);
                        }
                    }


                }

                if !missing.is_empty() {
                    self.error(n, Error::MissingFuncs { source: header, fields: missing });
                }
            }




            Decl::Module { name, body, user_defined, .. } => {
                let ns = self.namespaces.get_ns(ns);

                let Some(Ok(module_ns)) = ns.get_sym(name)
                else { return; };

                let module_ns = self.syms.as_ns(module_ns);


                let scope = Scope::new(if !user_defined { self.base_scope } else { *scope }, ScopeKind::ImplicitNamespace(module_ns));
                let mut scope = self.scopes.push(scope);

                let path = self.namespaces.get_ns(module_ns).path;
                for n in body.iter() {
                    self.node(path, &mut scope, module_ns, *n);
                }
            },


            Decl::Using { .. } => (),
            Decl::Extern { .. } => (),

            Decl::Attribute { decl: decl_id, attr, attr_range } => {
                self.decl(scope, ns, decl_id);

                match self.string_map.get(attr) {
                    "test" => {
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
                                attr: (attr_range, attr), value: range, expected: "'fn()'" });
                            return;
                        };

                        let Ok(func) = self.namespaces.get_ns(ns).get_sym(name).unwrap()
                        else { return };

                        self.tests.push(func);
                    },


                    "cached" => {
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
                                attr: (attr_range, attr), value: range, expected: "'a function'" });
                            return;
                        };

                        let Ok(func) = self.namespaces.get_ns(ns).get_sym(name).unwrap()
                        else { unreachable!() };

                        self.syms.cached_fn(func);
                    }

                    _ => {
                        self.error(n, Error::UnknownAttr(attr_range, attr));
                    }
                }
            },
        }
    }


    pub fn resolve_pattern(
        &mut self, id: NodeId, scope: &mut ScopeId, 
        pattern: Pattern, rhs: AnalysisResult, rhs_range: SourceRange
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
                    let vs = VariableScope::new(name, rhs.ty);
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


                    let gens = ty.gens(&self.syms);
                    let gens = self.syms.get_gens(gens);
                    for (&item, (_, ty)) in items.iter().zip(gens.iter()) {
                        let vs = VariableScope::new(item, *ty);
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
            Stmt::Variable { pat, hint, rhs } => {
                let mut rhs_anal = self.expr(path, *scope, rhs);

                let mut validate_hint = || {
                    if let Some(hint) = hint {
                        let hint = match self.dt_to_ty(*scope, id, hint) {
                            Ok(v)  => v,
                            Err(v) => {
                                rhs_anal.ty = Type::ERROR;
                                return Err(v);
                            },
                        };

                        if !rhs_anal.ty.eq(&mut self.syms, hint) {
                            rhs_anal.ty = hint;
                            return Err(Error::VariableValueAndHintDiffer {
                                value_type: rhs_anal.ty, hint_type: hint, source })
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
                    id.into(), scope, pat, rhs_anal, rhs_range);

                if let Err(e) = validate_hint {
                    self.error(id, e);
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

                match self.ast.expr(lhs) {
                      Expr::Identifier(_, _)
                    | Expr::IndexList { .. }
                    | Expr::AccessField { .. }
                    | Expr::Unwrap(_)
                    | Expr::OrReturn(_) if lhs_anal.is_mut => (),

                    _ => {
                        let range = self.ast.range(lhs);
                        self.error(id, Error::AssignIsNotLHSValue { source: range });
                    }
                }
            },


            Stmt::ForLoop { binding, expr, body } => {
                let iter_anal = self.expr(path, *scope, expr);

                // check if the exprs type is an iterable
                let Ok(sym) = iter_anal.ty.sym(&mut self.syms)
                else {
                    let range = self.ast.range(expr);

                    let scope = Scope::new(*scope, ScopeKind::Loop);
                    let mut scope = self.scopes.push(scope);

                    let _ = self.resolve_pattern(
                        id.into(), &mut scope, binding, 
                        AnalysisResult::error(), range
                    );

                    let _ = self.block(path, scope, &body);

                    return;
                };

                let func = self.syms.sym_ns(sym);
                let ns = self.namespaces.get_ns(func);
                let Some(sym) = ns.get_sym(StringMap::ITER_NEXT_FUNC)
                else { 
                    let range = self.ast.range(expr);
                    self.error(id, Error::ValueIsntAnIterator { ty: iter_anal.ty, range });

                    let scope = Scope::new(*scope, ScopeKind::Loop);
                    let mut scope = self.scopes.push(scope);

                    let _ = self.resolve_pattern(
                        id.into(), &mut scope, binding, 
                        AnalysisResult::error(), range
                    );

                    let _ = self.block(path, scope, &body);

                    return;
                };

                let Ok(sym) = sym else { return };
                

                // check if the exprs type is a mutable iterable
                let binding_ty = self.syms.sym(sym);
                let SymbolKind::Function(binding_ty) = binding_ty.kind()
                else { unreachable!() };

                let gens = iter_anal.ty.gens(&self.syms);
                let gens = self.syms.get_gens(gens);

                let binding_ty = binding_ty.ret().to_ty(gens, &mut self.syms);
                let binding_ty = match binding_ty {
                    Ok(v) => v,
                    Err(v) => {
                        self.error(id, v);
                        Type::ERROR
                    },
                };

                // unwrap the option
                let binding_ty = binding_ty.gens(&self.syms);
                let binding_ty = self.syms.get_gens(binding_ty);
                if binding_ty.is_empty() { return; }
                let binding_ty = binding_ty[0].1;

                let scope = Scope::new(*scope, ScopeKind::Loop);
                let mut scope = self.scopes.push(scope);

                self.resolve_pattern(
                    id.into(), &mut scope, binding, 
                    AnalysisResult::new(binding_ty), source
                );


                let _ = self.block(path, scope, &body);

            },
        }
    }


    pub fn expr(&mut self, path: StringIndex, scope: ScopeId, id: ExprId) -> AnalysisResult {
        self.expr_ex(path, scope, id, None)
    }

    pub fn expr_ex(&mut self, path: StringIndex, scope: ScopeId, id: ExprId, expected: Option<Type>) -> AnalysisResult {
        let range = self.ast.range(id);
        let expr = self.ast.expr(id);
        let result = (|| -> Result<AnalysisResult, Error> {Ok(match expr {
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
                let mut pregens = None;
                let variable = self.scopes.get(scope)
                    .find_var(ident, &self.scopes, &self.namespaces, &mut self.syms)
                    .or_else(|| {
                        let sym_id = self.scopes.get(scope).find_super(&self.scopes)?;

                        let mut candidate = None;
                        let candidates = self.syms.traits(sym_id).clone();
                        dbg!(&candidates);

                        self.scopes.get(scope)
                        .over::<()>(&self.scopes,
                        |scope| {
                            let ScopeKind::ImplicitNamespace(ns) = scope.kind()
                            else { return None };

                            let ns = self.namespaces.get_ns(ns);

                            for s in ns.syms().values() {
                                let Ok(s) = s
                                else { continue };

                                let Some((_, g, generics)) = candidates.get(s)
                                else { continue; };

                                let sym = self.syms.sym(*s);
                                let SymbolKind::Trait(tr) = sym.kind()
                                else { continue; };

                                let Some(ft) = tr.funcs.iter().find(|x| x.0 == ident)
                                else { continue; };

                                if candidate.is_none() {
                                    candidate = Some((*s, ft.1, *g, generics));
                                    return Some(());
                                } else {
                                    todo!("ambigious");
                                }

                            }

                            None

                        });


                        let Some((t, func, g, generics)) = candidate
                        else { return None; };

                        let mut vgens = sti::vec::Vec::with_cap_in(self.output, generics.iter().len());

                        for g in generics.iter() {
                            let var = self.syms.new_var(id, range);
                            vgens.push((*g, var));
                        }

                        let gens = self.syms.add_gens(vgens.leak());

                        let closure = self.syms.new_closure();
                        
                        pregens = Some(gens);
                        self.type_info.set_acc(id, t);
                        self.type_info.set_ident(id, Some(sym_id));

                        let mut func_args = sti::vec::Vec::with_cap_in(self.output, func.args().len());
                        for arg in func.args() {
                            let gn = arg.symbol()
                                .rec_replace(self.output, StringMap::SELF_TY, g);
                            func_args.push(FunctionArgument::new(arg.name(), gn));
                        }

                        let ret = func.ret()
                            .rec_replace(self.output, StringMap::SELF_TY, g);

                        let sym = self.func_sym(closure, func_args.leak(), ret, generics);

                        Some(Err(Ok(sym)))
                    });


                let Some(variable) = variable
                else { return Err(Error::VariableNotFound { name: ident, source: range }) };

                match variable {
                    Ok(variable) => {
                        if gens.is_some() {
                            return Err(Error::GenericLenMismatch { source: range, found: gens.map(|gs| gs.len()).unwrap_or(0), expected: 0 })
                        }

                        return Ok(AnalysisResult::new(variable.ty()))
                    },


                    Err(sym) => {
                        let sym_id = sym?;

                        let sym = self.syms.sym(sym_id);

                        match sym.kind() {
                            SymbolKind::Function(func) => {
                                self.type_info.set_ident(id, Some(sym_id));

                                if let Some(gens) = gens
                                && sym.generics().len() != gens.len() {
                                    return Err(Error::GenericLenMismatch { source: range, found: gens.len(), expected: sym.generics().len() })
                                }

                                let mut vgens = sti::vec::Vec::with_cap_in(self.output, sym.generics().iter().len());

                                if let Some(gens) = gens {
                                    for (g, dt) in sym.generics().iter().zip(gens.iter()) {
                                        let sym = self.dt_to_ty(scope, id, *dt)?;

                                        let g = BoundedGeneric::new(g.name(), &[]);
                                        vgens.push((g, sym));
                                    }

                                } else {
                                    for g in sym.generics().iter() {
                                        let var = self.syms.new_var(id, range);
                                        vgens.push((*g, var));
                                    }
                                }

                                //dbg!(sym, &vgens);
                                let gens = self.syms.add_gens(vgens.leak());

                                let mut anal = match func.kind() {
                                    FunctionKind::Closure(_) => AnalysisResult::new(Type::Ty(sym_id, gens)),
                                    _ => {
                                        let closure = self.syms.new_closure();

                                        let sym = self.func_sym(closure, func.args(), func.ret(), sym.generics());
                                        AnalysisResult::new(Type::Ty(sym, gens))
                                    }
                                };

                                anal.is_mut = false;
                                return Ok(anal)

                            },

                            _ => (),
                        }


                    },
                };

                return Err(Error::VariableNotFound { name: ident, source: range })
            },


            Expr::Closure { args, body } => {
                let closure = self.syms.new_closure();
                let ret_var = self.syms.new_var(id, range);

                let closure_scope = self.scopes.push(Scope::new(Some(scope), ScopeKind::Function(FunctionScope { ret: ret_var, ret_source: range })));
                let closure_scope = self.scopes.push(Scope::new(Some(closure_scope), ScopeKind::Closure(closure)));
                let mut active_scope = closure_scope;
                let mut sargs = sti::vec::Vec::new_in(self.syms.arena());
                for arg in args {
                    let ty = if let Some(ty) = arg.1 {
                        self.dt_to_ty(scope, id, ty)?
                    } else {
                        self.syms.new_var(id, arg.2)
                    };

                    active_scope = self.scopes.push(Scope::new(
                        Some(active_scope), 
                        ScopeKind::VariableScope(VariableScope::new(arg.0, ty))
                    ));

                    sargs.push((arg.0, ty, arg.2));

                }

                if let Some(sym) = expected
                && let Ok(sym_id) = sym.sym(&mut self.syms)
                && let SymbolKind::Function(func) = self.syms.sym(sym_id).kind() {
                    let gens = sym.gens(&self.syms);
                    let gens = self.syms.get_gens(gens);

                    for (sym_arg, arg) in func.args().iter().zip(sargs.iter()) {
                        let Ok(sym_arg) = sym_arg.symbol().to_ty(gens, &mut self.syms)
                        else { continue };

                        sym_arg.eq(&mut self.syms, arg.1);
                    }

                    if let Ok(sym_ret) = func.ret().to_ty(gens, &mut self.syms) {
                        sym_ret.eq(&mut self.syms, ret_var);
                    }
                }


                // process the body
                let ret = self.expr(path, active_scope, body);


                if !ret.ty.eq(&mut self.syms, ret_var) {
                    let source = self.ast.range(body);
                    return Err(Error::InvalidType { source, found: ret.ty, expected: ret_var });
                }


                let mut fargs = sti::vec::Vec::new_in(self.syms.arena());
                let mut gens = sti::vec::Vec::with_cap_in(self.syms.arena(), sargs.len() + 1);
                let mut gen_list = sti::vec::Vec::with_cap_in(self.syms.arena(), sargs.len() + 1);
                let t = BoundedGeneric::T;
                gens.push((t, ret.ty));
                gen_list.push(t);

                for (i, arg) in sargs.iter().enumerate() {
                    let sym = arg.1;
                    let g = self.string_map.num(i);
                    let g = BoundedGeneric::new(g, &[]);
                    gens.push((g, sym));
                    gen_list.push(g);
                    fargs.push(FunctionArgument::new(arg.0, Generic::new(arg.2, GenericKind::Generic(g), None)));
                }

                let ret = Generic::new(range, GenericKind::Generic(t), None);

                let closure_ty = self.func_sym(closure, fargs.leak(), ret, gen_list.leak());
                let gens = self.syms.add_gens(gens.leak());

                AnalysisResult::new(Type::Ty(closure_ty, gens))
            }


            Expr::Range { lhs, rhs  } => {
                let lhs_anal = self.expr(path, scope, lhs);
                let rhs_anal = self.expr(path, scope, rhs);

                if !lhs_anal.ty.is_int(&mut self.syms) {
                    let range = self.ast.range(lhs);
                    return Err(Error::InvalidRange { source: range, ty: lhs_anal.ty });
                }


                if !rhs_anal.ty.is_int(&mut self.syms) {
                    let range = self.ast.range(rhs);
                    return Err(Error::InvalidRange { source: range, ty: rhs_anal.ty });
                }


                AnalysisResult::new(Type::RANGE)
            },


            Expr::BinaryOp { operator, lhs, rhs } => {
                let lhs_anal = self.expr(path, scope, lhs);
                let rhs_anal = self.expr(path, scope, rhs);

                lhs_anal.ty.eq(&mut self.syms, rhs_anal.ty);

                let lhs_sym = lhs_anal.ty.sym(&mut self.syms)?;

                if lhs_sym == SymbolId::ERR { return Ok(AnalysisResult::error()) }
                if lhs_sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                let rhs_sym = rhs_anal.ty.sym(&mut self.syms)?;

                if rhs_sym == SymbolId::ERR { return Ok(AnalysisResult::error()) }
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


                let validate = validate()?;

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
                    return Err(Error::InvalidBinaryOp {
                        operator, lhs: lhs_anal.ty, rhs: rhs_anal.ty, source: range });

                }


                let lhs_sym = lhs_anal.ty.sym(&self.syms)?;
                let traits = self.syms.traits(lhs_sym);

                if traits.contains_key(&SymbolId::EQ_TRAIT) {
                    return Ok(AnalysisResult::new(Type::BOOL));
                }

                return Err(Error::TypeDoesntImplTrait { source: range, ty: lhs_anal.ty, tr: SymbolId::EQ_TRAIT });
            },


            Expr::UnaryOp { operator, rhs } => {
                let rhs_anal = self.expr(path, scope, rhs);
                let sym = rhs_anal.ty.sym(&mut self.syms)?;

                if sym == SymbolId::ERR { return Ok(AnalysisResult::error()) }
                if sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }

                match operator {
                    UnaryOperator::Not if sym == SymbolId::BOOL => (),
                    UnaryOperator::Neg if sym.is_num() => (),
                    
                    _ => return Err(Error::InvalidUnaryOp { operator, rhs: rhs_anal.ty, source: range })
                }

                AnalysisResult::new(rhs_anal.ty)
            },


            Expr::If { condition, body, else_block } => {
                let cond = self.expr(path, scope, condition);

                if let Ok(sym) = cond.ty.sym(&mut self.syms) {
                    if sym == SymbolId::ERR { return Ok(AnalysisResult::error()) }
                    if sym == SymbolId::NEVER { return Ok(AnalysisResult::never()) }
                }

                if !cond.ty.eq(&mut self.syms, Type::BOOL) {
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
                    return Err(Error::IfMissingElse { body: (body, value) })
                }

                AnalysisResult::new(value)
            },


            Expr::Match { value, mappings  } => {
                let anal = self.expr(path, scope, value);

                let sym = anal.ty.sym(&mut self.syms)?;
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else {
                    let range = self.ast.range(value);
                    return Err(Error::MatchValueIsntEnum { source: range, typ: anal.ty });
                };

                // check if the value is an enum
                if !matches!(cont.kind(), ContainerKind::Enum) {
                    let range = self.ast.range(value);
                    return Err(Error::MatchValueIsntEnum { source: range, typ: anal.ty });
                }

                // check the mapping names
                for (i, m) in mappings.iter().enumerate() {
                    let exists = cont.fields().iter().any(|x| {
                        let name = x.0;
                        m.variant() == name
                    });

                    if !exists {
                        return Err(Error::InvalidMatch {
                            name: m.variant(), range: m.range(), value: anal.ty });
                    }

                    for o in mappings.iter().skip(i+1) {
                        if o.variant() == m.variant() {
                            return Err(Error::DuplicateMatch {
                                declared_at: m.range(), error_point: o.range() });
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
                    return Err(Error::MissingMatch { name: KVec::from_slice(&missings), range });
                }


                // ty chck
                let ret_ty = self.syms.new_var(id, range);
                for (m, f) in mappings.iter().zip(cont.fields().iter()) {
                    let gens = anal.ty.gens(&self.syms);
                    let gens = self.syms.get_gens(gens);
                    let vs = VariableScope::new(m.binding(), f.1.to_ty(gens, &mut self.syms)?);

                    let scope = Scope::new(Some(scope), ScopeKind::VariableScope(vs));
                    let scope = self.scopes.push(scope);

                    let anal = self.expr(path, scope, m.expr());

                    if !anal.ty.eq(&mut self.syms, ret_ty) {
                        let range = self.ast.range(m.expr());
                        self.error(m.expr(), Error::InvalidType {
                            source: range, found: anal.ty, expected: ret_ty });
                    }

                }
                

                AnalysisResult::new(ret_ty)
            },


            Expr::Block { block } => self.block(path, scope, &*block),


            Expr::CreateStruct { data_type, fields  } => {
                let ty = self.dt_to_ty(scope, id, data_type)?;

                let sym = ty.sym(&mut self.syms)?;
                let sym = self.syms.sym(sym);

                let SymbolKind::Container(cont) = sym.kind()
                else { return Err(Error::StructCreationOnNonStruct { source: range, typ: ty }) };

                // check if the sym is a struct
                if !matches!(cont.kind(), ContainerKind::Struct) {
                    return Err(Error::StructCreationOnNonStruct { source: range, typ: ty });
                }

                // check if the fields are valid
                for f in fields.iter() {
                    let exists = cont.fields().iter().any(|x| {
                        let name = x.0;
                        name == f.0
                    });

                    if !exists {
                        return Err(Error::FieldDoesntExist {
                            source: f.1, field: f.0, typ: ty });
                    }
                }


                // check missing fields
                let mut missing_fields = Vec::new_in(self.temp);
                for f in cont.fields().iter() {
                    let name = f.0;

                    if !fields.iter().any(|x| x.0 == name) {
                        missing_fields.push(name);
                    }
                }

                if !missing_fields.is_empty() {
                    return Err(Error::MissingFields { source: range, fields: missing_fields.clone_in(GlobalAlloc) });
                }


                // type check the fields
                let sym_fields = {
                    let mut vec = Buffer::new(self.temp, cont.fields().len());
                    let gens = ty.gens(&mut self.syms);
                    let gens = self.syms.get_gens(gens);

                    for f in cont.fields() {
                        //dbg!(f);
                        vec.push((f.0, f.1.to_ty(gens, &mut self.syms)?))
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
                    return Ok(AnalysisResult::error())
                }

                let sym_id = expr.ty.sym(&mut self.syms)?;
                let sym = self.syms.sym(sym_id);

                let field_check = 'b: {
                    let SymbolKind::Container(cont) = sym.kind()
                    else { break 'b Err(Error::FieldAccessOnNonEnumOrStruct { source: range, typ: expr.ty }) };

                    let field = cont.fields().iter().enumerate().find(|(_, f)| {
                        let name = f.0;
                        field_name == name
                    });

                    let Some((_, field)) = field
                    else { break 'b Err(Error::FieldDoesntExist {
                        source: range, field: field_name, typ: expr.ty }) };
                    Ok((field, cont))
                };

                // if its a normal field
                let e = match field_check {
                    Ok((field, cont)) => {
                        let gens = expr.ty.gens(&self.syms);
                        let gens = self.syms.get_gens(gens);

                        let field_gen = field.1;
                        let field_ty = field_gen.to_ty(gens, &mut self.syms)?;

                        let ty = match cont.kind() {
                            ContainerKind::Struct => field_ty,

                            ContainerKind::Enum => {
                                let gens = self.output.alloc_new([(BoundedGeneric::T, field_ty)]);
                                Type::Ty(SymbolId::OPTION, self.syms.add_gens(gens))
                            },

                            ContainerKind::Tuple => field_ty,

                            ContainerKind::Generic => unreachable!(),
                        };

                        if let Some(e) = field_gen.err() {
                            self.type_info.exprs[id] = Some(crate::ExprInfo::Errored(e));
                        }

                        return Ok(AnalysisResult::new(ty))
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
                        let var = self.syms.new_var(id, range);
                        vgens.push((*g, var));
                    }

                    let sym_gens = self.syms.get_gens(expr.ty.gens(&self.syms));

                    //assert!(sym_gens.iter().zip(&vgens).all(|(a, b)| a.0 == b.1.0));

                    for ((n0, g0), (_, (n1, g1))) in sym_gens.iter().zip(&vgens) {
                        if n0 == n1 {
                            (*g0).eq(&mut self.syms, *g1);
                        }
                    }

                    if let Some(gens) = expr_gens {
                        for (g, (_, s)) in gens.iter().zip(vgens.iter().skip(sym_gens.len())) {
                            let ty = self.dt_to_ty(scope, id, *g);

                            let ty = match ty {
                                Ok(v) => v,
                                Err(v) => {
                                    self.error(id, v);
                                    continue;
                                },
                            };

                            if !ty.eq(&mut self.syms, *s) {
                                self.error(id, Error::InvalidType { source: range, found: *s, expected: ty });
                            }
                        }
                    }

                    let gens = self.syms.add_gens(vgens.leak());



                    let SymbolKind::Function(func) = sym.kind()
                    else { return Err(Error::CallOnNonFunction { source: range }) };
                    let anal = match func.kind() {
                        FunctionKind::Closure(_) => AnalysisResult::new(Type::Ty(sym_id, expr.ty.gens(&self.syms))),
                        _ => {
                            let closure = self.syms.new_closure();

                            let sym = self.func_sym(closure, func.args(), func.ret(), sym.generics());

                            AnalysisResult::new(Type::Ty(sym, gens))
                        }
                    };

                    return Ok(anal);
                }

                // try to find traits
                let mut candidate = None;
                let candidates = self.syms.traits(sym_id).clone();

                self.scopes.get(scope)
                .over::<()>(&self.scopes,
                |scope| {
                    let ScopeKind::ImplicitNamespace(ns) = scope.kind()
                    else { return None };

                    let ns = self.namespaces.get_ns(ns);

                    for s in ns.syms().values() {
                        let Ok(s) = s
                        else { continue };

                        let Some((_, g, generics)) = candidates.get(s)
                        else { continue; };

                        let sym = self.syms.sym(*s);
                        let SymbolKind::Trait(tr) = sym.kind()
                        else { continue; };

                        let Some(ft) = tr.funcs.iter().find(|x| x.0 == field_name)
                        else { continue; };

                        if candidate.is_none() {
                            candidate = Some((*s, ft.1, *g, generics));
                            return Some(());
                        } else {
                            todo!("ambigious");
                        }

                    }

                    None

                });


                let sym = ();
                let Some((t, func, g, generics)) = candidate
                else { return Err(e); };

                let mut vgens = sti::vec::Vec::with_cap_in(self.output, generics.iter().len());

                for g in generics.iter() {
                    let var = self.syms.new_var(id, range);
                    vgens.push((*g, var));
                }

                let sym_gens = self.syms.get_gens(expr.ty.gens(&self.syms));

                assert!(sym_gens.iter().zip(&vgens).all(|(a, b)| a.0 == b.1.0));

                for ((n0, g0), (_, (n1, g1))) in sym_gens.iter().zip(&vgens) {
                    if n0 == n1 {
                        (*g0).eq(&mut self.syms, *g1);
                    }
                }

                if let Some(gens) = expr_gens {
                    for (g, (_, s)) in gens.iter().zip(vgens.iter().skip(sym_gens.len())) {
                        let ty = self.dt_to_ty(scope, id, *g);

                        let ty = match ty {
                            Ok(v) => v,
                            Err(v) => {
                                self.error(id, v);
                                continue;
                            },
                        };

                        if !ty.eq(&mut self.syms, *s) {
                            self.error(id, Error::InvalidType { source: range, found: *s, expected: ty });
                        }
                    }
                }

                let gens = self.syms.add_gens(vgens.leak());

                let closure = self.syms.new_closure();

                let mut func_args = sti::vec::Vec::with_cap_in(self.output, func.args().len());
                for arg in func.args() {
                    let gn = arg.symbol()
                        .rec_replace(self.output, StringMap::SELF_TY, g);
                    func_args.push(FunctionArgument::new(arg.name(), gn));
                }

                let ret = func.ret()
                    .rec_replace(self.output, StringMap::SELF_TY, g);

                let sym = self.func_sym(closure, func_args.leak(), ret, generics);
                self.type_info.set_acc(id, t);

                AnalysisResult::new(Type::Ty(sym, gens))
            },


            Expr::CallFunction { lhs: lhs_expr, args } => {
                dbg!(self.ast.expr(lhs_expr));
                let lhs = self.expr(path, scope, lhs_expr);
                
                if lhs.ty.is_err(&mut self.syms) {
                    return Ok(AnalysisResult::error())
                }

                let lhs_range = self.ast.range(lhs_expr);
                let sym_id = lhs.ty.sym(&mut self.syms)?;

                let pool = self.ast.arena;
                let mut is_accessor = false;
                let args_anals = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, args.len());

                    if let Expr::AccessField { val, field_name, .. } = self.ast.expr(lhs_expr) {
                        let range = self.ast.range(val);
                        let anal = self.expr(path, scope, val);

                        // check if it's a field or not
                        let sym = anal.ty;
                        let sym = sym.sym(&mut self.syms)?;
                        let sym = self.syms.sym(sym);

                        if let SymbolKind::Container(cont) = sym.kind()
                        && cont.fields().iter().find(|x| x.0 == field_name).is_some() {
                            return Err(Error::CallOnField { source: lhs_range, field_name })
                        } else {
                            is_accessor = true;
                            vec.push((range, Some(anal), val));
                        }
                    }

                    for a in args {
                        vec.push((self.ast.range(*a), None, *a));
                    }

                    vec.leak()
                };

                let sym = self.syms.sym(sym_id);
                let SymbolKind::Function(func) = sym.kind()
                else { return Err(Error::CallOnNonFunction { source: lhs_range }); };

                let f_gens = lhs.ty.gens(&self.syms);
                let gens = self.syms.get_gens(f_gens);

                // check arg len
                if func.args().len() != args_anals.len() {
                    return Err(Error::FunctionArgsMismatch {
                        source: range, sig_len: func.args().len() - if is_accessor { 1 } else { 0 }, call_len: args.len() });
                }

                // find out the args
                let func_args = {
                    let mut vec = sti::vec::Vec::with_cap_in(&*pool, func.args().len());
                    for g in func.args() {
                        vec.push(g.symbol().to_ty(gens, &mut self.syms)?);
                    }

                    vec
                };

                let ret = func.ret().to_ty(gens, &mut self.syms)?;

                // ty check args
                for (a, &fa) in args_anals.iter().zip(func_args.iter()) {
                    let anal = self.expr_ex(path, scope, a.2, Some(fa));

                    if !anal.ty.eq(&mut self.syms, fa) {
                        self.error(a.2, Error::InvalidType {
                            source: a.0, found: anal.ty, expected: fa });
                    }
                }

                for (sym_g, (func_g, value)) in sym.generics().iter().zip(gens.iter()) {
                    assert_eq!(sym_g.name(), func_g.name());

                    if sym_g.bounds.is_empty() { continue }

                    let sym = value.sym(&self.syms)?;
                    let traits = self.syms.traits(sym);

                    for bound in sym_g.bounds {
                        if traits.contains_key(bound) { continue }

                        self.error(
                            lhs_expr,
                            Error::TypeDoesntImplTrait { 
                                source: range, ty: *value, tr: *bound }
                        );
                        
                        return Ok(AnalysisResult::error())
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

                let Some(sym) = sym
                else { 
                    return Err(Error::NamespaceNotFound { 
                        source: namespace_source, 
                        namespace 
                    }) 
                };

                let Ok(sym) = sym
                else {
                    return Err(sym.unwrap_err());
                };

                println!("within ns of {}", Type::Ty(sym, GenListId::EMPTY).display(&mut self.string_map, &mut self.syms));
                let ns = self.syms.sym_ns(sym);

                let scope = Scope::new(scope, ScopeKind::NamespaceFence);
                let scope = self.scopes.push(scope);
                let scope = Scope::new(scope, ScopeKind::ImplicitNamespace(ns));
                let scope = self.scopes.push(scope);
                let scope = Scope::new(scope, ScopeKind::ImplicitTrait(sym));
                let scope = self.scopes.push(scope);


                self.expr(path, scope, action)
            },


            Expr::WithinTypeNamespace { namespace, action  } => {
                let ty = self.dt_to_ty(scope, id, namespace)?;
                let sym = ty.sym(&mut self.syms)?;
                let ns = self.syms.sym_ns(sym);

                let scope = Scope::new(Some(scope), ScopeKind::ImplicitNamespace(ns));
                let scope = self.scopes.push(scope);

                self.expr(path, scope, action)
            },


            Expr::Loop { body } => {
                let scope = Scope::new(Some(scope), ScopeKind::Loop);
                let scope = self.scopes.push(scope);
                self.block(path, scope, &*body);

                AnalysisResult::new(Type::UNIT)
            },


            Expr::Return(ret) => {
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return Err(Error::OutsideOfAFunction { source: range }) };

                let ret_anal = self.expr(path, scope, ret);
                if ret_anal.ty.is_err(&mut self.syms) { return Ok(AnalysisResult::error()) }
                if ret_anal.ty.is_never(&mut self.syms) { return Ok(AnalysisResult::never()) }

                if ret_anal.ty.ne(&mut self.syms, func.ret) {
                    return Err(Error::ReturnAndFuncTypDiffer {
                        source: range, func_source: func.ret_source,
                        typ: ret_anal.ty, func_typ: func.ret })
                }

                AnalysisResult::new(Type::NEVER)
            },


            Expr::Continue => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none() { 
                    return Err(Error::ContinueOutsideOfLoop(range)) 
                }

                AnalysisResult::new(Type::NEVER)
            },


            Expr::Break => {
                if self.scopes.get(scope).find_loop(&self.scopes).is_none() { 
                    return Err(Error::BreakOutsideOfLoop(range)) 
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
                let ty = self.syms.new_var(id, range);

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

                let sym = list.ty.sym(&mut self.syms)?;

                if sym == SymbolId::NEVER || sym == SymbolId::ERR { return Ok(AnalysisResult::new(list.ty)) }

                if sym != SymbolId::LIST {
                    return Err(Error::IndexOnNonList(range, list.ty));
                }

                if !index.ty.is_int(&mut self.syms) {
                    return Err(Error::InvalidType { source: range, found: index.ty, expected: Type::I64 })
                }

                let gens = list.ty.gens(&self.syms);
                let (_, ty) = self.syms.get_gens(gens)[0];

                AnalysisResult::new(ty)
            }


            Expr::AsCast { lhs, data_type  } => {
                let anal = self.expr(path, scope, lhs);
                let ty = self.dt_to_ty(scope, id, data_type)?;

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
                let sym = expr.ty.sym(&mut self.syms)?;
                if sym == SymbolId::ERR { return Ok(AnalysisResult::error()) }

                if sym != SymbolId::OPTION
                   && sym != SymbolId::RESULT {
                    return Err(Error::CantUnwrapOnGivenType(range, expr.ty));
                }

                let gens = expr.ty.gens(&self.syms);
                let gens = self.syms.get_gens(gens);
                
                AnalysisResult::new(gens[0].1)
            },


            Expr::OrReturn(val) => {
                let expr = self.expr(path, scope, val);
                let sym = expr.ty.sym(&mut self.syms)?;
                let Some(func) = self.scopes.get(scope).find_curr_func(&self.scopes)
                else { return Err(Error::OutsideOfAFunction { source: range }) };

                if sym == SymbolId::OPTION {
                    let func_sym = func.ret;
                    let opt_sym = {
                        let val = self.syms.new_var(id, range);
                        let gens = self.output.alloc_new([(BoundedGeneric::T, val)]);
                        let gens = self.syms.add_gens(gens);

                        Type::Ty(SymbolId::OPTION, gens)
                    };

                    if !opt_sym.eq(&mut self.syms, func_sym) {
                        return Err(Error::FunctionDoesntReturnAnOption { source: range, func_typ: func.ret });
                    }

                    let gens = expr.ty.gens(&self.syms);
                    let gens = self.syms.get_gens(gens);

                    return Ok(AnalysisResult::new(gens[0].1));
                }

                
                if sym == SymbolId::RESULT {
                    let res_sym = {
                        let val = self.syms.new_var(id, range);
                        let gens = self.output.alloc_new([(BoundedGeneric::T, val), (BoundedGeneric::A, val)]);
                        let gens = self.syms.add_gens(gens);

                        Type::Ty(SymbolId::RESULT, gens)
                    };

                    if !res_sym.eq(&mut self.syms, func.ret) {
                        return Err(Error::FunctionDoesntReturnAResult { source: range, func_typ: func.ret });
                    }

                    let func_gens = func.ret.gens(&self.syms);
                    let func_gens = self.syms.get_gens(func_gens);

                    let gens = expr.ty.gens(&self.syms);
                    let gens = self.syms.get_gens(gens);

                    debug_assert_eq!(func_gens.len(), 2);
                    debug_assert_eq!(gens.len(), 2);

                    if !func_gens[1].1.eq(&mut self.syms, gens[1].1) {
                        return Err(Error::FunctionReturnsAResultButTheErrIsntTheSame {
                            source: range, func_source: func.ret_source,
                            func_err_typ: func_gens[1].1, err_typ: gens[1].1 });
                    }

                    return Ok(AnalysisResult::new(gens[0].1));
                }


                return Err(Error::CantTryOnGivenType(range, expr.ty));
            },


        })})();


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

