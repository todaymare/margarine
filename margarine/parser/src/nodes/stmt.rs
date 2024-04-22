use common::{source::SourceRange, string_map::StringIndex};
use sti::define_key;

use crate::{DataType, Block};

use super::expr::ExprId;

define_key!(u32, pub StmtId);


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stmt<'a> {
    Variable {
        name: StringIndex,
        hint: Option<DataType<'a>>,
        is_mut: bool,
        rhs: ExprId,
    },


    VariableTuple {
        names: &'a [(StringIndex, bool)],
        hint: Option<DataType<'a>>,
        rhs: ExprId,
    },


    UpdateValue {
        lhs: ExprId,
        rhs: ExprId,
    },


    ForLoop {
        binding: (bool, StringIndex, SourceRange),
        expr: (bool, ExprId),
        body: Block<'a>,
    }
}
