use errors::ErrorId;
use parser::nodes::BinaryOperator;
use sti::prelude::{Arena, Vec};

use crate::{Type, FuncId, TypeId};

#[derive(Clone)]
#[allow(dead_code)]
pub enum IR<'a> {
    Error(ErrorId),

    Unit { dst: Reg },
    Copy { dst: Reg, src: Reg },
    CopyData { dst: Reg, src: Reg },
    
    LitS { dst: Reg, lit: StrConstId },
    LitI { dst: Reg, lit: i64 },
    LitF { dst: Reg, lit: f64 },
    LitB { dst: Reg, lit: bool },

    CastAny { dst: Reg, src: Reg, target: TypeId },

    CreateStruct { dst: Reg, type_id: TypeId, fields: &'a [Reg] },
    AccField { dst: Reg, src: Reg, field_index: u16 },
    SetField { dst: Reg, val: Reg, field_indexes: &'a [Reg] },
    AccEnumVariant { dst: Reg, src: Reg, variant: EnumVariant, typ: TypeId },
    AccUnwrapEnumVariant { dst: Reg, src: Reg, variant: EnumVariant, typ: TypeId },
    SetEnumVariant { dst: Reg, src: Reg, variant: EnumVariant },

    Call { dst: Reg, function: FuncId, args: &'a [(Reg, Reg)] },    
    ExternCall { dst: Reg, function: FuncId, args: &'a [(Reg, Reg)] },
    
    Unwrap { src: Reg, dst: Reg },
    OrReturn { src: Reg, dst: Reg },

    Not { dst: Reg, src: Reg },
    NegI { dst: Reg, src: Reg },
    NegF { dst: Reg, src: Reg },


    BinaryOp { op: BinaryOperator, typ: Type, dst: Reg, lhs: Reg, rhs: Reg },
}


#[derive(Debug, Clone, Copy)]
pub enum Terminator<'a> {
    Ret,
    Jmp(BlockId),
    Jif { cond: Reg, if_true: BlockId, if_false: BlockId },
    Match { src: Reg, jumps: &'a [BlockId] },
}


#[derive(Debug, Clone, Copy)]
pub struct EnumVariant(pub u16);

#[derive(Clone, Copy)]
pub struct BlockId(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StrConstId(pub u32);

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Reg(pub usize);

#[derive(Clone)]
pub struct Block<'a> {
    pub id: BlockId,
    pub body: Vec<IR<'a>, &'a Arena>,
    pub terminator: Terminator<'a>,
}


impl<'a> Block<'a> {
    #[inline(always)]
    pub fn push(&mut self, ir: IR<'a>) {
        self.body.push(ir)
    }
}


impl core::fmt::Display for Reg {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "#{}", self.0)
    }
}


impl core::fmt::Display for BlockId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bb{}", self.0)
    }
}


impl core::fmt::Debug for BlockId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        <Self as core::fmt::Display>::fmt(self, f)
    }
}


impl core::fmt::Debug for Block<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}:", self.id)?;


        for ir in self.body.iter() {
            writeln!(f, "  {ir:?}")?;
        }
        
        
        write!(f, "  -> ")?;
        match self.terminator {
            Terminator::Ret => write!(f, "ret"),
            Terminator::Jmp(v) => write!(f, "jmp {v}"),
            Terminator::Jif { cond, if_true, if_false } => write!(f, "jif {cond} {if_true} {if_false}"),
            Terminator::Match { src, jumps } => write!(f, "match {src} {jumps:?}"),
        }?;
        
        writeln!(f)
    }
}


impl core::fmt::Debug for IR<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IR::Error(e) => write!(f, "err {e:?}"),
            IR::Unit { dst } => write!(f, "unit {dst}"),
            IR::Copy { dst, src } => write!(f, "copy {dst} {src}"),
            IR::CopyData { dst, src } => write!(f, "copyd {dst} {src}"),
            IR::LitS { dst, lit } => write!(f, "lits {dst} {lit:?}"),
            IR::LitI { dst, lit } => write!(f, "liti {dst} {lit}"),
            IR::LitF { dst, lit } => write!(f, "litf {dst} {lit}"),
            IR::LitB { dst, lit } => write!(f, "litb {dst} {lit}"),
            IR::CastAny { dst, src, target } => write!(f, "cany {dst} {src} {target:?}"),
            IR::CreateStruct { dst, type_id, fields } => write!(f, "cstrct {dst} {type_id:?} {fields:?}"),
            IR::AccField { dst, src, field_index } => write!(f, "astrct {dst} {src} {field_index}"),
            IR::SetField { dst, val, field_indexes } => write!(f, "sstrct {dst} {val} {field_indexes:?}"),
            IR::AccEnumVariant { dst, src, variant, typ } => write!(f, "aev {dst} {src} {variant:?} {typ:?}"),
            IR::AccUnwrapEnumVariant { dst, src, variant, typ } => write!(f, "auev {dst} {src} {variant:?} {typ:?}"),
            IR::SetEnumVariant { dst, src, variant } => write!(f, "sev {dst} {src} {variant:?}"),
            IR::Call { dst, function, args } => write!(f, "call {dst} {function:?} {args:?}"),
            IR::ExternCall { dst, function, args } => write!(f, "ecall {dst} {function:?} {args:?}"),
            IR::Unwrap { dst, src } => write!(f, "unwrap {dst} {src}"),
            IR::OrReturn { dst, src } => write!(f, "try {dst} {src}"),
            IR::Not { dst, src } => write!(f, "not {dst} {src}"),
            IR::NegI { dst, src } => write!(f, "negi {dst} {src}"),
            IR::NegF { dst, src } => write!(f, "negf {dst} {src}"),
            IR::BinaryOp { op, typ, dst, lhs, rhs } => write!(f, "binop {dst} {lhs} {rhs} {op:?} {typ:?}"),
        }
    }
}
