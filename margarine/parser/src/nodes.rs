use common::{string_map::StringIndex, source::SourceRange};
use lexer::Literal;
use thin_vec::ThinVec;

use crate::{DataType, DataTypeKind, Block};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    kind: NodeKind,
    pub(crate) source_range: SourceRange,
    tags: ThinVec<StringIndex>,
    pub data_kind: DataTypeKind,
}

impl Node {
    pub fn new(kind: NodeKind, source_range: SourceRange) -> Self { 
        Self { 
            kind, 
            source_range,
            tags: ThinVec::new(),
            data_kind: DataTypeKind::Unknown, 
        } 
    }


    #[inline(always)]
    pub fn add_tag(&mut self, tag: StringIndex) {
        self.tags.push(tag)
    }


    #[inline(always)]
    pub fn range(&self) -> SourceRange {
        self.source_range
    }


    #[inline(always)]
    pub fn kind(&self) -> &NodeKind {
        &self.kind
    }


    #[inline(always)]
    pub fn kind_mut(&mut self) -> &mut NodeKind {
        &mut self.kind
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Declaration(Declaration),
    Statement(Statement),
    Expression(Expression),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Struct {
        kind: StructKind,
        name: StringIndex,
        fields: Vec<(StringIndex, DataType, SourceRange)>,
    },

    Enum {
        name: StringIndex,
        mappings: Vec<EnumMapping>,
    },

    Function {
        is_system: bool,
        is_anonymous: bool,
        name: StringIndex,
        arguments: Vec<FunctionArgument>,
        return_type: DataType,
        body: Block,
    },
    
    Impl {
        data_type: DataType,
        body: Block,
    },

    Using {
        file: StringIndex,
    },

    Module {
        name: StringIndex,
        body: Block,
    },

    Extern {
        file: StringIndex,
        functions: Vec<ExternFunction>,
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable {
        name: StringIndex,
        hint: Option<DataType>,
        is_mut: bool,
        rhs: Box<Node>,
    },


    UpdateValue {
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unit,
    
    Literal(Literal),

    Identifier(StringIndex),

    BinaryOp {
        operator: BinaryOperator,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },

    UnaryOp {
        operator: UnaryOperator,
        rhs: Box<Node>,
    },

    If {
        condition: Box<Node>,
        body: Block,
        else_block: Option<Box<Node>>,
    },

    Match {
        value: Box<Node>,
        mappings: Vec<MatchMapping>,
    },

    Block {
        block: Block,
    },

    CreateStruct {
        data_type: DataType,
        fields: Vec<(StringIndex, SourceRange, Node)>,
    },

    AccessField {
        val: Box<Node>,
        field: StringIndex,
        field_meta: (u16, bool),
    },

    CallFunction {
        name: StringIndex,
        is_accessor: Option<Box<Node>>,
        args: Vec<(Node, bool)>,
    },

    WithinNamespace {
        namespace: StringIndex,
        namespace_source: SourceRange,
        action: Box<Node>,
    },

    WithinTypeNamespace {
        namespace: DataType,
        action: Box<Node>,
    },

    Loop {
        body: Block,
    },
    
    Return(Box<Node>),
    Continue,
    Break,

    CastAny {
        lhs: Box<Node>,
        data_type: DataType,
    },

    Unwrap(Box<Node>),

    OrReturn(Box<Node>),
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StructKind {
    Component,
    Resource,
    Normal,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    name: StringIndex,
    path: StringIndex,
    args: Vec<FunctionArgument>,
    return_type: DataType,
    source_range: SourceRange,
}

impl ExternFunction {
    pub(crate) fn new(name: StringIndex, path: StringIndex, args: Vec<FunctionArgument>, return_type: DataType, source_range: SourceRange) -> Self { 
        Self { name, args, return_type, source_range, path } 
    }


    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn path(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn args(&self) -> &[FunctionArgument] { &self.args }
    #[inline(always)]
    pub fn args_mut(&mut self) -> &mut [FunctionArgument] { &mut self.args }
    #[inline(always)]
    pub fn return_type(&self) -> &DataType { &self.return_type }
    #[inline(always)]
    pub fn return_type_mut(&mut self) -> &mut DataType { &mut self.return_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }

}


#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    name: StringIndex,
    data_type: DataType,
    is_inout: bool,
    source_range: SourceRange,
}


impl FunctionArgument {
    pub fn new(name: StringIndex, data_type: DataType, is_inout: bool, source_range: SourceRange) -> Self { 
        Self { name, data_type, is_inout, source_range } 
    }


    #[inline(always)]
    pub fn data_type(&self) -> &DataType { &self.data_type }
    #[inline(always)]
    pub fn data_type_mut(&mut self) -> &mut DataType { &mut self.data_type }
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn is_inout(&self) -> bool { self.is_inout }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MatchMapping {
    variant: StringIndex,
    binding: StringIndex,
    source_range: SourceRange,
    expression: Node,
}


impl MatchMapping {
    pub fn new(variant: StringIndex, binding: StringIndex, source_range: SourceRange, expression: Node) -> Self { 
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
    pub fn node(&self) -> &Node { &self.expression }
    #[inline(always)]
    pub fn node_mut(&mut self) -> &mut Node { &mut self.expression }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }

}


#[derive(Debug, PartialEq, Clone)]
pub struct EnumMapping {
    name: StringIndex,
    number: u16,
    data_type: DataType,
    source_range: SourceRange,
    is_implicit_unit: bool,
}

impl EnumMapping {
    pub fn new(name: StringIndex, number: u16, data_type: DataType, source_range: SourceRange, is_implicit_unit: bool) -> Self { 
        if is_implicit_unit {
            assert!(data_type.kind().is(&crate::DataTypeKind::Unit));
        }

        Self { name, data_type, source_range, is_implicit_unit, number } 
    }

    
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn data_type(&self) -> &DataType { &self.data_type }
    #[inline(always)]
    pub fn data_type_mut(&mut self) -> &mut DataType { &mut self.data_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn is_implicit_unit(&self) -> bool { self.is_implicit_unit }
    #[inline(always)]
    pub fn number(&self) -> u16 { self.number }
}


#[derive(Debug, Clone, PartialEq)]
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


#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOperator {
    Not,
    Neg,
}

