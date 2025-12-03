#[repr(u8)]
#[derive(Debug)]
pub enum OpCode {
    Ret,
    Unit,
    PushLocalSpace,
    PopLocalSpace,
    Err,
    ConstInt,
    ConstFloat,
    ConstBool,
    ConstStr,
    Call,
    Pop,
    Copy,
    CreateList,
    CreateStruct,
    LoadField,
    IndexList,
    StoreList,
    StoreField,
    LoadEnumField,
    CreateFuncRef,
    CallFuncRef,
    Unwrap,
    Swap,
    UnwrapFail,
    CastIntToFloat,
    CastFloatToInt,
    CastBoolToInt,
    NegInt,
    AddInt,
    SubInt,
    MulInt,
    DivInt,
    RemInt,
    EqInt,
    NeInt,
    GtInt,
    GeInt,
    LtInt,
    LeInt,
    BitwiseOr,
    BitwiseAnd,
    BitwiseXor,
    BitshiftLeft,
    BitshiftRight,
    NegFloat,
    AddFloat,
    SubFloat,
    MulFloat,
    DivFloat,
    RemFloat,
    EqFloat,
    NeFloat,
    GtFloat,
    GeFloat,
    LtFloat,
    LeFloat,
    EqBool,
    NeBool,
    NotBool,
    Load,
    Store,
    UnwrapStore,
    Jump,
    SwitchOn,
    Switch,
}

#[allow(non_upper_case_globals)]
pub mod consts {
    pub const Ret: u8 = super::OpCode::Ret as u8;
    pub const Unit: u8 = super::OpCode::Unit as u8;
    pub const PushLocalSpace: u8 = super::OpCode::PushLocalSpace as u8;
    pub const PopLocalSpace: u8 = super::OpCode::PopLocalSpace as u8;
    pub const Err: u8 = super::OpCode::Err as u8;
    pub const ConstInt: u8 = super::OpCode::ConstInt as u8;
    pub const ConstFloat: u8 = super::OpCode::ConstFloat as u8;
    pub const ConstBool: u8 = super::OpCode::ConstBool as u8;
    pub const ConstStr: u8 = super::OpCode::ConstStr as u8;
    pub const Call: u8 = super::OpCode::Call as u8;
    pub const Pop: u8 = super::OpCode::Pop as u8;
    pub const Copy: u8 = super::OpCode::Copy as u8;
    pub const CreateList: u8 = super::OpCode::CreateList as u8;
    pub const CreateStruct: u8 = super::OpCode::CreateStruct as u8;
    pub const LoadField: u8 = super::OpCode::LoadField as u8;
    pub const IndexList: u8 = super::OpCode::IndexList as u8;
    pub const StoreList: u8 = super::OpCode::StoreList as u8;
    pub const StoreField: u8 = super::OpCode::StoreField as u8;
    pub const LoadEnumField: u8 = super::OpCode::LoadEnumField as u8;
    pub const CreateFuncRef: u8 = super::OpCode::CreateFuncRef as u8;
    pub const CallFuncRef: u8 = super::OpCode::CallFuncRef as u8;
    pub const Unwrap: u8 = super::OpCode::Unwrap as u8;
    pub const Swap: u8 = super::OpCode::Swap as u8;
    pub const UnwrapFail: u8 = super::OpCode::UnwrapFail as u8;
    pub const CastIntToFloat: u8 = super::OpCode::CastIntToFloat as u8;
    pub const CastFloatToInt: u8 = super::OpCode::CastFloatToInt as u8;
    pub const CastBoolToInt: u8 = super::OpCode::CastBoolToInt as u8;
    pub const NegInt: u8 = super::OpCode::NegInt as u8;
    pub const AddInt: u8 = super::OpCode::AddInt as u8;
    pub const SubInt: u8 = super::OpCode::SubInt as u8;
    pub const MulInt: u8 = super::OpCode::MulInt as u8;
    pub const DivInt: u8 = super::OpCode::DivInt as u8;
    pub const RemInt: u8 = super::OpCode::RemInt as u8;
    pub const EqInt: u8 = super::OpCode::EqInt as u8;
    pub const NeInt: u8 = super::OpCode::NeInt as u8;
    pub const GtInt: u8 = super::OpCode::GtInt as u8;
    pub const GeInt: u8 = super::OpCode::GeInt as u8;
    pub const LtInt: u8 = super::OpCode::LtInt as u8;
    pub const LeInt: u8 = super::OpCode::LeInt as u8;
    pub const BitwiseOr: u8 = super::OpCode::BitwiseOr as u8;
    pub const BitwiseAnd: u8 = super::OpCode::BitwiseAnd as u8;
    pub const BitwiseXor: u8 = super::OpCode::BitwiseXor as u8;
    pub const BitshiftLeft: u8 = super::OpCode::BitshiftLeft as u8;
    pub const BitshiftRight: u8 = super::OpCode::BitshiftRight as u8;
    pub const NegFloat: u8 = super::OpCode::NegFloat as u8;
    pub const AddFloat: u8 = super::OpCode::AddFloat as u8;
    pub const SubFloat: u8 = super::OpCode::SubFloat as u8;
    pub const MulFloat: u8 = super::OpCode::MulFloat as u8;
    pub const DivFloat: u8 = super::OpCode::DivFloat as u8;
    pub const RemFloat: u8 = super::OpCode::RemFloat as u8;
    pub const EqFloat: u8 = super::OpCode::EqFloat as u8;
    pub const NeFloat: u8 = super::OpCode::NeFloat as u8;
    pub const GtFloat: u8 = super::OpCode::GtFloat as u8;
    pub const GeFloat: u8 = super::OpCode::GeFloat as u8;
    pub const LtFloat: u8 = super::OpCode::LtFloat as u8;
    pub const LeFloat: u8 = super::OpCode::LeFloat as u8;
    pub const EqBool: u8 = super::OpCode::EqBool as u8;
    pub const NeBool: u8 = super::OpCode::NeBool as u8;
    pub const NotBool: u8 = super::OpCode::NotBool as u8;
    pub const Load: u8 = super::OpCode::Load as u8;
    pub const Store: u8 = super::OpCode::Store as u8;
    pub const UnwrapStore: u8 = super::OpCode::UnwrapStore as u8;
    pub const Jump: u8 = super::OpCode::Jump as u8;
    pub const SwitchOn: u8 = super::OpCode::SwitchOn as u8;
    pub const Switch: u8 = super::OpCode::Switch as u8;
}

