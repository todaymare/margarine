use std::fmt::write;

use common::string_map::StringIndex;
use errors::ErrorId;
use parser::nodes::BinaryOperator;
use sti::prelude::{Arena, Vec};

use crate::Type;

#[derive(Clone)]
#[allow(dead_code)]
pub enum IR<'a> {
    Error(ErrorId),

    Unit { dst: Reg },
    Copy { dst: Reg, src: Reg },
    
    LitS { dst: Reg, lit: StrConstId },
    LitI { dst: Reg, lit: i64 },
    LitF { dst: Reg, lit: f64 },
    LitB { dst: Reg, lit: bool },

    CastAny { dst: Reg, src: Reg, target: TypeId },

    CreateStruct { dst: Reg, type_id: TypeId, fields: Vec<Reg> },
    AccField { dst: Reg, src: Reg, field_index: u16 },
    SetField { dst: Reg, val: Reg, field_indexes: Vec<u16> },
    AccEnumVariant { dst: Reg, src: Reg, variant: EnumVariant },
    SetEnumVariant { dst: Reg, src: Reg, variant: EnumVariant },

    Call { dst: Reg, function: StringIndex, args: &'a [(Reg, Reg)] },    
    ExternCall { dst: Reg, function: StringIndex, args: &'a [(Reg, Reg)] },
    
    Unwrap { src: Reg },
    OrReturn { src: Reg },

    Not { dst: Reg, src: Reg },
    NegI { dst: Reg, src: Reg },
    NegF { dst: Reg, src: Reg },


    BinaryOp { op: BinaryOperator, typ: Type, dst: Reg, lhs: Reg, rhs: Reg },
}


#[derive(Debug, Clone)]
pub enum Terminator {
    Ret,
    Jmp(BlockId),
    Jif { cond: Reg, if_true: BlockId, if_false: BlockId },
    Match { src: Reg, jumps: Vec<BlockId> },
}


#[derive(Debug, Clone, Copy)]
pub struct EnumVariant(u16);

#[derive(Debug, Clone, Copy)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct BlockId(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StrConstId(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reg(pub usize);

#[derive(Debug, Clone)]
pub struct Block<'a> {
    pub id: BlockId,
    pub body: Vec<IR<'a>, &'a Arena>,
    pub terminator: Terminator,
}


impl<'a> Block<'a> {
    pub fn push(&mut self, ir: IR<'a>) {
        self.body.push(ir)
    }
}


impl core::fmt::Display for Reg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}


impl core::fmt::Debug for IR<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IR::Error(e) => write!(f, "err {e:?}"),
            IR::Unit { dst } => write!(f, "unit {dst}"),
            IR::Copy { dst, src } => write!(f, "copy {dst} {src}"),
            IR::LitS { dst, lit } => write!(f, "lits {dst} {lit:?}"),
            IR::LitI { dst, lit } => write!(f, "liti {dst} {lit}"),
            IR::LitF { dst, lit } => write!(f, "litf {dst} {lit}"),
            IR::LitB { dst, lit } => write!(f, "litb {dst} {lit}"),
            IR::CastAny { dst, src, target } => write!(f, "cany {dst} {src} {target:?}"),
            IR::CreateStruct { dst, type_id, fields } => write!(f, "cstrct {dst} {type_id:?} {fields:?}"),
            IR::AccField { dst, src, field_index } => write!(f, "astrct {dst} {src} {field_index}"),
            IR::SetField { dst, val, field_indexes } => write!(f, "sstrct {dst} {val} {field_indexes:?}"),
            IR::AccEnumVariant { dst, src, variant } => write!(f, "aev {dst} {src} {variant:?}"),
            IR::SetEnumVariant { dst, src, variant } => write!(f, "sev {dst} {src} {variant:?}"),
            IR::Call { dst, function, args } => write!(f, "call {dst} {function:?} {args:?}"),
            IR::ExternCall { dst, function, args } => write!(f, "ecall {dst} {function:?} {args:?}"),
            IR::Unwrap { src } => write!(f, "unwrap {src}"),
            IR::OrReturn { src } => write!(f, "try {src}"),
            IR::Not { dst, src } => write!(f, "not {dst} {src}"),
            IR::NegI { dst, src } => write!(f, "negi {dst} {src}"),
            IR::NegF { dst, src } => write!(f, "negf {dst} {src}"),
            IR::BinaryOp { op, typ, dst, lhs, rhs } => write!(f, "binop {dst} {lhs} {rhs} {op:?} {typ:?}"),
        }
    }
}
