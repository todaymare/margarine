use common::{source::SourceRange, string_map::StringIndex};

use crate::DataType;

use super::expr::ExpressionNode;

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
        rhs: &'a ExpressionNode<'a>,
    },


    VariableTuple {
        names: &'a [(StringIndex, bool)],
        hint: Option<DataType<'a>>,
        rhs: &'a ExpressionNode<'a>,
    },


    UpdateValue {
        lhs: &'a ExpressionNode<'a>,
        rhs: &'a ExpressionNode<'a>,
    },
}
