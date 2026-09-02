use common::buffer::Buffer;
use common::string_map::StringIndex;
use parser::nodes::decl::{Decl, DeclId};
use parser::nodes::expr::{Expr, ExprId};
use parser::nodes::stmt::{Stmt, StmtId};
use parser::nodes::{NodeId, AST};
use sti::arena::Arena;
use sti::define_key;
use sti::vec::KVec;
use crate::namespace::NamespaceId;
use crate::syms::sym_map::{BoundedGeneric, SymbolMap};
use crate::TyInfo;
use crate::scope::{ScopeId, ScopeMap};


define_key!(pub BlockId(pub u32));


#[derive(Default)]
pub struct Blocks {
    pub(crate) entries: KVec<BlockId, BlockState>,
    pub(crate) decls : KVec<DeclId, Option<BlockId>>,
    pub(crate) exprs : KVec<ExprId, Option<BlockId>>,
    pub(crate) stmts : KVec<StmtId, Option<BlockId>>,
}


pub struct BlockDescriptor {
    pub node: NodeId,
    pub kind: BlockKind,
}


#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PhaseState {
    #[default]
    Uninit,
    InProgress,
    Done,
}


#[derive(Clone, Copy, Debug, Default)]
pub enum BlockKind {
    #[default]
    Ordinary,
    Method { impl_decl: DeclId },
}


#[derive(Default)]
pub struct BlockState {
    pub origin: Option<NodeId>,
    pub parent: Option<BlockId>,
    pub kind: BlockKind,
    pub ns: Option<NamespaceId>,
    pub path: Option<StringIndex>,
    pub collect_scope: Option<ScopeId>,
    pub ty_scope: Option<ScopeId>,
    pub names: PhaseState,
    pub uses: PhaseState,
    pub impls: PhaseState,
    pub types: PhaseState,
    pub validate: PhaseState,
}