#[allow(non_upper_case_globals, non_snake_case, unused)]
pub mod builder {
    pub struct Builder {
        pub bytecode: Vec<u8>,
    }

    impl Builder {
        pub fn new() -> Self {
            Self { bytecode: vec![] }
        }

        pub fn len(&self) -> usize { self.bytecode.len() }

        pub fn append(&mut self, oth: &Builder) {
            self.bytecode.extend_from_slice(&oth.bytecode);
        }

        pub fn ret(&mut self, local_count: u8) {
            self.bytecode.push(super::OpCode::Ret.as_u8());
            self.bytecode.extend_from_slice(&local_count.to_le_bytes());
        }

        pub fn unit(&mut self) {
            self.bytecode.push(super::OpCode::Unit.as_u8());
        }

        pub fn push_local_space(&mut self, amount: u8) {
            self.bytecode.push(super::OpCode::PushLocalSpace.as_u8());
            self.bytecode.extend_from_slice(&amount.to_le_bytes());
        }

        pub fn pop_local_space(&mut self, amount: u8) {
            self.bytecode.push(super::OpCode::PopLocalSpace.as_u8());
            self.bytecode.extend_from_slice(&amount.to_le_bytes());
        }

        pub fn err(&mut self, ty: u8, file: u32, index: u32) {
            self.bytecode.push(super::OpCode::Err.as_u8());
            self.bytecode.extend_from_slice(&ty.to_le_bytes());
            self.bytecode.extend_from_slice(&file.to_le_bytes());
            self.bytecode.extend_from_slice(&index.to_le_bytes());
        }

        pub fn const_int(&mut self, val: i64) {
            self.bytecode.push(super::OpCode::ConstInt.as_u8());
            self.bytecode.extend_from_slice(&val.to_le_bytes());
        }

        pub fn const_float(&mut self, val: f64) {
            self.bytecode.push(super::OpCode::ConstFloat.as_u8());
            self.bytecode.extend_from_slice(&val.to_le_bytes());
        }

        pub fn const_bool(&mut self, val: u8) {
            self.bytecode.push(super::OpCode::ConstBool.as_u8());
            self.bytecode.extend_from_slice(&val.to_le_bytes());
        }

        pub fn const_str(&mut self, val: u32) {
            self.bytecode.push(super::OpCode::ConstStr.as_u8());
            self.bytecode.extend_from_slice(&val.to_le_bytes());
        }

        pub fn call(&mut self, func: u32, argc: u8) {
            self.bytecode.push(super::OpCode::Call.as_u8());
            self.bytecode.extend_from_slice(&func.to_le_bytes());
            self.bytecode.extend_from_slice(&argc.to_le_bytes());
        }

        pub fn pop(&mut self) {
            self.bytecode.push(super::OpCode::Pop.as_u8());
        }

        pub fn copy(&mut self) {
            self.bytecode.push(super::OpCode::Copy.as_u8());
        }

        pub fn create_list(&mut self, elem_count: u32) {
            self.bytecode.push(super::OpCode::CreateList.as_u8());
            self.bytecode.extend_from_slice(&elem_count.to_le_bytes());
        }

        pub fn create_struct(&mut self, field_count: u8) {
            self.bytecode.push(super::OpCode::CreateStruct.as_u8());
            self.bytecode.extend_from_slice(&field_count.to_le_bytes());
        }

        pub fn load_field(&mut self, field_index: u8) {
            self.bytecode.push(super::OpCode::LoadField.as_u8());
            self.bytecode.extend_from_slice(&field_index.to_le_bytes());
        }

        pub fn index_list(&mut self) {
            self.bytecode.push(super::OpCode::IndexList.as_u8());
        }

        pub fn store_list(&mut self) {
            self.bytecode.push(super::OpCode::StoreList.as_u8());
        }

        pub fn store_field(&mut self, field_index: u8) {
            self.bytecode.push(super::OpCode::StoreField.as_u8());
            self.bytecode.extend_from_slice(&field_index.to_le_bytes());
        }

        pub fn load_enum_field(&mut self, enum_index: u32) {
            self.bytecode.push(super::OpCode::LoadEnumField.as_u8());
            self.bytecode.extend_from_slice(&enum_index.to_le_bytes());
        }

        pub fn create_func_ref(&mut self, capture_count: u8) {
            self.bytecode.push(super::OpCode::CreateFuncRef.as_u8());
            self.bytecode.extend_from_slice(&capture_count.to_le_bytes());
        }

        pub fn call_func_ref(&mut self, argc: u8) {
            self.bytecode.push(super::OpCode::CallFuncRef.as_u8());
            self.bytecode.extend_from_slice(&argc.to_le_bytes());
        }

        pub fn unwrap(&mut self) {
            self.bytecode.push(super::OpCode::Unwrap.as_u8());
        }

        pub fn swap(&mut self) {
            self.bytecode.push(super::OpCode::Swap.as_u8());
        }

        pub fn unwrap_fail(&mut self) {
            self.bytecode.push(super::OpCode::UnwrapFail.as_u8());
        }

        pub fn cast_int_to_float(&mut self) {
            self.bytecode.push(super::OpCode::CastIntToFloat.as_u8());
        }

        pub fn cast_float_to_int(&mut self) {
            self.bytecode.push(super::OpCode::CastFloatToInt.as_u8());
        }

        pub fn cast_bool_to_int(&mut self) {
            self.bytecode.push(super::OpCode::CastBoolToInt.as_u8());
        }

        pub fn neg_int(&mut self) {
            self.bytecode.push(super::OpCode::NegInt.as_u8());
        }

