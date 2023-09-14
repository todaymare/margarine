use common::string_map::StringIndex;
use lexer::Literal;
use parser::nodes::{BinaryOperator, UnaryOperator};

use crate::TypeId;

pub type TypedBlock<'a> = &'a [TypedNode<'a>];


#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    UserType(TypeId),
    Str,
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
    #[inline(always)]
    pub fn new(kind: TypedNodeKind<'a>) -> Self { Self { kind } }


    #[inline(always)]
    pub fn expr(kind: TypedExpressionKind<'a>, typ: Type) -> Self {
        Self::new(TypedNodeKind::Expression(TypedExpression { kind, typ }))
    }


    #[inline(always)]
    pub fn stmt(kind: TypedStatement<'a>) -> Self {
        Self::new(TypedNodeKind::Statement(kind))
    }


    #[inline(always)]
    pub fn typ(&self) -> Type {
        match &self.kind {
            TypedNodeKind::Statement(_) => Type::Unit,
            TypedNodeKind::Expression(e) => e.typ,
        }
    }
}


#[derive(Debug)]
pub enum TypedNodeKind<'a> {
    Statement(TypedStatement<'a>),
    Expression(TypedExpression<'a>),
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
pub struct TypedExpression<'a> {
    kind: TypedExpressionKind<'a>,
    typ: Type,
}

impl<'a> TypedExpression<'a> {
    pub fn new(kind: TypedExpressionKind<'a>, typ: Type) -> Self { Self { kind, typ } }
}


#[derive(Debug)]
pub enum TypedExpressionKind<'a> {
    Unit,

    Literal(Literal),

    AccVariable(StringIndex),

    BinaryOp {
        operator: BinaryOperator,
        lhs: &'a TypedNode<'a>,
        rhs: &'a TypedNode<'a>,
    },

    UnaryOp {
        operator: UnaryOperator,
    }
}
