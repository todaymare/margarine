macro_rules! bytecode {
    ($($ident: ident $(($desc: literal))?),* $(,)?) => {
        #[repr(u8)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum Bytecode {
            $(
                $(#[doc = $desc])?
                $ident
            ),*
        }


        #[allow(non_upper_case_globals)]
        pub mod bytecode_consts {
            $(
                pub const $ident : u8 = super::Bytecode::$ident as u8;
            )*
        }


        impl Bytecode {
            #[inline(always)]
            pub fn from_u8(u8: u8) -> Option<Bytecode> {
                match u8 {
                    $(bytecode_consts::$ident => Some(Bytecode::$ident),)*
                    _ => None,
                }
            }


            #[inline(always)]
            pub fn as_u8(self) -> u8 { self as _ }
        }
    }
}


bytecode! {
    Func("func [name: str] [is_system: u8] [func_size: u32] [ret: TypeId(u32)]\
        [len(args): u16] [args: []Arg(name: str, is_inout: bool, type: TypeId(u32))]"),

    Struct("struct [name: str] [type: type_id] [kind: u8] \
        [len(fields): u16] [fields: []Field[name: str, type: TypeId(u32)]]"),
    
    
    Ret("ret []"),
    Copy("copy [dst: Reg(u8)] [src: Reg(u8)] [len(inouts) = u8] [inouts: []u8]"),
    Lit("lit [dst: Reg(u8)] [val: u32]"),

    Push("push [amount: u8]"),
    Pop("pop [amount: u8]"),

    Jmp("jmp [dst: u32]"),
    Jif("jif [cond: Reg(u8)] [true: u32] [false: u32]"),

    Match("match [index: Reg(u8)] [jmps: []u32]"),

    CastAny("castany [dst: Reg(u8)] [src: Reg(u8)] [target type: TypeId(u32)]"),

    CreateStruct("struct [dst: Reg(u8)] [type_id: TypeId(u32)] [len(list): u8] [list: []Reg(u8)]"),
    AccField("accfield [dst: Reg(u8)] [src: Reg(u8)] [index: u8]"),
    SetField("setfield [dst: Reg(u8)] [src: Reg(u8)] [index: u8]"),

    Call("call [dst: Reg(u8)] [func: u32] [len(args): u8] [args: []Reg(u8)]"),

    Not("not [dst: Reg(u8)] [src: Reg(u8)]"),
    NegI("negi [dst: Reg(u8)] [src: Reg(u8)]"),
    NegF("negf [dst: Reg(u8)] [src: Reg(u8)]"),

    AddI("addi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    AddF("addf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    AddU("addu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    SubI("subi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    SubF("subf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    SubU("subu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    MulI("muli [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    MulF("mulf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    MulU("mulu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    DivI("divi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    DivF("divf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    DivU("divu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    RemI("remi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    RemF("remf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    RemU("remu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    LeftShiftI("lsi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    LeftShiftU("lsu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    RightShiftI("rsi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    RightShiftU("rsu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    BitwiseAndI("andi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    BitwiseAndU("andu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    BitwiseOrI("ori [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    BitwiseOrU("oru [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    BitwiseXorI("xori [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    BitwiseXorU("xoru [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    EqI("eqi [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    EqF("eqf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    EqU("equ [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    EqB("eqb [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    NeI("nei [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    NeF("nef [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    NeU("neu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    NeB("neb [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),


    GtI("gti [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    GtF("gtf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    GtU("gtu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    GeI("gei [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    GeF("gef [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    GeU("geu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    LtI("lti [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    LtF("ltf [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    LtU("ltu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),

    LeI("lei [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    LeF("lef [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
    LeU("leu [dst: Reg(u8)] [lhs: Reg(u8)] [rhs: Reg(u8)]"),
}