        pub fn add_int(&mut self) {
            self.bytecode.push(super::OpCode::AddInt.as_u8());
        }

        pub fn sub_int(&mut self) {
            self.bytecode.push(super::OpCode::SubInt.as_u8());
        }

        pub fn mul_int(&mut self) {
            self.bytecode.push(super::OpCode::MulInt.as_u8());
        }

        pub fn div_int(&mut self) {
            self.bytecode.push(super::OpCode::DivInt.as_u8());
        }

        pub fn rem_int(&mut self) {
            self.bytecode.push(super::OpCode::RemInt.as_u8());
        }

        pub fn eq_int(&mut self) {
            self.bytecode.push(super::OpCode::EqInt.as_u8());
        }

        pub fn ne_int(&mut self) {
            self.bytecode.push(super::OpCode::NeInt.as_u8());
        }

        pub fn gt_int(&mut self) {
            self.bytecode.push(super::OpCode::GtInt.as_u8());
        }

        pub fn ge_int(&mut self) {
            self.bytecode.push(super::OpCode::GeInt.as_u8());
        }

        pub fn lt_int(&mut self) {
            self.bytecode.push(super::OpCode::LtInt.as_u8());
        }

        pub fn le_int(&mut self) {
            self.bytecode.push(super::OpCode::LeInt.as_u8());
        }

        pub fn bitwise_or(&mut self) {
            self.bytecode.push(super::OpCode::BitwiseOr.as_u8());
        }

        pub fn bitwise_and(&mut self) {
            self.bytecode.push(super::OpCode::BitwiseAnd.as_u8());
        }

        pub fn bitwise_xor(&mut self) {
            self.bytecode.push(super::OpCode::BitwiseXor.as_u8());
        }

        pub fn bitshift_left(&mut self) {
            self.bytecode.push(super::OpCode::BitshiftLeft.as_u8());
        }

        pub fn bitshift_right(&mut self) {
            self.bytecode.push(super::OpCode::BitshiftRight.as_u8());
        }

        pub fn neg_float(&mut self) {
            self.bytecode.push(super::OpCode::NegFloat.as_u8());
        }

        pub fn add_float(&mut self) {
            self.bytecode.push(super::OpCode::AddFloat.as_u8());
        }

        pub fn sub_float(&mut self) {
            self.bytecode.push(super::OpCode::SubFloat.as_u8());
        }

        pub fn mul_float(&mut self) {
            self.bytecode.push(super::OpCode::MulFloat.as_u8());
        }

        pub fn div_float(&mut self) {
            self.bytecode.push(super::OpCode::DivFloat.as_u8());
        }

        pub fn rem_float(&mut self) {
            self.bytecode.push(super::OpCode::RemFloat.as_u8());
        }

        pub fn eq_float(&mut self) {
            self.bytecode.push(super::OpCode::EqFloat.as_u8());
        }

        pub fn ne_float(&mut self) {
            self.bytecode.push(super::OpCode::NeFloat.as_u8());
        }

        pub fn gt_float(&mut self) {
            self.bytecode.push(super::OpCode::GtFloat.as_u8());
        }

        pub fn ge_float(&mut self) {
            self.bytecode.push(super::OpCode::GeFloat.as_u8());
        }

        pub fn lt_float(&mut self) {
            self.bytecode.push(super::OpCode::LtFloat.as_u8());
        }

        pub fn le_float(&mut self) {
            self.bytecode.push(super::OpCode::LeFloat.as_u8());
        }

        pub fn eq_bool(&mut self) {
            self.bytecode.push(super::OpCode::EqBool.as_u8());
        }

        pub fn ne_bool(&mut self) {
            self.bytecode.push(super::OpCode::NeBool.as_u8());
        }

        pub fn not_bool(&mut self) {
            self.bytecode.push(super::OpCode::NotBool.as_u8());
        }

        pub fn load(&mut self, index: u8) {
            self.bytecode.push(super::OpCode::Load.as_u8());
            self.bytecode.extend_from_slice(&index.to_le_bytes());
        }

        pub fn store(&mut self, index: u8) {
            self.bytecode.push(super::OpCode::Store.as_u8());
            self.bytecode.extend_from_slice(&index.to_le_bytes());
        }

        pub fn unwrap_store(&mut self) {
            self.bytecode.push(super::OpCode::UnwrapStore.as_u8());
        }

        pub fn jump(&mut self, offset: i32) {
            self.bytecode.push(super::OpCode::Jump.as_u8());
            self.bytecode.extend_from_slice(&offset.to_le_bytes());
        }

        pub fn switch_on(&mut self, true_offset: i32, false_offset: i32) {
            self.bytecode.push(super::OpCode::SwitchOn.as_u8());
            self.bytecode.extend_from_slice(&true_offset.to_le_bytes());
            self.bytecode.extend_from_slice(&false_offset.to_le_bytes());
        }

        pub fn switch(&mut self, offsets: &[u8]) {
            self.bytecode.push(super::OpCode::Switch.as_u8());
            self.bytecode.extend_from_slice(&( offsets.len() as u32 ).to_le_bytes());
            self.bytecode.extend_from_slice(offsets);
        }

