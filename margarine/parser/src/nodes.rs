pub mod expr;
pub mod stmt;
pub mod decl;
pub mod attr;
pub mod err;


use common::{source::SourceRange, string_map::StringIndex};

use crate::DataType;

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


#[derive(PartialEq, Clone, Copy, Debug)]
pub struct Pattern<'a> {
    source: SourceRange,
    is_inout: bool,
    kind: PatternKind<'a>,
}


impl<'a> Pattern<'a> {
    pub fn new(source: SourceRange, is_inout: bool, kind: PatternKind<'a>) -> Self { Self { source, kind, is_inout } }

    #[inline(always)]
    pub fn is_inout(&self) -> bool { self.is_inout }
}


#[derive(PartialEq, Clone, Copy, Debug)]
pub enum PatternKind<'a> {
    Ident(StringIndex),
    Tuple(&'a [Pattern<'a>]),
    Struct(DataType<'a>, &'a [(StringIndex, Pattern<'a>)]),
}
