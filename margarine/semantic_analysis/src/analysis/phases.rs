use crate::analysis::blocks::{BlockId, BlockKind, PhaseState};

use super::*;

impl<'me, 'out, 'temp, 'ast: 'out, 'str> TyChecker<'me, 'out, 'temp, 'ast, 'str> {
    pub(crate) fn ensure_names(&mut self, id: BlockId) {
        match self.blocks.block_state(id).names {
            PhaseState::Done => return,
            PhaseState::InProgress => unreachable!("unexpected block name cycle"),
            PhaseState::Uninit => (),
        }

        self.blocks.set_block_phase(id, |state| state.names = PhaseState::InProgress);

        let (parent, kind) = {
            let state = self.blocks.block_state(id);
            (state.parent, state.kind)
        };

        match kind {
            BlockKind::Ordinary => {
                if let Some(parent) = parent {
                    self.ensure_names(parent);
                }

                if self.blocks.block_state(id).origin.is_none() {
                    let (path, ns) = {
                        let state = self.blocks.block_state(id);
                        (state.path.unwrap(), state.ns.unwrap())
                    };
                    let gen_count = self.blocks.block_generic_count(id, self.ast, &self.type_info);
                    let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
                    for index in 0..len {
                        let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
                        self.collect_names(path, ns, std::slice::from_ref(&node), gen_count);
                    }
                } else {
                    let Some(parent) = parent
                    else {
                        unreachable!("non-root block has no parent");
                    };
                    let parent_ns = self.blocks.block_namespace(parent)
                        .expect("named parent has no namespace");
                    let parent_scope = self.blocks.block_scope(parent)
                        .expect("named parent has no scope");

                    let (path, ns) = match self.blocks.block_state(id).origin {
                        Some(NodeId::Decl(decl_id)) => match self.ast.decl(decl_id) {
                            parser::nodes::decl::Decl::Module { name, .. } => {
                                match self.namespaces.get_ns(parent_ns).get_sym(name) {
                                    Some(Ok(sym)) => {
                                        let ns = self.syms.as_ns(sym);
                                        (self.namespaces.get_ns(ns).path, ns)
                                    },
                                    _ => {
                                        let path = self.string_map.concat(
                                            self.blocks.block_state(parent).path
                                                .expect("named parent has no path"),
                                            name,
                                        );
                                        let ns = self.namespaces.push(
                                            Namespace::new(path),
                                            Some(parent_ns),
                                        );
                                        (path, ns)
                                    },
                                }
                            },

                            parser::nodes::decl::Decl::Function { sig, .. } => {
                                let path = self.namespaces.get_ns(parent_ns).get_sym(sig.name)
                                    .and_then(|result| result.ok())
                                    .map(|_| sig.name)
                                    .unwrap_or_else(|| self.string_map.concat(
                                        self.blocks.block_state(parent).path.unwrap(),
                                        sig.name,
                                    ));
                                let ns = self.namespaces.push(
                                    Namespace::new(path),
                                    Some(parent_ns),
                                );
                                (path, ns)
                            },

                            _ => unreachable!("ordinary block origin is not a module or function"),
                        },

                        Some(NodeId::Expr(_)) | Some(NodeId::Stmt(_)) => {
                            let path = self.blocks.block_state(parent).path.unwrap();
                            let ns = self.namespaces.push(
                                Namespace::new(path),
                                Some(parent_ns),
                            );
                            (path, ns)
                        },
                        None | Some(NodeId::Err(_)) => unreachable!("child block has invalid origin"),
                    };

                    let scope = self.scopes.push(Scope::new(
                        Some(parent_scope),
                        ScopeKind::ImplicitNamespace(ns),
                    ));
                    self.blocks.set_block_phase(id, |state| {
                        state.ns = Some(ns);
                        state.path = Some(path);
                        state.collect_scope = Some(scope);
                        state.ty_scope = Some(scope);
                    });
                    let gen_count = self.blocks.block_generic_count(id, self.ast, &self.type_info);
                    let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
                    for index in 0..len {
                        let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
                        self.collect_names(path, ns, std::slice::from_ref(&node), gen_count);
                    }
                }
            },

            BlockKind::Method { impl_decl } => {
                let Some(parent) = parent else {
                    unreachable!("method block has no parent");
                };
                self.ensure_impls(parent);

                let parent_ns = self.blocks.block_namespace(parent)
                    .expect("named method parent has no namespace");
                let parent_scope = self.blocks.block_scope(parent)
                    .expect("named method parent has no scope");
                let impl_ns = self.blocks.impl_namespace(
                    impl_decl,
                    self.ast,
                    &self.type_info,
                    &mut self.syms,
                )
                .unwrap_or_else(|| {
                    let path = self.blocks.block_state(parent).path
                        .expect("named method parent has no path");
                    self.namespaces.push(Namespace::new(path), Some(parent_ns))
                });
                let decl_id = match self.blocks.block_state(id).origin {
                    Some(NodeId::Decl(decl_id)) => decl_id,
                    _ => unreachable!("method block origin is not a declaration"),
                };
                let Some((name, fallback_path)) = (match self.ast.decl(decl_id) {
                    parser::nodes::decl::Decl::Function { sig, .. } => Some((
                        sig.name,
                        self.namespaces.get_ns(impl_ns).path,
                    )),
                    _ => None,
                }) else {
                    unreachable!("method block declaration is not a function");
                };
                if self.namespaces.get_ns(impl_ns).get_sym(name).is_none() {
                    let gen_count = self.type_info.impls.get(&impl_decl)
                        .expect("method implementation metadata missing")
                        .2.len();
                    self.collect_names(fallback_path, impl_ns, &[decl_id.into()], gen_count);
                }

                let path = self.namespaces.get_ns(impl_ns).get_sym(name)
                    .and_then(|result| result.ok())
                    .map(|_| name)
                    .unwrap_or_else(|| self.string_map.concat(fallback_path, name));

                // Keep the body namespace in the lexical module hierarchy.
                // The implementation namespace remains in the scope chain for
                // receiver and method lookup.
                let ns = self.namespaces.push(Namespace::new(path), Some(parent_ns));
                let impl_scope = self.scopes.push(Scope::new(
                    Some(parent_scope),
                    ScopeKind::ImplicitNamespace(impl_ns),
                ));
                let scope = self.scopes.push(Scope::new(
                    Some(impl_scope),
                    ScopeKind::ImplicitNamespace(ns),
                ));

                self.blocks.set_block_phase(id, |state| {
                    state.ns = Some(ns);
                    state.path = Some(path);
                    state.collect_scope = Some(scope);
                    state.ty_scope = Some(scope);
                });
                let gen_count = self.blocks.block_generic_count(id, self.ast, &self.type_info);
                let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
                for index in 0..len {
                    let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
                    self.collect_names(path, ns, std::slice::from_ref(&node), gen_count);
                }
            },
        }

        self.blocks.set_block_phase(id, |state| state.names = PhaseState::Done);
    }


    pub(crate) fn ensure_uses(&mut self, id: BlockId) {
        match self.blocks.block_state(id).uses {
            PhaseState::Done => return,
            PhaseState::InProgress => unreachable!("unexpected block use cycle"),
            PhaseState::Uninit => (),
        }
        self.blocks.set_block_phase(id, |state| state.uses = PhaseState::InProgress);
        self.ensure_names(id);

        let scope = self.blocks.block_scope(id)
            .expect("named block has no collection scope");
        let ns = self.blocks.block_namespace(id)
            .expect("named block has no namespace");
        let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
        for index in 0..len {
            let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
            self.collect_uses(scope, ns, std::slice::from_ref(&node));
        }
        self.blocks.set_block_phase(id, |state| state.uses = PhaseState::Done);
    }


    pub(crate) fn ensure_impls(&mut self, id: BlockId) {
        match self.blocks.block_state(id).impls {
            PhaseState::Done => return,
            PhaseState::InProgress => unreachable!("unexpected block impl cycle"),
            PhaseState::Uninit => (),
        }
        self.blocks.set_block_phase(id, |state| state.impls = PhaseState::InProgress);
        self.ensure_uses(id);

        let scope = self.blocks.block_scope(id)
            .expect("named block has no collection scope");
        let ns = self.blocks.block_namespace(id)
            .expect("named block has no namespace");
        let path = self.blocks.block_state(id).path
            .expect("named block has no path");
        let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
        for index in 0..len {
            let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
            self.collect_impls(path, scope, ns, std::slice::from_ref(&node));
        }
        self.blocks.set_block_phase(id, |state| state.impls = PhaseState::Done);
    }


    fn install_function_ty_scope(&mut self, block_id: BlockId) {
        let Some(NodeId::Decl(id)) = self.blocks.block_state(block_id).origin
        else { unreachable!("function type scope has no declaration origin") };
        let scope = self.blocks.block_scope(block_id)
            .expect("function block has no collection scope");
        let Some(ns) = self.blocks.function_lookup_namespace(
            block_id,
            self.ast,
            &self.type_info,
            &mut self.syms,
        )
        else { return };
        let name = match self.ast.decl(id) {
            parser::nodes::decl::Decl::Function { sig, .. } => sig.name,
            _ => unreachable!("function type scope origin is not a function"),
        };
        let Some(Ok(func_id)) = self.namespaces.get_ns(ns).get_sym(name)
        else { return };

        let function_generics: std::vec::Vec<_> =
            self.syms.sym(func_id).generics().iter().copied().collect();
        let source = match self.ast.decl(id) {
            parser::nodes::decl::Decl::Function { sig, .. } => sig.source,
            _ => unreachable!("function type scope origin is not a function"),
        };

        let mut generics = Buffer::new(self.output, function_generics.len());
        for generic in function_generics {
            let ty = self.syms.pending(&mut self.namespaces, None, generic.name(), 0);
            let kind = SymbolKind::Container(Container::new(&[], ContainerKind::Generic));
            self.syms.add_sym(ty, Symbol::new(generic.name(), &[], kind));

            for &bound in generic.bounds {
                let Some(trait_id) = bound.sym()
                else { continue };
                let bound_error = self.validate_trait_bound(id.into(), bound, &[]);
                self.syms.traits(ty).entry(trait_id).or_default().push(TraitImplEntry {
                    namespace: NamespaceId::MAX,
                    trait_ty: bound,
                    receiver: Generic::new(source, GenericKind::Sym(ty, &[])),
                    generics: &[],
                    declaration: None,
                    bound_error,
                });
            }
            generics.push((generic, self.syms.get_ty(ty, &[])));
        }

        let mut base_scope = scope;
        if let BlockKind::Method { impl_decl } = self.blocks.block_state(block_id).kind
        && let Some((_, receiver, _)) = self.type_info.impls.get(&impl_decl)
        {
            base_scope = self.scopes.push(Scope::new(
                Some(base_scope),
                ScopeKind::AliasDecl(StringMap::SELF_TY, *receiver),
            ));
        }

        let generics = generics.leak();
        let ty_scope = self.scopes.push(Scope::new(
            Some(base_scope),
            ScopeKind::Generics(GenericsScope::new(generics)),
        ));
        self.blocks.set_block_phase(block_id, |state| state.ty_scope = Some(ty_scope));
    }
    fn finalize_method_signature(&mut self, block_id: BlockId, impl_decl: DeclId) {
        let Some(NodeId::Decl(id)) = self.blocks.block_state(block_id).origin
        else { unreachable!("method signature has no declaration origin") };
        let Some((_, receiver, impl_generics)) =
            self.type_info.impls.get(&impl_decl).copied()
        else { return };
        let Some(receiver_sym) = receiver.sym()
        else { return };
        let parent = self.blocks.parent_block(block_id)
            .expect("method block has no parent");
        let scope = self.blocks.block_scope(parent)
            .expect("method parent has no collection scope");
        let Some(ns) = self.blocks.impl_namespace(
            impl_decl,
            self.ast,
            &self.type_info,
            &mut self.syms,
        )
        else { return };
        let name = match self.ast.decl(id) {
            parser::nodes::decl::Decl::Function { sig, .. } => sig.name,
            _ => unreachable!("method signature origin is not a function"),
        };
        let Some(Ok(func_id)) = self.namespaces.get_ns(ns).get_sym(name)
        else { return };
        if self.syms.sym_ok(func_id).is_some() {
            return;
        }

        let scope = self.scopes.push(Scope::new(
            Some(scope),
            ScopeKind::AliasDecl(StringMap::SELF_TY, receiver),
        ));
        let path = self.blocks.block_state(parent).path
            .expect("method parent has no path");
        self.compute_types(
            path,
            scope,
            ns,
            &[id.into()],
            Some((receiver_sym, impl_generics, None)),
        );
    }



    pub(crate) fn ensure_types(&mut self, id: BlockId) {
        match self.blocks.block_state(id).types {
            PhaseState::Done => return,
            PhaseState::InProgress => unreachable!("unexpected block type cycle"),
            PhaseState::Uninit => (),
        }
        if let Some(parent) = self.blocks.parent_block(id) {
            self.ensure_types(parent);
        }
        self.blocks.set_block_phase(id, |state| state.types = PhaseState::InProgress);
        self.ensure_impls(id);

        if let Some(NodeId::Decl(decl)) = self.blocks.block_state(id).origin
        && matches!(self.ast.decl(decl), parser::nodes::decl::Decl::Function { .. })
        {
            if let BlockKind::Method { impl_decl } = self.blocks.block_state(id).kind {
                self.finalize_method_signature(id, impl_decl);
            }
            self.install_function_ty_scope(id);
        }

        let scope = self.blocks.block_ty_scope(id)
            .expect("named block has no type scope");
        let ns = self.blocks.block_namespace(id)
            .expect("named block has no namespace");
        let path = self.blocks.block_state(id).path
            .expect("named block has no path");
        let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
        for index in 0..len {
            let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
            self.compute_types(path, scope, ns, std::slice::from_ref(&node), None);
        }
        self.blocks.set_block_phase(id, |state| state.types = PhaseState::Done);

        for child in self.blocks.direct_child_block_ids(id, self.ast, self.root_nodes) {
            if matches!(self.blocks.block_state(child).origin,
                Some(NodeId::Decl(decl))
                if matches!(self.ast.decl(decl), parser::nodes::decl::Decl::Module { .. }))
            {
                self.ensure_types(child);
            }
        }
    }


    pub(crate) fn ensure_validate(&mut self, id: BlockId) {
        match self.blocks.block_state(id).validate {
            PhaseState::Done => return,
            PhaseState::InProgress => unreachable!("unexpected block validation cycle"),
            PhaseState::Uninit => (),
        }
        self.blocks.set_block_phase(id, |state| state.validate = PhaseState::InProgress);
        self.ensure_types(id);

        let scope = self.blocks.block_ty_scope(id)
            .expect("named block has no type scope");
        let ns = self.blocks.block_namespace(id)
            .expect("named block has no namespace");
        let path = self.blocks.block_state(id).path
            .expect("named block has no path");
        let len = self.blocks.block_nodes(id, self.ast, self.root_nodes).len();
        for index in 0..len {
            let node = self.blocks.block_nodes(id, self.ast, self.root_nodes)[index];
            self.validate_types(path, scope, ns, std::slice::from_ref(&node), None);
        }
        self.blocks.set_block_phase(id, |state| state.validate = PhaseState::Done);

        for child in self.blocks.direct_child_block_ids(id, self.ast, self.root_nodes) {
            if matches!(self.blocks.block_state(child).origin,
                Some(NodeId::Decl(decl))
                if matches!(self.ast.decl(decl), parser::nodes::decl::Decl::Module { .. }))
            {
                self.ensure_validate(child);
            }
        }
    }
}
