pub mod expr;
pub mod stmt;
pub mod decl;
pub mod attr;
pub mod err;


use common::source::SourceRange;

use self::{decl::DeclarationNode, stmt::StatementNode, expr::ExpressionNode, attr::AttributeNode, err::ErrorNode};


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Node<'a> {
    Declaration(DeclarationNode<'a>),
    Statement(StatementNode<'a>),
    Expression(ExpressionNode<'a>),
    Attribute(&'a AttributeNode<'a>),
    Error(ErrorNode),
}


impl Node<'_> {
    pub fn range(self) -> SourceRange {
        match self {
            Node::Declaration(v) => v.range(),
            Node::Statement(v) => v.range(),
            Node::Expression(v) => v.range(),
            Node::Attribute(v) => v.range(),
            Node::Error(v) => v.range(),
        }
    }
}


impl<'a> From<DeclarationNode<'a>> for Node<'a> {
    fn from(value: DeclarationNode<'a>) -> Self {
        Self::Declaration(value)
    }
}

impl<'a> From<StatementNode<'a>> for Node<'a> {
    fn from(value: StatementNode<'a>) -> Self {
        Self::Statement(value)
    }
}

impl<'a> From<ExpressionNode<'a>> for Node<'a> {
    fn from(value: ExpressionNode<'a>) -> Self {
        Self::Expression(value)
    }
}

impl<'a> From<&'a AttributeNode<'a>> for Node<'a> {
    fn from(value: &'a AttributeNode<'a>) -> Self {
        Self::Attribute(value)
    }
}

impl<'a> From<ErrorNode> for Node<'a> {
    fn from(value: ErrorNode) -> Self {
        Self::Error(value)
    }
}
