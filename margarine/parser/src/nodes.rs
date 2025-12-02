pub mod expr;
pub mod stmt;
pub mod decl;
pub mod err;


use common::source::SourceRange;
use errors::ErrorId;
use sti::{arena::Arena, slice::KSlice, vec::KVec};

use self::{decl::{Decl, DeclId}, expr::{Expr, ExprId}, stmt::{Stmt, StmtId}};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeId {
    Decl(DeclId),
    Stmt(StmtId),
    Expr(ExprId),
    Err (ErrorId),
}


pub struct AST<'a> {
    stmts: KVec<StmtId, (Stmt<'a>, SourceRange)>,
    exprs: KVec<ExprId, (Expr<'a>, SourceRange)>,
    decls: KVec<DeclId, (Decl<'a>, SourceRange)>,
    pub arena: &'a Arena,
}

impl<'a> AST<'a> {
    pub fn new(arena: &'a Arena) -> Self {
        Self {
            stmts: KVec::new(),
            exprs: KVec::new(),
            decls: KVec::new(),
            arena, 
        }
    }


    pub fn range(&self, node: impl Into<NodeId>) -> SourceRange {
        match node.into() {
            NodeId::Stmt(e) => self.stmts[e].1,
            NodeId::Expr(e) => self.exprs[e].1,
            NodeId::Decl(e) => self.decls[e].1,
            NodeId::Err (_) => SourceRange::ZERO,
        }
    }


    pub fn add_stmt(&mut self, stmt: Stmt<'a>, src: SourceRange) -> StmtId {
        self.stmts.push((stmt, src))
    }


    pub fn add_expr(&mut self, expr: Expr<'a>, src: SourceRange) -> ExprId {
        self.exprs.push((expr, src))
    }


    pub fn add_decl(&mut self, decl: Decl<'a>, src: SourceRange) -> DeclId {
        self.decls.push((decl, src))
    }


    pub fn stmt(&self, stmt: StmtId) -> Stmt<'a> { self.stmts[stmt].0 }
    pub fn expr(&self, expr: ExprId) -> Expr<'a> { self.exprs[expr].0 }
    pub fn decl(&self, decl: DeclId) -> Decl<'a> { self.decls[decl].0 }
    pub fn set_decl(&mut self, decl_id: DeclId, decl: Decl<'a>) { self.decls[decl_id].0 = decl }


    pub fn stmts(&self) -> &KSlice<StmtId, (Stmt<'a>, SourceRange)> {
        self.stmts.as_kslice()
    }

    pub fn exprs(&self) -> &KSlice<ExprId, (Expr<'a>, SourceRange)> {
        self.exprs.as_kslice()
    }

    pub fn decls(&self) -> &KSlice<DeclId, (Decl<'a>, SourceRange)> {
        self.decls.as_kslice()
    }
}



impl Into<NodeId> for StmtId {
    fn into(self) -> NodeId {
        NodeId::Stmt(self)
    }
}


impl Into<NodeId> for ExprId {
    fn into(self) -> NodeId {
        NodeId::Expr(self)
    }
}

impl Into<NodeId> for DeclId {
    fn into(self) -> NodeId {
        NodeId::Decl(self)
    }
}
