use sti::define_key;

define_key!(pub SymbolId(pub u32));

impl SymbolId {
    pub const UNIT   : Self = Self(0);
    pub const I64    : Self = Self(1);
    pub const F64    : Self = Self(2);
    pub const BOOL   : Self = Self(3);
    pub const NEVER  : Self = Self(6);
    pub const PTR    : Self = Self(7);
    pub const RANGE  : Self = Self(8);
    pub const OPTION : Self = Self(9);
    pub const RESULT : Self = Self(12);
    pub const STR    : Self = Self(15);
    pub const LIST   : Self = Self(16);
    pub const BUILTIN_TYPE_ID: Self = Self(17);
    pub const BUILTIN_SIZE_OF : Self = Self(18);
    pub const EQ_TRAIT : Self = Self(19);
    pub const RC : Self = Self(20);
    pub const BUILTIN_RC : Self = Self(21);
    pub const RC_GET : Self = Self(22);
    pub const RC_SET : Self = Self(23);
    pub const PTR_ALLOC  : Self = Self(24);
    pub const PTR_FREE   : Self = Self(25);
    pub const PTR_READ   : Self = Self(26);
    pub const PTR_WRITE  : Self = Self(27);
    pub const PTR_WRITE_UNINIT : Self = Self(28);
    pub const PTR_NULL   : Self = Self(29);
    pub const PTR_OFFSET : Self = Self(30);
    pub const PTR_CAST   : Self = Self(31);
    pub const DESTROY_TRAIT : Self = Self(32);
    pub const PTR_DROP       : Self = Self(33);
    pub const LIST_CONCAT    : Self = Self(34);
    pub const LIST_SLICE_PAIR: Self = Self(35);
    pub const LIST_SLICE     : Self = Self(36);
    pub const LIST_LEN      : Self = Self(37);
    pub const BYTE         : Self = Self(38);
    pub const LIST_ITER    : Self = Self(39);
    pub const BUILTIN_LIST_ITER : Self = Self(40);
    pub const BUILTIN_LIST_ITER_NEXT : Self = Self(41);
    pub const BUILTIN_FLOAT_SQRT : Self = Self(42);


    pub fn supports_arith(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::BYTE
        )
    }


    pub fn supports_bw(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::BYTE
        )
    }


    pub fn supports_ord(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::BYTE
        )
    }

    pub fn supports_eq(self) -> bool {
        self.is_float() || self.is_num()
    }


    pub fn is_num(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::F64
            | Self::BYTE
        )
    }


    pub fn is_int(self) -> bool {
        matches!(self,
            | Self::I64
            | Self::BYTE
        )
    }

    pub fn is_sint(self) -> bool {
        matches!(self,
            | Self::I64
        )
    }


    pub fn is_float(self) -> bool {
        matches!(self,
            | Self::F64
        )
    }
}
