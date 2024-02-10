use std::fmt::Display;

use common::{source::SourceRange, string_map::StringIndex};
use lexer::Literal;

use crate::{DataType, Block};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ExpressionNode<'a> {
    kind: Expression<'a>,
    pub(crate) source_range: SourceRange,
}


impl<'arena> ExpressionNode<'arena> {
    pub fn new(kind: Expression<'arena>, source_range: SourceRange) -> Self { 
        Self {
            kind, 
            source_range,
        } 
    }


    #[inline(always)]
    pub fn range(&self) -> SourceRange {
        self.source_range
    }


    #[inline(always)]
    pub fn kind(&self) -> Expression<'arena> {
        self.kind
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'a> {
    Unit,
    
    Literal(Literal),

    Identifier(StringIndex),

    BinaryOp {
        operator: BinaryOperator,
        lhs: &'a ExpressionNode<'a>,
        rhs: &'a ExpressionNode<'a>,
    },

    UnaryOp {
        operator: UnaryOperator,
        rhs: &'a ExpressionNode<'a>,
    },

    If {
        condition: &'a ExpressionNode<'a>,
        body: &'a ExpressionNode<'a>,
        else_block: Option<&'a ExpressionNode<'a>>,
    },

    Match {
        value: &'a ExpressionNode<'a>,
        taken_as_inout: bool,
        mappings: &'a [MatchMapping<'a>],
    },

    Block {
        block: Block<'a>,
    },

    CreateStruct {
        data_type: DataType<'a>,
        fields: &'a [(StringIndex, SourceRange, ExpressionNode<'a>)],
    },

    AccessField {
        val: &'a ExpressionNode<'a>,
        field_name: StringIndex,
    },

    CallFunction {
        name: StringIndex,
        is_accessor: bool,
        args: &'a [(ExpressionNode<'a>, bool)],
    },

    WithinNamespace {
        namespace: StringIndex,
        namespace_source: SourceRange,
        action: &'a ExpressionNode<'a>,
    },

    WithinTypeNamespace {
        namespace: DataType<'a>,
        action: &'a ExpressionNode<'a>,
    },

    Loop {
        body: Block<'a>,
    },
    
    Return(&'a ExpressionNode<'a>),
    Continue,
    Break,

    Tuple(&'a [ExpressionNode<'a>]),

    CastAny {
        lhs: &'a ExpressionNode<'a>,
        data_type: DataType<'a>,
    },

    AsCast {
        lhs: &'a ExpressionNode<'a>,
        data_type: DataType<'a>,
    },

    Unwrap(&'a ExpressionNode<'a>),

    OrReturn(&'a ExpressionNode<'a>),
}


#[derive(Debug, PartialEq)]
pub struct MatchMapping<'a> {
    variant: StringIndex,
    binding: StringIndex,
    binding_range: SourceRange,
    source_range: SourceRange,
    expression: ExpressionNode<'a>,
    is_inout: bool,
}


impl<'arena> MatchMapping<'arena> {
    pub fn new(
        variant: StringIndex, 
        binding: StringIndex, 
        binding_range: SourceRange,
        source_range: SourceRange, 
        expression: ExpressionNode<'arena>,
        is_inout: bool,
    ) -> Self { 
        Self { 
            variant, 
            binding, 
            expression,
            source_range, 
            is_inout,
            binding_range,
        } 
    }

    
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.variant }
    #[inline(always)]
    pub fn binding(&self) -> StringIndex { self.binding }
    #[inline(always)]
    pub fn node(&self) -> ExpressionNode<'arena> { self.expression }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn binding_range(&self) -> SourceRange { self.binding_range }
    #[inline(always)]
    pub fn is_inout(&self) -> bool { self.is_inout }

}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinaryOperator {
    /// '+'
    Add,
    /// '-'
    Sub,
    /// '*'
    Mul,
    /// '/'
    Div,
    /// '%'
    Rem,

    /// '<<'
    BitshiftLeft,
    /// '>>'
    BitshiftRight,
    /// '&'
    BitwiseAnd,
    /// '|'
    BitwiseOr,
    /// '^'
    BitwiseXor,

    /// '=='
    Eq,
    /// '!='
    Ne,
    /// '>'
    Gt,
    /// '>='
    Ge,
    /// '<'
    Lt,
    /// '<='
    Le,
}

impl BinaryOperator {
    pub fn is_arith(self) -> bool {
        match self {
            | BinaryOperator::BitshiftLeft
            | BinaryOperator::BitshiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::Eq
            | BinaryOperator::Ne
            | BinaryOperator::Gt
            | BinaryOperator::Ge
            | BinaryOperator::Lt
            | BinaryOperator::Le 
             => false,
            
            | BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Rem
             => true,
        }

    }

    
    pub fn is_bw(self) -> bool {
        match self {
            | BinaryOperator::Eq
            | BinaryOperator::Ne
            | BinaryOperator::Gt
            | BinaryOperator::Ge
            | BinaryOperator::Lt
            | BinaryOperator::Le
            | BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Rem
             => false,

            | BinaryOperator::BitshiftLeft
            | BinaryOperator::BitshiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
             => true,
        }

    }

    
    pub fn is_ocomp(self) -> bool {
        match self {
            | BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Rem
            | BinaryOperator::BitshiftLeft
            | BinaryOperator::BitshiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::Eq
            | BinaryOperator::Ne
             => false,

            | BinaryOperator::Gt
            | BinaryOperator::Ge
            | BinaryOperator::Lt
            | BinaryOperator::Le
             => true,
        }
    }

    
    pub fn is_ecomp(self) -> bool {
        match self {
            | BinaryOperator::Add
            | BinaryOperator::Sub
            | BinaryOperator::Mul
            | BinaryOperator::Div
            | BinaryOperator::Rem
            | BinaryOperator::BitshiftLeft
            | BinaryOperator::BitshiftRight
            | BinaryOperator::BitwiseAnd
            | BinaryOperator::BitwiseOr
            | BinaryOperator::BitwiseXor
            | BinaryOperator::Gt
            | BinaryOperator::Ge
            | BinaryOperator::Lt
            | BinaryOperator::Le
             => false,

            | BinaryOperator::Eq
            | BinaryOperator::Ne
             => true,
        }
    }
}


impl Display for BinaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            BinaryOperator::Add => "+",
            BinaryOperator::Sub => "-",
            BinaryOperator::Mul => "*",
            BinaryOperator::Div => "/",
            BinaryOperator::Rem => "%",
            BinaryOperator::BitshiftLeft => ">>",
            BinaryOperator::BitshiftRight => "<<",
            BinaryOperator::BitwiseAnd => "&",
            BinaryOperator::BitwiseOr => "|",
            BinaryOperator::BitwiseXor => "^",
            BinaryOperator::Eq => "==",
            BinaryOperator::Ne => "!=",
            BinaryOperator::Gt => ">",
            BinaryOperator::Ge => ">=",
            BinaryOperator::Lt => "<",
            BinaryOperator::Le => "<=",
        })
    }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnaryOperator {
    Not,
    Neg,
}


impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            UnaryOperator::Not => "!",
            UnaryOperator::Neg => "-",
        })
    }
}