impl Blocks {
    pub(crate) fn block_nodes<'a>(
        &self,
        id: BlockId,
        ast: &'a AST<'_>,
        root_nodes: &'a [NodeId],
    ) -> &'a [NodeId] {
        match self.block_state(id).origin {
            None => root_nodes,

            Some(NodeId::Decl(id)) => match ast.decl(id) {
                  Decl::Function { body, .. }
                | Decl::Module { body, .. } => &*body,

                _ => unreachable!("non-block declaration"),
            },

            Some(NodeId::Expr(id)) => match ast.expr(id) {
                  Expr::Block { block } 
                | Expr::Loop { body: block } => &*block,

                _ => unreachable!("non-block expression"),
            },

            Some(NodeId::Stmt(id)) => match ast.stmt(id) {
                Stmt::ForLoop { body, .. } => &*body,

                _ => unreachable!("non-block statement"),
            },

            Some(NodeId::Err(_)) => unreachable!("error node cannot own a block"),
        }
    }


    pub(crate) fn all_generics<'out>(
        &self,
        output: &'out Arena,
        scopes: &ScopeMap<'out>,
        scope: ScopeId,
        declared: &[BoundedGeneric<'out>],
        prefix: &[BoundedGeneric<'out>],
    ) -> &'out [BoundedGeneric<'out>] {
        let mut inherited = std::vec::Vec::new();
        scopes.get(scope).collect_generics(scopes, &mut inherited);

        let mut all = Buffer::<BoundedGeneric<'out>, _>::new(
            output,
            prefix.len() + inherited.len() + declared.len(),
        );

        for generic in prefix.iter()
            .chain(inherited.iter().map(|(generic, _)| generic))
            .chain(declared.iter()) {
            if !all.iter().any(|existing| existing.name() == generic.name()) {
                all.push(*generic);
            }
        }
        all.leak()
    }


    pub(crate) fn block_generic_count(
        &self,
        id: BlockId,
        ast: &AST<'_>,
        type_info: &TyInfo,
    ) -> usize {
        let mut count = 0;
        let mut current = Some(id);
        while let Some(block) = current {

            let state = self.block_state(block);
            if let BlockKind::Method { impl_decl } = state.kind {
                count += type_info.impls[&impl_decl].2.len()
            }
            
            if let Some(NodeId::Decl(decl)) = self.block_state(block).origin
            && let Decl::Function { sig, .. } = ast.decl(decl)
            {
                count += sig.generics.len();
            }
            current = state.parent;
        }
        count
    }


    pub(crate) fn direct_child_blocks<'a>(
        &self,
        id: BlockId,
        ast: &'a AST<'_>,
        root_nodes: &'a [NodeId],
    ) -> std::vec::Vec<BlockDescriptor> {
        let mut children = std::vec::Vec::new();
        for &node in self.block_nodes(id, ast, root_nodes) {
            self.discover_node(ast, node, &mut children, None);
        }

        children
    }


    pub(crate) fn direct_child_block_ids<'a>(
        &self,
        id: BlockId,
        ast: &'a AST<'_>,
        root_nodes: &'a [NodeId],
    ) -> std::vec::Vec<BlockId> {
        self.direct_child_blocks(id, ast, root_nodes)
            .into_iter()
            .map(|child| self.block_id(child.node))
            .collect()
    }


    pub(crate) fn discover_node(
        &self,
        ast: &AST<'_>,
        node: NodeId,
        children: &mut std::vec::Vec<BlockDescriptor>,
        impl_decl: Option<DeclId>,
    ) {
        match node {
            NodeId::Decl(id) => self.discover_decl(ast, id, children, impl_decl),
            NodeId::Stmt(id) => self.discover_stmt(ast, id, children, impl_decl),
            NodeId::Expr(id) => self.discover_expr(ast, id, children, impl_decl),
            NodeId::Err(_) => (),
        }
    }


    pub(crate) fn discover_decl(
        &self,
        ast: &AST<'_>,
        id: DeclId,
        children: &mut std::vec::Vec<BlockDescriptor>,
        impl_decl: Option<DeclId>,
    ) {
        match ast.decl(id) {
            Decl::Function { .. } => {
                let kind = impl_decl
                    .map(|impl_decl| BlockKind::Method { impl_decl })
                    .unwrap_or(BlockKind::Ordinary);

                children.push(BlockDescriptor { node: NodeId::Decl(id), kind });
            },

            Decl::Module { .. } => {
                children.push(BlockDescriptor { node: NodeId::Decl(id), kind: BlockKind::Ordinary });
            },

              Decl::Impl { body, .. }
            | Decl::ImplTrait { body, .. } => {
                for &node in *body {
                    self.discover_node(ast, node, children, Some(id));
                }
            },

            Decl::Attribute { decl, .. } => {
                self.discover_decl(ast, decl, children, impl_decl);
            },

            _ => (),
        }
    }


    pub(crate) fn discover_stmt(
        &self,
        ast: &AST<'_>,
        id: StmtId,
        children: &mut std::vec::Vec<BlockDescriptor>,
        impl_decl: Option<DeclId>,
    ) {
        match ast.stmt(id) {
            Stmt::ForLoop { .. } => {
                children.push(BlockDescriptor { node: NodeId::Stmt(id), kind: BlockKind::Ordinary });
            },

            Stmt::Variable { rhs, .. } => {
                self.discover_expr(ast, rhs, children, impl_decl);
            },

            Stmt::UpdateValue { lhs, rhs } => {
                self.discover_expr(ast, lhs, children, impl_decl);
                self.discover_expr(ast, rhs, children, impl_decl);
            },

            Stmt::Attribute { node, .. } => {
                self.discover_node(ast, node, children, impl_decl);
            },
        }
    }


    pub(crate) fn discover_expr(
        &self,
        ast: &AST<'_>,
        id: ExprId,
        children: &mut std::vec::Vec<BlockDescriptor>,
        impl_decl: Option<DeclId>,
    ) {
        match ast.expr(id) {
            Expr::Block { .. } | Expr::Loop { .. } => {
                children.push(BlockDescriptor { node: NodeId::Expr(id), kind: BlockKind::Ordinary });
            },

            Expr::Paren(expr)
            | Expr::UnaryOp { rhs: expr, .. }
            | Expr::AccessField { val: expr, .. }
            | Expr::Closure { body: expr, .. }
            | Expr::WithinNamespace { action: expr, .. }
            | Expr::WithinTypeNamespace { action: expr, .. }
            | Expr::Return(expr)
            | Expr::AsCast { lhs: expr, .. }
            | Expr::Unwrap(expr)
            | Expr::OrReturn(expr) => {
                self.discover_expr(ast, expr, children, impl_decl);
            },

            Expr::Range { lhs, rhs }
            | Expr::BinaryOp { lhs, rhs, .. }
            | Expr::IndexList { list: lhs, index: rhs } => {
                self.discover_expr(ast, lhs, children, impl_decl);
                self.discover_expr(ast, rhs, children, impl_decl);
            },

            Expr::If { condition, body, else_block } => {
                self.discover_expr(ast, condition, children, impl_decl);
                self.discover_expr(ast, body, children, impl_decl);
                if let Some(else_block) = else_block {
                    self.discover_expr(ast, else_block, children, impl_decl);
                }
            },

            Expr::Match { value, mappings } => {
                self.discover_expr(ast, value, children, impl_decl);
                for mapping in mappings {
                    self.discover_expr(ast, mapping.expr(), children, impl_decl);
                }
            },

            Expr::CreateStruct { fields, .. } => {
                for (_, _, expr) in fields {
                    self.discover_expr(ast, *expr, children, impl_decl);
                }
            },

            Expr::CallFunction { lhs, args } => {
                self.discover_expr(ast, lhs, children, impl_decl);
                for arg in args {
                    self.discover_expr(ast, arg.expr, children, impl_decl);
                }
            },

            Expr::Tuple(values) | Expr::CreateList { exprs: values } => {
                for &value in values {
                    self.discover_expr(ast, value, children, impl_decl);
                }
            },

            Expr::Unit | Expr::Literal(_) | Expr::Identifier(_, _)
            | Expr::Continue | Expr::Break => (),
        }
    }


    pub(crate) fn block_state(&self, id: BlockId) -> &BlockState {
        &self.entries[id]
    }


    pub(crate) fn set_block_phase(&mut self, id: BlockId, phase: impl FnOnce(&mut BlockState)) {
        phase(&mut self.entries[id]);
    }


    pub(crate) fn block_namespace(&self, id: BlockId) -> Option<NamespaceId> {
        self.block_state(id).ns
    }


    pub(crate) fn block_scope(&self, id: BlockId) -> Option<ScopeId> {
        self.block_state(id).collect_scope
    }


    pub(crate) fn block_ty_scope(&self, id: BlockId) -> Option<ScopeId> {
        self.block_state(id).ty_scope
    }


    pub(crate) fn parent_block(&self, id: BlockId) -> Option<BlockId> {
        self.block_state(id).parent
    }


    pub(crate) fn has_method_ancestor(&self, id: BlockId) -> bool {
        let mut current = self.parent_block(id);
        while let Some(parent) = current {
            if matches!(self.block_state(parent).kind, BlockKind::Method { .. }) {
                return true;
            }
            current = self.parent_block(parent);
        }
        false
    }


    pub(crate) fn function_lookup_namespace(
        &self,
        id: BlockId,
        ast: &AST<'_>,
        type_info: &TyInfo,
        syms: &mut SymbolMap,
    ) -> Option<NamespaceId> {
        let parent = self.parent_block(id);
        match self.block_state(id).kind {
            BlockKind::Method { impl_decl } =>
                self.impl_namespace(impl_decl, ast, type_info, syms),
            BlockKind::Ordinary => parent.and_then(|parent| self.block_namespace(parent)),
        }
    }


    pub(crate) fn impl_namespace(
        &self,
        id: DeclId,
        ast: &AST<'_>,
        type_info: &TyInfo,
        syms: &mut SymbolMap,
    ) -> Option<NamespaceId> {
        let (_, receiver, _) = type_info.impls.get(&id)?;
        let receiver = receiver.sym()?;

        match ast.decl(id) {
            Decl::Impl { .. } => Some(syms.sym_ns(receiver)),
            Decl::ImplTrait { .. } => syms.traits(receiver)
                .values()
                .flat_map(|entries| entries.iter())
                .find(|entry| entry.declaration == Some(id))
                .map(|entry| entry.namespace),
            _ => unreachable!("implementation block must be an impl declaration"),
        }
    }


    pub fn block_id(&self, node: NodeId) -> BlockId {
        let id =
        match node {
            NodeId::Decl(node) => self.decls[node],
            NodeId::Expr(node) => self.exprs[node],
            NodeId::Stmt(node) => self.stmts[node],
            NodeId::Err(_) => None,
        };

        id.expect("AST block was not discovered")
    }
}


impl std::ops::Index<BlockId> for Blocks {
    type Output = BlockState;

    fn index(&self, index: BlockId) -> &Self::Output {
        &self.entries[index]
    }
}
