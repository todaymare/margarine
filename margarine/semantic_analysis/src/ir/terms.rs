use common::string_map::StringIndex;
use errors::ErrorId;
use sti::prelude::{Arena, Vec};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IR<'a> {
    DebugName { dst: Reg, name: StringIndex },
    Error(ErrorId),

    Unit { dst: Reg },
    Copy { dst: Reg, src: Reg },
    Literal { dst: Reg, lit: LiteralId },

    CastAny { dst: Reg, src: Reg, target: TypeId },

    CreateStruct { dst: Reg, type_id: TypeId, fields: Vec<Reg> },
    AccField { dst: Reg, src: Reg, field_index: u16 },
    SetField { dst: Reg, val: Reg, field_index: Vec<u16> },
    AccEnumVariant { dst: Reg, src: Reg, variant: EnumVariant },
    SetEnumVariant { dst: Reg, src: Reg, variant: EnumVariant },

    Call { dst: Reg, function: StringIndex, args: &'a [(Reg, Reg)] },    
    ExternCall { dst: Reg, function: StringIndex, args: &'a [(Reg, Reg)] },
    
    Unwrap { src: Reg },
    OrReturn { src: Reg },

    Not { dst: Reg, src: Reg },
    NegI { dst: Reg, src: Reg },
    NegF { dst: Reg, src: Reg },


    AddI { dst: Reg, lhs: Reg, rhs: Reg },
    AddF { dst: Reg, lhs: Reg, rhs: Reg },
    AddU { dst: Reg, lhs: Reg, rhs: Reg },

    SubI { dst: Reg, lhs: Reg, rhs: Reg },
    SubF { dst: Reg, lhs: Reg, rhs: Reg },
    SubU { dst: Reg, lhs: Reg, rhs: Reg },

    MulI { dst: Reg, lhs: Reg, rhs: Reg },
    MulF { dst: Reg, lhs: Reg, rhs: Reg },
    MulU { dst: Reg, lhs: Reg, rhs: Reg },

    DivI { dst: Reg, lhs: Reg, rhs: Reg },
    DivF { dst: Reg, lhs: Reg, rhs: Reg },
    DivU { dst: Reg, lhs: Reg, rhs: Reg },

    RemI { dst: Reg, lhs: Reg, rhs: Reg },
    RemF { dst: Reg, lhs: Reg, rhs: Reg },
    RemU { dst: Reg, lhs: Reg, rhs: Reg },

    LeftShiftI { dst: Reg, lhs: Reg, rhs: Reg },
    LeftShiftU { dst: Reg, lhs: Reg, rhs: Reg },

    RightShiftI { dst: Reg, lhs: Reg, rhs: Reg },
    RightShiftU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseAndI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseAndU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseOrI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseOrU { dst: Reg, lhs: Reg, rhs: Reg },

    BitwiseXorI { dst: Reg, lhs: Reg, rhs: Reg },
    BitwiseXorU { dst: Reg, lhs: Reg, rhs: Reg },

    EqI { dst: Reg, lhs: Reg, rhs: Reg },
    EqF { dst: Reg, lhs: Reg, rhs: Reg },
    EqU { dst: Reg, lhs: Reg, rhs: Reg },
    EqB { dst: Reg, lhs: Reg, rhs: Reg },

    NeI { dst: Reg, lhs: Reg, rhs: Reg },
    NeF { dst: Reg, lhs: Reg, rhs: Reg },
    NeU { dst: Reg, lhs: Reg, rhs: Reg },
    NeB { dst: Reg, lhs: Reg, rhs: Reg },

    GtI { dst: Reg, lhs: Reg, rhs: Reg },
    GtF { dst: Reg, lhs: Reg, rhs: Reg },
    GtU { dst: Reg, lhs: Reg, rhs: Reg },

    GeI { dst: Reg, lhs: Reg, rhs: Reg },
    GeF { dst: Reg, lhs: Reg, rhs: Reg },
    GeU { dst: Reg, lhs: Reg, rhs: Reg },

    LtI { dst: Reg, lhs: Reg, rhs: Reg },
    LtF { dst: Reg, lhs: Reg, rhs: Reg },
    LtU { dst: Reg, lhs: Reg, rhs: Reg },

    LeI { dst: Reg, lhs: Reg, rhs: Reg },
    LeF { dst: Reg, lhs: Reg, rhs: Reg },
    LeU { dst: Reg, lhs: Reg, rhs: Reg },
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
pub struct Reg(pub usize);

#[derive(Debug, Clone, Copy)]
pub struct LiteralId(pub u32);

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