        pub fn ret_at(&mut self, _at: usize, local_count: u8) {
            self.bytecode[_at] = super::OpCode::Ret.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&local_count)].copy_from_slice(&local_count.to_le_bytes());
            _offset += core::mem::size_of_val(&local_count);
        }

        pub fn unit_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::Unit.as_u8();
            let mut _offset = 1;
        }

        pub fn push_local_space_at(&mut self, _at: usize, amount: u8) {
            self.bytecode[_at] = super::OpCode::PushLocalSpace.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&amount)].copy_from_slice(&amount.to_le_bytes());
            _offset += core::mem::size_of_val(&amount);
        }

        pub fn pop_local_space_at(&mut self, _at: usize, amount: u8) {
            self.bytecode[_at] = super::OpCode::PopLocalSpace.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&amount)].copy_from_slice(&amount.to_le_bytes());
            _offset += core::mem::size_of_val(&amount);
        }

        pub fn err_at(&mut self, _at: usize, ty: u8, file: u32, index: u32) {
            self.bytecode[_at] = super::OpCode::Err.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&ty)].copy_from_slice(&ty.to_le_bytes());
            _offset += core::mem::size_of_val(&ty);
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&file)].copy_from_slice(&file.to_le_bytes());
            _offset += core::mem::size_of_val(&file);
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&index)].copy_from_slice(&index.to_le_bytes());
            _offset += core::mem::size_of_val(&index);
        }

        pub fn const_int_at(&mut self, _at: usize, val: i64) {
            self.bytecode[_at] = super::OpCode::ConstInt.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&val)].copy_from_slice(&val.to_le_bytes());
            _offset += core::mem::size_of_val(&val);
        }

        pub fn const_float_at(&mut self, _at: usize, val: f64) {
            self.bytecode[_at] = super::OpCode::ConstFloat.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&val)].copy_from_slice(&val.to_le_bytes());
            _offset += core::mem::size_of_val(&val);
        }

        pub fn const_bool_at(&mut self, _at: usize, val: u8) {
            self.bytecode[_at] = super::OpCode::ConstBool.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&val)].copy_from_slice(&val.to_le_bytes());
            _offset += core::mem::size_of_val(&val);
        }

        pub fn const_str_at(&mut self, _at: usize, val: u32) {
            self.bytecode[_at] = super::OpCode::ConstStr.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&val)].copy_from_slice(&val.to_le_bytes());
            _offset += core::mem::size_of_val(&val);
        }

        pub fn call_at(&mut self, _at: usize, func: u32, argc: u8) {
            self.bytecode[_at] = super::OpCode::Call.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&func)].copy_from_slice(&func.to_le_bytes());
            _offset += core::mem::size_of_val(&func);
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&argc)].copy_from_slice(&argc.to_le_bytes());
            _offset += core::mem::size_of_val(&argc);
        }

        pub fn pop_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::Pop.as_u8();
            let mut _offset = 1;
        }

        pub fn copy_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::Copy.as_u8();
            let mut _offset = 1;
        }

        pub fn create_list_at(&mut self, _at: usize, elem_count: u32) {
            self.bytecode[_at] = super::OpCode::CreateList.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&elem_count)].copy_from_slice(&elem_count.to_le_bytes());
            _offset += core::mem::size_of_val(&elem_count);
        }

        pub fn create_struct_at(&mut self, _at: usize, field_count: u8) {
            self.bytecode[_at] = super::OpCode::CreateStruct.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&field_count)].copy_from_slice(&field_count.to_le_bytes());
            _offset += core::mem::size_of_val(&field_count);
        }

        pub fn load_field_at(&mut self, _at: usize, field_index: u8) {
            self.bytecode[_at] = super::OpCode::LoadField.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&field_index)].copy_from_slice(&field_index.to_le_bytes());
            _offset += core::mem::size_of_val(&field_index);
        }

        pub fn index_list_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::IndexList.as_u8();
            let mut _offset = 1;
        }

        pub fn store_list_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::StoreList.as_u8();
            let mut _offset = 1;
        }

        pub fn store_field_at(&mut self, _at: usize, field_index: u8) {
            self.bytecode[_at] = super::OpCode::StoreField.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&field_index)].copy_from_slice(&field_index.to_le_bytes());
            _offset += core::mem::size_of_val(&field_index);
        }

        pub fn load_enum_field_at(&mut self, _at: usize, enum_index: u32) {
            self.bytecode[_at] = super::OpCode::LoadEnumField.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&enum_index)].copy_from_slice(&enum_index.to_le_bytes());
            _offset += core::mem::size_of_val(&enum_index);
        }

        pub fn create_func_ref_at(&mut self, _at: usize, capture_count: u8) {
            self.bytecode[_at] = super::OpCode::CreateFuncRef.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&capture_count)].copy_from_slice(&capture_count.to_le_bytes());
            _offset += core::mem::size_of_val(&capture_count);
        }

        pub fn call_func_ref_at(&mut self, _at: usize, argc: u8) {
            self.bytecode[_at] = super::OpCode::CallFuncRef.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&argc)].copy_from_slice(&argc.to_le_bytes());
            _offset += core::mem::size_of_val(&argc);
        }

        pub fn unwrap_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::Unwrap.as_u8();
            let mut _offset = 1;
        }

        pub fn swap_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::Swap.as_u8();
            let mut _offset = 1;
        }

        pub fn unwrap_fail_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::UnwrapFail.as_u8();
            let mut _offset = 1;
        }

        pub fn cast_int_to_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::CastIntToFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn cast_float_to_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::CastFloatToInt.as_u8();
            let mut _offset = 1;
        }

        pub fn cast_bool_to_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::CastBoolToInt.as_u8();
            let mut _offset = 1;
        }

        pub fn neg_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NegInt.as_u8();
            let mut _offset = 1;
        }

        pub fn add_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::AddInt.as_u8();
            let mut _offset = 1;
        }

        pub fn sub_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::SubInt.as_u8();
            let mut _offset = 1;
        }

        pub fn mul_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::MulInt.as_u8();
            let mut _offset = 1;
        }

        pub fn div_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::DivInt.as_u8();
            let mut _offset = 1;
        }

        pub fn rem_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::RemInt.as_u8();
            let mut _offset = 1;
        }

        pub fn eq_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::EqInt.as_u8();
            let mut _offset = 1;
        }

        pub fn ne_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NeInt.as_u8();
            let mut _offset = 1;
        }

        pub fn gt_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::GtInt.as_u8();
            let mut _offset = 1;
        }

        pub fn ge_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::GeInt.as_u8();
            let mut _offset = 1;
        }

        pub fn lt_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::LtInt.as_u8();
            let mut _offset = 1;
        }

        pub fn le_int_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::LeInt.as_u8();
            let mut _offset = 1;
        }

        pub fn bitwise_or_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::BitwiseOr.as_u8();
            let mut _offset = 1;
        }

        pub fn bitwise_and_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::BitwiseAnd.as_u8();
            let mut _offset = 1;
        }

        pub fn bitwise_xor_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::BitwiseXor.as_u8();
            let mut _offset = 1;
        }

        pub fn bitshift_left_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::BitshiftLeft.as_u8();
            let mut _offset = 1;
        }

        pub fn bitshift_right_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::BitshiftRight.as_u8();
            let mut _offset = 1;
        }

        pub fn neg_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NegFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn add_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::AddFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn sub_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::SubFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn mul_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::MulFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn div_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::DivFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn rem_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::RemFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn eq_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::EqFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn ne_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NeFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn gt_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::GtFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn ge_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::GeFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn lt_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::LtFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn le_float_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::LeFloat.as_u8();
            let mut _offset = 1;
        }

        pub fn eq_bool_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::EqBool.as_u8();
            let mut _offset = 1;
        }

        pub fn ne_bool_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NeBool.as_u8();
            let mut _offset = 1;
        }

        pub fn not_bool_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::NotBool.as_u8();
            let mut _offset = 1;
        }

        pub fn load_at(&mut self, _at: usize, index: u8) {
            self.bytecode[_at] = super::OpCode::Load.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&index)].copy_from_slice(&index.to_le_bytes());
            _offset += core::mem::size_of_val(&index);
        }

        pub fn store_at(&mut self, _at: usize, index: u8) {
            self.bytecode[_at] = super::OpCode::Store.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&index)].copy_from_slice(&index.to_le_bytes());
            _offset += core::mem::size_of_val(&index);
        }

        pub fn unwrap_store_at(&mut self, _at: usize, ) {
            self.bytecode[_at] = super::OpCode::UnwrapStore.as_u8();
            let mut _offset = 1;
        }

        pub fn jump_at(&mut self, _at: usize, offset: i32) {
            self.bytecode[_at] = super::OpCode::Jump.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&offset)].copy_from_slice(&offset.to_le_bytes());
            _offset += core::mem::size_of_val(&offset);
        }

        pub fn switch_on_at(&mut self, _at: usize, true_offset: i32, false_offset: i32) {
            self.bytecode[_at] = super::OpCode::SwitchOn.as_u8();
            let mut _offset = 1;
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&true_offset)].copy_from_slice(&true_offset.to_le_bytes());
            _offset += core::mem::size_of_val(&true_offset);
            self.bytecode[_at+_offset.._at+_offset+core::mem::size_of_val(&false_offset)].copy_from_slice(&false_offset.to_le_bytes());
            _offset += core::mem::size_of_val(&false_offset);
        }

        pub fn switch_at(&mut self, _at: usize, offsets: &[u8]) {
            self.bytecode[_at] = super::OpCode::Switch.as_u8();
            let mut _offset = 1;
            let _len = offsets.len() as u32;
            self.bytecode[_at+_offset.._at+_offset+4].copy_from_slice(&_len.to_le_bytes());
            _offset += 4;
            self.bytecode[_at+_offset.._at+_offset+_len as usize].copy_from_slice(offsets);
            _offset += _len as usize;
        }
    }

    impl core::fmt::Debug for Builder {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
            use core::fmt::Write;

            let mut strct = f.debug_map();

            let mut iter = crate::Reader::new(&self.bytecode);
            while let Some([opcode]) = iter.try_next_n::<1>() {
                let offset = unsafe { iter.src.offset_from(self.bytecode.as_ptr()) } - 1;

                let Some(opcode) = super::OpCode::from_u8(opcode)
                else {
                    strct.entry(&offset, &"<invalid opcode>".to_string());
                    break;
                };

                match opcode {
                    super::OpCode::Ret => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "local_count: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Ret({fields})",
                        ));
                    }
                    super::OpCode::Unit => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "Unit({fields})",
                        ));
                    }
                    super::OpCode::PushLocalSpace => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "amount: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "PushLocalSpace({fields})",
                        ));
                    }
                    super::OpCode::PopLocalSpace => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "amount: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "PopLocalSpace({fields})",
                        ));
                    }
                    super::OpCode::Err => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "ty: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "file: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "index: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Err({fields})",
                        ));
                    }
                    super::OpCode::ConstInt => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "val: {}",
                                    <i64>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<i64>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "ConstInt({fields})",
                        ));
                    }
                    super::OpCode::ConstFloat => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "val: {}",
                                    <f64>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<f64>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "ConstFloat({fields})",
                        ));
                    }
                    super::OpCode::ConstBool => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "val: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "ConstBool({fields})",
                        ));
                    }
                    super::OpCode::ConstStr => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "val: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "ConstStr({fields})",
                        ));
                    }
                    super::OpCode::Call => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "func: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "argc: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Call({fields})",
                        ));
                    }
                    super::OpCode::Pop => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "Pop({fields})",
                        ));
                    }
                    super::OpCode::Copy => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "Copy({fields})",
                        ));
                    }
                    super::OpCode::CreateList => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "elem_count: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "CreateList({fields})",
                        ));
                    }
                    super::OpCode::CreateStruct => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "field_count: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "CreateStruct({fields})",
                        ));
                    }
                    super::OpCode::LoadField => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "field_index: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "LoadField({fields})",
                        ));
                    }
                    super::OpCode::IndexList => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "IndexList({fields})",
                        ));
                    }
                    super::OpCode::StoreList => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "StoreList({fields})",
                        ));
                    }
                    super::OpCode::StoreField => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "field_index: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "StoreField({fields})",
                        ));
                    }
                    super::OpCode::LoadEnumField => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "enum_index: {}",
                                    <u32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "LoadEnumField({fields})",
                        ));
                    }
                    super::OpCode::CreateFuncRef => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "capture_count: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "CreateFuncRef({fields})",
                        ));
                    }
                    super::OpCode::CallFuncRef => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "argc: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "CallFuncRef({fields})",
                        ));
                    }
                    super::OpCode::Unwrap => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "Unwrap({fields})",
                        ));
                    }
                    super::OpCode::Swap => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "Swap({fields})",
                        ));
                    }
                    super::OpCode::UnwrapFail => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "UnwrapFail({fields})",
                        ));
                    }
                    super::OpCode::CastIntToFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "CastIntToFloat({fields})",
                        ));
                    }
                    super::OpCode::CastFloatToInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "CastFloatToInt({fields})",
                        ));
                    }
                    super::OpCode::CastBoolToInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "CastBoolToInt({fields})",
                        ));
                    }
                    super::OpCode::NegInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NegInt({fields})",
                        ));
                    }
                    super::OpCode::AddInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "AddInt({fields})",
                        ));
                    }
                    super::OpCode::SubInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "SubInt({fields})",
                        ));
                    }
                    super::OpCode::MulInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "MulInt({fields})",
                        ));
                    }
                    super::OpCode::DivInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "DivInt({fields})",
                        ));
                    }
                    super::OpCode::RemInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "RemInt({fields})",
                        ));
                    }
                    super::OpCode::EqInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "EqInt({fields})",
                        ));
                    }
                    super::OpCode::NeInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NeInt({fields})",
                        ));
                    }
                    super::OpCode::GtInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "GtInt({fields})",
                        ));
                    }
                    super::OpCode::GeInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "GeInt({fields})",
                        ));
                    }
                    super::OpCode::LtInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "LtInt({fields})",
                        ));
                    }
                    super::OpCode::LeInt => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "LeInt({fields})",
                        ));
                    }
                    super::OpCode::BitwiseOr => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "BitwiseOr({fields})",
                        ));
                    }
                    super::OpCode::BitwiseAnd => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "BitwiseAnd({fields})",
                        ));
                    }
                    super::OpCode::BitwiseXor => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "BitwiseXor({fields})",
                        ));
                    }
                    super::OpCode::BitshiftLeft => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "BitshiftLeft({fields})",
                        ));
                    }
                    super::OpCode::BitshiftRight => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "BitshiftRight({fields})",
                        ));
                    }
                    super::OpCode::NegFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NegFloat({fields})",
                        ));
                    }
                    super::OpCode::AddFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "AddFloat({fields})",
                        ));
                    }
                    super::OpCode::SubFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "SubFloat({fields})",
                        ));
                    }
                    super::OpCode::MulFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "MulFloat({fields})",
                        ));
                    }
                    super::OpCode::DivFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "DivFloat({fields})",
                        ));
                    }
                    super::OpCode::RemFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "RemFloat({fields})",
                        ));
                    }
                    super::OpCode::EqFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "EqFloat({fields})",
                        ));
                    }
                    super::OpCode::NeFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NeFloat({fields})",
                        ));
                    }
                    super::OpCode::GtFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "GtFloat({fields})",
                        ));
                    }
                    super::OpCode::GeFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "GeFloat({fields})",
                        ));
                    }
                    super::OpCode::LtFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "LtFloat({fields})",
                        ));
                    }
                    super::OpCode::LeFloat => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "LeFloat({fields})",
                        ));
                    }
                    super::OpCode::EqBool => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "EqBool({fields})",
                        ));
                    }
                    super::OpCode::NeBool => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NeBool({fields})",
                        ));
                    }
                    super::OpCode::NotBool => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "NotBool({fields})",
                        ));
                    }
                    super::OpCode::Load => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "index: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Load({fields})",
                        ));
                    }
                    super::OpCode::Store => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "index: {}",
                                    <u8>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<u8>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Store({fields})",
                        ));
                    }
                    super::OpCode::UnwrapStore => {
                        let mut fields = String::new();
                        unsafe {

                        }
                        strct.entry(&offset,
                            &format!(
                            "UnwrapStore({fields})",
                        ));
                    }
                    super::OpCode::Jump => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "offset: {}",
                                    <i32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<i32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Jump({fields})",
                        ));
                    }
                    super::OpCode::SwitchOn => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "true_offset: {}",
                                    <i32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<i32>() }}>()
                                    )
                                ).unwrap();
                            }
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                write!(
                                    &mut fields,
                                    "false_offset: {}",
                                    <i32>::from_le_bytes(
                                        iter.next_n::<{{ core::mem::size_of::<i32>() }}>()
                                    )
                                ).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "SwitchOn({fields})",
                        ));
                    }
                    super::OpCode::Switch => {
                        let mut fields = String::new();
                        unsafe {
                            {
                                if !fields.is_empty() {
                                    fields.push_str(", ");
                                }
                                let len = <u32>::from_le_bytes(iter.next_n::<4>());
                                let data = iter.next_slice(len as usize);
                                write!(&mut fields, "offsets: [len={} bytes]", len).unwrap();
                            }

                        }
                        strct.entry(&offset,
                            &format!(
                            "Switch({fields})",
                        ));
                    }

                }
            }

            strct.finish();
            Ok(())
        }
    }
}

