use common::{source::SourceRange, string_map::StringIndex};

use crate::{DataType, Block};

use super::{expr::ExpressionNode, Pattern};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StatementNode<'a> {
    kind: Statement<'a>,
    pub(crate) source_range: SourceRange,
}


impl<'arena> StatementNode<'arena> {
    pub fn new(kind: Statement<'arena>, source_range: SourceRange) -> Self { 
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
    pub fn kind(&self) -> Statement<'arena> {
        self.kind
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Statement<'a> {
    Variable {
        name: StringIndex,
        hint: Option<DataType<'a>>,
        is_mut: bool,
        rhs: ExpressionNode<'a>,
    },


    VariableTuple {
        names: &'a [(StringIndex, bool)],
        hint: Option<DataType<'a>>,
        rhs: ExpressionNode<'a>,
    },


    UpdateValue {
        lhs: ExpressionNode<'a>,
        rhs: ExpressionNode<'a>,
    },


    ForLoop {
        binding: (bool, Pattern<'a>),
        expr: (bool, ExpressionNode<'a>),
        body: Block<'a>,
    }
}
