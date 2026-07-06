use sti::define_key;

define_key!(pub SymbolId(pub u32));

impl SymbolId {
    pub const UNIT   : Self = Self(0);
    pub const I64    : Self = Self(1);
    pub const F64    : Self = Self(2);
    pub const BOOL   : Self = Self(3);
    pub const ERR    : Self = Self(6);
    pub const NEVER  : Self = Self(7);
    pub const PTR    : Self = Self(8);
    pub const RANGE  : Self = Self(9);
    pub const OPTION : Self = Self(10);
    pub const RESULT : Self = Self(13);
    pub const STR    : Self = Self(16);
    pub const ANY    : Self = Self(17);
    pub const LIST   : Self = Self(18);
    pub const BUILTIN_TYPE_ID: Self = Self(19);
    pub const BUILTIN_ANY    : Self = Self(20);
    pub const BUILTIN_DOWNCAST_ANY : Self = Self(21);
    pub const BUILTIN_SIZE_OF : Self = Self(22);
    pub const EQ_TRAIT : Self = Self(23);


    pub fn supports_arith(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }


    pub fn supports_bw(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }


    pub fn supports_ord(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }

    pub fn supports_eq(self) -> bool {
        self.is_float() || self.is_num()
    }


    pub fn is_num(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::ERR
        )
    }


    pub fn is_int(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }

    pub fn is_sint(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::ERR
        )
    }


    pub fn is_float(self) -> bool {
        matches!(self,
            | Self::F64
            | Self::ERR
        )
    }
}
