use common::{source::SourceRange, string_map::StringIndex};
use sti::define_key;

use crate::{nodes::Pattern, Block, DataType};

use super::expr::ExprId;

define_key!(pub StmtId(u32));


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Stmt<'a> {
    Variable {
        pat : Pattern<'a>,
        hint: Option<DataType<'a>>,
        rhs: ExprId,
    },


    UpdateValue {
        lhs: ExprId,
        rhs: ExprId,
    },


    ForLoop {
        binding: Pattern<'a>,
        expr: ExprId,
        body: Block<'a>,
    }
}
