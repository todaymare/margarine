use common::{source::SourceRange, string_map::StringIndex};
use sti::define_key;

use crate::{DataType, Block};

use super::expr::ExprId;

define_key!(pub StmtId(u32));


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stmt<'a> {
    Variable {
        name: StringIndex,
        hint: Option<DataType<'a>>,
        rhs: ExprId,
    },


    VariableTuple {
        names: &'a [StringIndex],
        hint: Option<DataType<'a>>,
        rhs: ExprId,
    },


    UpdateValue {
        lhs: ExprId,
        rhs: ExprId,
    },


    ForLoop {
        binding: (StringIndex, SourceRange),
        expr: ExprId,
        body: Block<'a>,
    }
}
