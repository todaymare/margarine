use common::{SourceRange, SymbolIndex};
use lexer::Literal;
use thin_vec::ThinVec;

use crate::{DataType, Block};

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    kind: NodeKind,
    pub(crate) source_range: SourceRange,
    tags: ThinVec<SymbolIndex>,
}

impl Node {
    pub fn new(kind: NodeKind, source_range: SourceRange) -> Self { 
        Self { 
            kind, 
            source_range,
            tags: ThinVec::new(), 
        } 
    }


    #[inline(always)]
    pub fn add_tag(&mut self, tag: SymbolIndex) {
        self.tags.push(tag)
    }


    #[inline(always)]
    pub fn range(&self) -> SourceRange {
        self.source_range
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
        name: SymbolIndex,
        fields: Vec<(SymbolIndex, DataType)>,
    },

    Enum {
        name: SymbolIndex,
        mappings: Vec<(SymbolIndex, DataType)>,
    },

    Function {
        is_system: bool,
        name: SymbolIndex,
        arguments: Vec<FunctionArgument>,
        return_type: DataType,
        body: Block,
    },
    
    Impl {
        data_type: DataType,
        body: Block,
    },

    Using {
        file: SymbolIndex,
    },

    Module {
        name: SymbolIndex,
        body: Block,
    },

    Extern {
        file: SymbolIndex,
        functions: Vec<ExternFunction>,
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    Variable {
        name: SymbolIndex,
        hint: Option<DataType>,
        is_mut: bool,
        rhs: Box<Node>,
    },


    UpdateValue {
        lhs: Box<Node>,
        rhs: Box<Node>,
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Unit,
    
    Literal(Literal),

    Identifier(SymbolIndex),

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
        fields: Vec<(SymbolIndex, Node)>,
    },

    AccessField {
        val: Box<Node>,
        field: SymbolIndex,
    },

    CallFunction {
        name: SymbolIndex,
        is_accessor: Option<Box<Node>>,
        args: Vec<Node>,
    },

    WithinNamespace {
        namespace: SymbolIndex,
        action: Box<Node>,
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum StructKind {
    Component,
    Resource,
    Normal,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunction {
    name: SymbolIndex,
    fields: Vec<DataType>,
    return_type: DataType,
}

impl ExternFunction {
    pub(crate) fn new(name: SymbolIndex, fields: Vec<DataType>, return_type: DataType) -> Self { 
        Self { name, fields, return_type } 
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct FunctionArgument {
    name: SymbolIndex,
    data_type: DataType,
    is_mut: bool,
}


impl FunctionArgument {
    pub(crate) fn new(name: SymbolIndex, data_type: DataType, is_mut: bool) -> Self { 
        Self { name, data_type, is_mut } 
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct MatchMapping {
    variant: SymbolIndex,
    variable: SymbolIndex,
    expression: Node,
}


impl MatchMapping {
    pub fn new(variant: SymbolIndex, variable: SymbolIndex, expression: Node) -> Self { 
        Self { 
            variant, 
            variable, 
            expression 
        } 
    }
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