impl OpCode {
    #[inline(always)]
    #[must_use]
    pub fn as_u8(self) -> u8 { self as _ }

    #[inline(always)]
    #[must_use]
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            consts::Ret => Some(Self::Ret),
            consts::Unit => Some(Self::Unit),
            consts::PushLocalSpace => Some(Self::PushLocalSpace),
            consts::PopLocalSpace => Some(Self::PopLocalSpace),
            consts::Err => Some(Self::Err),
            consts::ConstInt => Some(Self::ConstInt),
            consts::ConstFloat => Some(Self::ConstFloat),
            consts::ConstBool => Some(Self::ConstBool),
            consts::ConstStr => Some(Self::ConstStr),
            consts::Call => Some(Self::Call),
            consts::Pop => Some(Self::Pop),
            consts::Copy => Some(Self::Copy),
            consts::CreateList => Some(Self::CreateList),
            consts::CreateStruct => Some(Self::CreateStruct),
            consts::LoadField => Some(Self::LoadField),
            consts::IndexList => Some(Self::IndexList),
            consts::StoreList => Some(Self::StoreList),
            consts::StoreField => Some(Self::StoreField),
            consts::LoadEnumField => Some(Self::LoadEnumField),
            consts::CreateFuncRef => Some(Self::CreateFuncRef),
            consts::CallFuncRef => Some(Self::CallFuncRef),
            consts::Unwrap => Some(Self::Unwrap),
            consts::Swap => Some(Self::Swap),
            consts::UnwrapFail => Some(Self::UnwrapFail),
            consts::CastIntToFloat => Some(Self::CastIntToFloat),
            consts::CastFloatToInt => Some(Self::CastFloatToInt),
            consts::CastBoolToInt => Some(Self::CastBoolToInt),
            consts::NegInt => Some(Self::NegInt),
            consts::AddInt => Some(Self::AddInt),
            consts::SubInt => Some(Self::SubInt),
            consts::MulInt => Some(Self::MulInt),
            consts::DivInt => Some(Self::DivInt),
            consts::RemInt => Some(Self::RemInt),
            consts::EqInt => Some(Self::EqInt),
            consts::NeInt => Some(Self::NeInt),
            consts::GtInt => Some(Self::GtInt),
            consts::GeInt => Some(Self::GeInt),
            consts::LtInt => Some(Self::LtInt),
            consts::LeInt => Some(Self::LeInt),
            consts::BitwiseOr => Some(Self::BitwiseOr),
            consts::BitwiseAnd => Some(Self::BitwiseAnd),
            consts::BitwiseXor => Some(Self::BitwiseXor),
            consts::BitshiftLeft => Some(Self::BitshiftLeft),
            consts::BitshiftRight => Some(Self::BitshiftRight),
            consts::NegFloat => Some(Self::NegFloat),
            consts::AddFloat => Some(Self::AddFloat),
            consts::SubFloat => Some(Self::SubFloat),
            consts::MulFloat => Some(Self::MulFloat),
            consts::DivFloat => Some(Self::DivFloat),
            consts::RemFloat => Some(Self::RemFloat),
            consts::EqFloat => Some(Self::EqFloat),
            consts::NeFloat => Some(Self::NeFloat),
            consts::GtFloat => Some(Self::GtFloat),
            consts::GeFloat => Some(Self::GeFloat),
            consts::LtFloat => Some(Self::LtFloat),
            consts::LeFloat => Some(Self::LeFloat),
            consts::EqBool => Some(Self::EqBool),
            consts::NeBool => Some(Self::NeBool),
            consts::NotBool => Some(Self::NotBool),
            consts::Load => Some(Self::Load),
            consts::Store => Some(Self::Store),
            consts::UnwrapStore => Some(Self::UnwrapStore),
            consts::Jump => Some(Self::Jump),
            consts::SwitchOn => Some(Self::SwitchOn),
            consts::Switch => Some(Self::Switch),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Decoded<'src> {
    Ret { local_count: u8 },
    Unit {  },
    PushLocalSpace { amount: u8 },
    PopLocalSpace { amount: u8 },
    Err { ty: u8, file: u32, index: u32 },
    ConstInt { val: i64 },
    ConstFloat { val: f64 },
    ConstBool { val: u8 },
    ConstStr { val: u32 },
    Call { func: u32, argc: u8 },
    Pop {  },
    Copy {  },
    CreateList { elem_count: u32 },
    CreateStruct { field_count: u8 },
    LoadField { field_index: u8 },
    IndexList {  },
    StoreList {  },
    StoreField { field_index: u8 },
    LoadEnumField { enum_index: u32 },
    CreateFuncRef { capture_count: u8 },
    CallFuncRef { argc: u8 },
    Unwrap {  },
    Swap {  },
    UnwrapFail {  },
    CastIntToFloat {  },
    CastFloatToInt {  },
    CastBoolToInt {  },
    NegInt {  },
    AddInt {  },
    SubInt {  },
    MulInt {  },
    DivInt {  },
    RemInt {  },
    EqInt {  },
    NeInt {  },
    GtInt {  },
    GeInt {  },
    LtInt {  },
    LeInt {  },
    BitwiseOr {  },
    BitwiseAnd {  },
    BitwiseXor {  },
    BitshiftLeft {  },
    BitshiftRight {  },
    NegFloat {  },
    AddFloat {  },
    SubFloat {  },
    MulFloat {  },
    DivFloat {  },
    RemFloat {  },
    EqFloat {  },
    NeFloat {  },
    GtFloat {  },
    GeFloat {  },
    LtFloat {  },
    LeFloat {  },
    EqBool {  },
    NeBool {  },
    NotBool {  },
    Load { index: u8 },
    Store { index: u8 },
    UnwrapStore {  },
    Jump { offset: i32 },
    SwitchOn { true_offset: i32, false_offset: i32 },
    Switch { offsets: &'src [u8] },
}

impl OpCode {
    pub fn decode<'src>(reader: &mut crate::Reader<'src>) -> Option<(Self, Decoded<'src>)> {
        unsafe {
        let [opcode] = reader.try_next_n::<1>()?;
        let opcode = Self::from_u8(opcode)?;

        match opcode {
            Self::Ret => {
        let local_count = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::Ret { local_count }))
            }
            Self::Unit => {

                Some((opcode, Decoded::Unit {  }))
            }
            Self::PushLocalSpace => {
        let amount = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::PushLocalSpace { amount }))
            }
            Self::PopLocalSpace => {
        let amount = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::PopLocalSpace { amount }))
            }
            Self::Err => {
        let ty = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
        let file = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
        let index = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
                Some((opcode, Decoded::Err { ty, file, index }))
            }
            Self::ConstInt => {
        let val = <i64>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<i64>() }>());
                Some((opcode, Decoded::ConstInt { val }))
            }
            Self::ConstFloat => {
        let val = <f64>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<f64>() }>());
                Some((opcode, Decoded::ConstFloat { val }))
            }
            Self::ConstBool => {
        let val = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::ConstBool { val }))
            }
            Self::ConstStr => {
        let val = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
                Some((opcode, Decoded::ConstStr { val }))
            }
            Self::Call => {
        let func = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
        let argc = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::Call { func, argc }))
            }
            Self::Pop => {

                Some((opcode, Decoded::Pop {  }))
            }
            Self::Copy => {

                Some((opcode, Decoded::Copy {  }))
            }
            Self::CreateList => {
        let elem_count = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
                Some((opcode, Decoded::CreateList { elem_count }))
            }
            Self::CreateStruct => {
        let field_count = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::CreateStruct { field_count }))
            }
            Self::LoadField => {
        let field_index = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::LoadField { field_index }))
            }
            Self::IndexList => {

                Some((opcode, Decoded::IndexList {  }))
            }
            Self::StoreList => {

                Some((opcode, Decoded::StoreList {  }))
            }
            Self::StoreField => {
        let field_index = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::StoreField { field_index }))
            }
            Self::LoadEnumField => {
        let enum_index = <u32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u32>() }>());
                Some((opcode, Decoded::LoadEnumField { enum_index }))
            }
            Self::CreateFuncRef => {
        let capture_count = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::CreateFuncRef { capture_count }))
            }
            Self::CallFuncRef => {
        let argc = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::CallFuncRef { argc }))
            }
            Self::Unwrap => {

                Some((opcode, Decoded::Unwrap {  }))
            }
            Self::Swap => {

                Some((opcode, Decoded::Swap {  }))
            }
            Self::UnwrapFail => {

                Some((opcode, Decoded::UnwrapFail {  }))
            }
            Self::CastIntToFloat => {

                Some((opcode, Decoded::CastIntToFloat {  }))
            }
            Self::CastFloatToInt => {

                Some((opcode, Decoded::CastFloatToInt {  }))
            }
            Self::CastBoolToInt => {

                Some((opcode, Decoded::CastBoolToInt {  }))
            }
            Self::NegInt => {

                Some((opcode, Decoded::NegInt {  }))
            }
            Self::AddInt => {

                Some((opcode, Decoded::AddInt {  }))
            }
            Self::SubInt => {

                Some((opcode, Decoded::SubInt {  }))
            }
            Self::MulInt => {

                Some((opcode, Decoded::MulInt {  }))
            }
            Self::DivInt => {

                Some((opcode, Decoded::DivInt {  }))
            }
            Self::RemInt => {

                Some((opcode, Decoded::RemInt {  }))
            }
            Self::EqInt => {

                Some((opcode, Decoded::EqInt {  }))
            }
            Self::NeInt => {

                Some((opcode, Decoded::NeInt {  }))
            }
            Self::GtInt => {

                Some((opcode, Decoded::GtInt {  }))
            }
            Self::GeInt => {

                Some((opcode, Decoded::GeInt {  }))
            }
            Self::LtInt => {

                Some((opcode, Decoded::LtInt {  }))
            }
            Self::LeInt => {

                Some((opcode, Decoded::LeInt {  }))
            }
            Self::BitwiseOr => {

                Some((opcode, Decoded::BitwiseOr {  }))
            }
            Self::BitwiseAnd => {

                Some((opcode, Decoded::BitwiseAnd {  }))
            }
            Self::BitwiseXor => {

                Some((opcode, Decoded::BitwiseXor {  }))
            }
            Self::BitshiftLeft => {

                Some((opcode, Decoded::BitshiftLeft {  }))
            }
            Self::BitshiftRight => {

                Some((opcode, Decoded::BitshiftRight {  }))
            }
            Self::NegFloat => {

                Some((opcode, Decoded::NegFloat {  }))
            }
            Self::AddFloat => {

                Some((opcode, Decoded::AddFloat {  }))
            }
            Self::SubFloat => {

                Some((opcode, Decoded::SubFloat {  }))
            }
            Self::MulFloat => {

                Some((opcode, Decoded::MulFloat {  }))
            }
            Self::DivFloat => {

                Some((opcode, Decoded::DivFloat {  }))
            }
            Self::RemFloat => {

                Some((opcode, Decoded::RemFloat {  }))
            }
            Self::EqFloat => {

                Some((opcode, Decoded::EqFloat {  }))
            }
            Self::NeFloat => {

                Some((opcode, Decoded::NeFloat {  }))
            }
            Self::GtFloat => {

                Some((opcode, Decoded::GtFloat {  }))
            }
            Self::GeFloat => {

                Some((opcode, Decoded::GeFloat {  }))
            }
            Self::LtFloat => {

                Some((opcode, Decoded::LtFloat {  }))
            }
            Self::LeFloat => {

                Some((opcode, Decoded::LeFloat {  }))
            }
            Self::EqBool => {

                Some((opcode, Decoded::EqBool {  }))
            }
            Self::NeBool => {

                Some((opcode, Decoded::NeBool {  }))
            }
            Self::NotBool => {

                Some((opcode, Decoded::NotBool {  }))
            }
            Self::Load => {
        let index = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::Load { index }))
            }
            Self::Store => {
        let index = <u8>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<u8>() }>());
                Some((opcode, Decoded::Store { index }))
            }
            Self::UnwrapStore => {

                Some((opcode, Decoded::UnwrapStore {  }))
            }
            Self::Jump => {
        let offset = <i32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<i32>() }>());
                Some((opcode, Decoded::Jump { offset }))
            }
            Self::SwitchOn => {
        let true_offset = <i32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<i32>() }>());
        let false_offset = <i32>::from_le_bytes(reader.next_n::<{ core::mem::size_of::<i32>() }>());
                Some((opcode, Decoded::SwitchOn { true_offset, false_offset }))
            }
            Self::Switch => {
        let _len = <u32>::from_le_bytes(reader.next_n::<4>());
        let offsets = reader.next_slice(_len as usize);
                Some((opcode, Decoded::Switch { offsets }))
            }
        }
        }
    }
}