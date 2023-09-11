use common::string_map::StringIndex;
use lexer::Literal;

use crate::TypeId;

pub type TypedBlock<'a> = &'a [TypedNode<'a>];


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    UserType(TypeId),
    Int,
    Bool,
    Float,
    Unit,
    Any,
    Never,

    Error,
}


#[derive(Debug)]
pub struct TypedNode<'a> {
    pub kind: TypedNodeKind<'a>,
}

impl<'a> TypedNode<'a> {
    pub fn new(kind: TypedNodeKind<'a>) -> Self { Self { kind } }
}


#[derive(Debug)]
pub enum TypedNodeKind<'a> {
    Statement(TypedStatement<'a>),
    Expression(TypedExpression),
}


#[derive(Debug)]
pub enum TypedStatement<'a> {
    VariableDeclaration {
        name: StringIndex,
        rhs: &'a TypedNode<'a>,
    },


    UpdateValue {
        lhs: &'a TypedNode<'a>,
        rhs: &'a TypedNode<'a>,
    }
}


#[derive(Debug)]
pub enum TypedExpression {
    Unit,
    Literal(Literal),
}
