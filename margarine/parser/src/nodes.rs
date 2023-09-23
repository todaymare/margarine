use std::fmt::{Display, Write};

use common::{string_map::StringIndex, source::SourceRange};
use errors::ErrorId;
use lexer::Literal;

use crate::{DataType, Block};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Node<'a> {
    kind: NodeKind<'a>,
    pub(crate) source_range: SourceRange,
}

impl<'arena> Node<'arena> {
    pub fn new(kind: NodeKind<'arena>, source_range: SourceRange) -> Self { 
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
    pub fn kind(&self) -> &NodeKind<'arena> {
        &self.kind
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum NodeKind<'a> {
    Declaration(Declaration<'a>),
    Statement(Statement<'a>),
    Expression(Expression<'a>),
    Error(ErrorId),
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Declaration<'a> {
    Struct {
        kind: StructKind,
        name: StringIndex,
        header: SourceRange,
        fields: &'a [(StringIndex, DataType<'a>, SourceRange)],
    },

    Enum {
        name: StringIndex,
        header: SourceRange,
        mappings: &'a [EnumMapping<'a>],
    },

    Function {
        is_system: bool,
        name: StringIndex,
        header: SourceRange,
        arguments: &'a [FunctionArgument<'a>],
        return_type: DataType<'a>,
        body: Block<'a>,
    },
    
    Impl {
        data_type: DataType<'a>,
        body: Block<'a>,
    },

    Using {
        file: StringIndex,
    },

    Module {
        name: StringIndex,
        body: Block<'a>,
    },

    Extern {
        file: StringIndex,
        functions: &'a [ExternFunction<'a>],
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement<'a> {
    Variable {
        name: StringIndex,
        hint: Option<DataType<'a>>,
        is_mut: bool,
        rhs: &'a Node<'a>,
    },


    UpdateValue {
        lhs: &'a Node<'a>,
        rhs: &'a Node<'a>,
    },
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Expression<'a> {
    Unit,
    
    Literal(Literal),

    Identifier(StringIndex),

    BinaryOp {
        operator: BinaryOperator,
        lhs: &'a Node<'a>,
        rhs: &'a Node<'a>,
    },

    UnaryOp {
        operator: UnaryOperator,
        rhs: &'a Node<'a>,
    },

    If {
        condition: &'a Node<'a>,
        body: Block<'a>,
        else_block: Option<&'a Node<'a>>,
    },

    Match {
        value: &'a Node<'a>,
        mappings: &'a [MatchMapping<'a>],
    },

    Block {
        block: Block<'a>,
    },

    CreateStruct {
        data_type: DataType<'a>,
        fields: &'a [(StringIndex, SourceRange, Node<'a>)],
    },

    AccessField {
        val: &'a Node<'a>,
        field: StringIndex,
        field_meta: (u16, bool),
    },

    CallFunction {
        name: StringIndex,
        is_accessor: bool,
        args: &'a [(Node<'a>, bool)],
    },

    WithinNamespace {
        namespace: StringIndex,
        namespace_source: SourceRange,
        action: &'a Node<'a>,
    },

    WithinTypeNamespace {
        namespace: DataType<'a>,
        action: &'a Node<'a>,
    },

    Loop {
        body: Block<'a>,
    },
    
    Return(&'a Node<'a>),
    Continue,
    Break,

    CastAny {
        lhs: &'a Node<'a>,
        data_type: DataType<'a>,
    },

    Unwrap(&'a Node<'a>),

    OrReturn(&'a Node<'a>),
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructKind {
    Component,
    Resource,
    Normal,
}


#[derive(Debug, PartialEq)]
pub struct ExternFunction<'arena> {
    name: StringIndex,
    path: StringIndex,
    args: &'arena [FunctionArgument<'arena>],
    return_type: DataType<'arena>,
    source_range: SourceRange,
}

impl<'arena> ExternFunction<'arena> {
    pub(crate) fn new(name: StringIndex, path: StringIndex, args: &'arena [FunctionArgument<'arena>], return_type: DataType<'arena>, source_range: SourceRange) -> Self { 
        Self { name, args, return_type, source_range, path } 
    }


    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn path(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn args(&self) -> &[FunctionArgument<'arena>] { &self.args }
    #[inline(always)]
    pub fn return_type(&self) -> &DataType<'arena> { &self.return_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }

}


#[derive(Debug, PartialEq)]
pub struct FunctionArgument<'a> {
    name: StringIndex,
    data_type: DataType<'a>,
    is_inout: bool,
    source_range: SourceRange,
}


impl<'arena> FunctionArgument<'arena> {
    pub fn new(name: StringIndex, data_type: DataType<'arena>, is_inout: bool, source_range: SourceRange) -> Self { 
        Self { name, data_type, is_inout, source_range } 
    }


    #[inline(always)]
    pub fn data_type(&self) -> &DataType<'arena> { &self.data_type }
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn is_inout(&self) -> bool { self.is_inout }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
}


#[derive(Debug, PartialEq)]
pub struct MatchMapping<'a> {
    variant: StringIndex,
    binding: StringIndex,
    source_range: SourceRange,
    expression: Node<'a>,
}


impl<'arena> MatchMapping<'arena> {
    pub fn new(variant: StringIndex, binding: StringIndex, source_range: SourceRange, expression: Node<'arena>) -> Self { 
        Self { 
            variant, 
            binding, 
            expression,
            source_range, 
        } 
    }

    
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.variant }
    #[inline(always)]
    pub fn binding(&self) -> StringIndex { self.binding }
    #[inline(always)]
    pub fn node(&self) -> &Node<'arena> { &self.expression }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }

}


#[derive(Debug, PartialEq)]
pub struct EnumMapping<'a> {
    name: StringIndex,
    number: u16,
    data_type: DataType<'a>,
    source_range: SourceRange,
    is_implicit_unit: bool,
}

impl<'arena> EnumMapping<'arena> {
    pub fn new(name: StringIndex, number: u16, data_type: DataType<'arena>, source_range: SourceRange, is_implicit_unit: bool) -> Self { 
        if is_implicit_unit {
            assert!(data_type.kind().is(&crate::DataTypeKind::Unit));
        }

        Self { name, data_type, source_range, is_implicit_unit, number } 
    }

    
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn data_type(&self) -> &DataType<'arena> { &self.data_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn is_implicit_unit(&self) -> bool { self.is_implicit_unit }
    #[inline(always)]
    pub fn number(&self) -> u16 { self.number }
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
