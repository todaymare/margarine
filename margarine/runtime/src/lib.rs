pub mod bytecode;
pub mod runtime;
pub mod ecs;

use std::{mem::{size_of, ManuallyDrop}, fmt::Debug, collections::HashMap, rc::Rc, ptr::null};

// static_assert_eq!(size_of::<DataUnion>(), 8);
// static_assert_eq!(size_of::<Data>(), 16);


#[repr(C)]
#[derive(Debug)]
pub struct VM<'a> {
    callstack: Vec<Stackframe>,
    current: ProgramCounter,
    pub stack: Stack,
    constants: Box<[Data]>,

    name_to_typeid: HashMap<&'a str, TypeId>,
    name_to_func: HashMap<&'a str, FunctionIndex>,
    structures: HashMap<TypeId, Structure<'a>>,
    functions: Vec<Code>,
    functions_debug_info: Vec<FunctionDebugInfo<'a>>,
}


impl VM<'_> {
    pub fn new(
        stack: Stack, 
        metadata: CompilerMetadata,
        constants: Box<[Data]>
    ) -> Self { 
        Self { 
            callstack: Vec::new(),
            current: ProgramCounter::new(Code::new(null(), null())),
            stack, 
            constants,

            name_to_typeid: HashMap::with_capacity(metadata.num_of_structs), 
            name_to_func: HashMap::with_capacity(metadata.num_of_functions),
            structures: HashMap::with_capacity(metadata.num_of_structs), 
            functions: Vec::with_capacity(metadata.num_of_functions), 
            functions_debug_info: Vec::with_capacity(metadata.num_of_functions),
        } 
    }

    pub fn query_type(&self, qualified_name: &str) -> Option<TypeId> {
        self.name_to_typeid.get(qualified_name).copied()
    }


    pub fn query_function(&self, qualified_name: &str) -> Option<FunctionIndex> {
        self.name_to_func.get(qualified_name).copied()
    }


    pub fn get_type(&self, type_id: TypeId) -> Option<&Structure> {
        self.structures.get(&type_id)
    }


    pub fn get_function_block(&self, id: FunctionIndex) -> Code {
        self.functions[id.0]
    }


    pub fn get_function_info(&self, id: FunctionIndex) -> &FunctionDebugInfo {
        &self.functions_debug_info[id.0]
    }
}


#[repr(C)]
#[derive(Debug)]
pub struct Stack {
    values: Vec<Data>,
    bottom: usize,
    top: usize,
}


impl Stack {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: vec![Data::new_unit(); capacity],
            bottom: 0,
            top: 0,
        }
    }

    
    pub fn set_reg(&mut self, reg: u8, data: Data) {
        debug_assert!((self.bottom + reg as usize) < self.top);
        unsafe { *self.values.get_unchecked_mut(self.bottom + reg as usize) = data };
    }

    
    pub fn reg(&self, reg: u8) -> &Data {
        &self.values[self.bottom+reg as usize]
    }

    
    pub fn reg_mut(&mut self, reg: u8) -> &mut Data {
        &mut self.values[self.bottom+reg as usize]
    }


    pub fn take_reg(&mut self, reg: u8) -> Data {
        std::mem::replace(self.reg_mut(reg), Data::new_unit())
    }


    pub fn push(&mut self, amount: usize) {
        self.top += amount;
        debug_assert!(self.top <= self.values.len())
    }

    
    pub fn pop(&mut self, amount: usize) {
        self.top -= amount;
        debug_assert!(self.top >= self.bottom)
    }

}


#[derive(Clone, Copy, Debug)]
pub struct Code {
    start: *const u8,

    #[cfg(debug_assertions)]
    end: *const u8,
}


impl Code {
    pub fn new(
        start: *const u8,

        #[cfg(debug_assertions)]
        end: *const u8,
    ) -> Self {
        Self {
            start,

            #[cfg(debug_assertions)]
            end,
            
        }
    }
}


#[derive(Debug, Clone)]
pub struct ProgramCounter {
    code: Code,
    counter: *const u8,
}


impl ProgramCounter {
    pub fn new(code: Code) -> Self {
        Self {
            code,
            counter: code.start,
        }
    }


    ///
    /// Increments the program counter
    /// by 1, returning the old value
    ///
    #[inline(always)]
    pub fn next(&mut self) -> u8 {
        #[cfg(debug_assertions)]
        assert!(self.counter <= self.code.end);

        let val = unsafe { *self.counter };
        unsafe { self.counter = self.counter.add(1) };
        
        val
    }

    ///
    /// Returns an array of the next N
    /// elements and increments the program
    /// counter by N
    ///
    #[inline(always)]
    pub fn next_n<const N: usize>(&mut self) -> [u8; N] {
        #[cfg(debug_assertions)]
        assert!(unsafe { self.counter.add(N) } <= self.code.end);

        std::array::from_fn(|_| self.next())
    }


    ///
    /// Jumps to the given offset using the
    /// `Code`'s start given when creating `Self`
    ///
    #[inline(always)]
    pub fn jump(&mut self, offset: usize) {
        self.counter = unsafe { self.code.start.add(offset) };
        #[cfg(debug_assertions)]
        {
            assert!(self.counter <= self.code.end);
            assert!(self.counter >= self.code.start);
        }
    }


    ///
    /// Skips `n` amount of bytes
    ///
    #[inline(always)]
    pub fn skip(&mut self, n: usize) {
        self.counter = unsafe { self.counter.add(n) };
        #[cfg(debug_assertions)]
        assert!(self.counter <= self.code.end);
    }


    ///
    /// Reads the next 2 bytes as a u16
    ///
    #[inline(always)]
    pub fn next_u16(&mut self) -> u16 {
        u16::from_le_bytes(self.next_n::<2>())
    }


    ///
    /// Reads the next 4 bytes as a u32
    ///
    #[inline(always)]
    pub fn next_u32(&mut self) -> u32 {
        u32::from_le_bytes(self.next_n::<4>())
    }


    ///
    /// Reads the next bytes as a string
    /// The string format is the following
    /// - `u32` for the `length` in bytes
    /// - The next `length` bytes are treated
    ///   as UTF-8
    ///
    #[inline(always)]
    pub fn next_str<'b>(&mut self) -> &'b str {
        let size = self.next_u32();

        #[cfg(debug_assertions)]
        assert!(unsafe { self.counter.add(size as usize)} <= self.code.end);

        let slice = unsafe { std::slice::from_raw_parts(self.counter, size as usize) };
        let string = unsafe { std::str::from_utf8_unchecked(slice) };

        self.counter = unsafe { self.counter.add(size as usize) };
        
        string
    }

    ///
    /// Reads the next bytes as a `TypeId`
    ///
    #[inline(always)]
    pub fn next_type(&mut self) -> TypeId {
        TypeId(self.next_u32())
    }
}


#[derive(Debug)]
pub struct Stackframe {
    pub pc: ProgramCounter,
    pub stack_bottom: usize,
    pub dst: u8,
}


#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);


impl TypeId {
    pub const INT_TAG  : TypeId = TypeId(1);
    pub const FLOAT_TAG: TypeId = TypeId(2);
    pub const UINT_TAG : TypeId = TypeId(3);
    pub const BOOL_TAG : TypeId = TypeId(4);
    pub const UNIT_TAG : TypeId = TypeId(0);
    pub const CAST_ERROR: TypeId = TypeId(256);
}


#[repr(C)]
pub struct Data {
    type_id: TypeId,
    metadata: DataMetadata,
    data: DataUnion,
}


#[repr(C)]
pub union DataUnion {
    int: i64,
    float: f64,
    uint: u64,
    bool: bool,
    unit: Unit,
    obj: ManuallyDrop<Object>,
}


#[derive(Clone)]
pub enum Object {
    String(Rc<str>),
    Structure(Rc<[Data]>),
}


#[repr(C)]
#[derive(Clone, Copy, Debug)]
// only keeping it like this in case
// i add more stuff in the metadata
pub struct DataMetadata(u32);


impl DataMetadata {
    pub fn variant(self) -> u32 { self.0 }
}


impl Clone for Data {
    fn clone(&self) -> Self {
        Self {
            type_id: self.type_id,
            metadata: self.metadata,
            data: unsafe { match self.type_id {
                TypeId::INT_TAG => DataUnion { int: self.data.int },
                TypeId::FLOAT_TAG => DataUnion { float: self.data.float },
                TypeId::UINT_TAG => DataUnion { uint: self.data.uint },
                TypeId::BOOL_TAG => DataUnion { bool: self.data.bool },
                TypeId::UNIT_TAG => DataUnion { unit: self.data.unit },

                _ => DataUnion { obj: self.data.obj.clone() }
            } },
        }
    }
}


impl Drop for Data {
    fn drop(&mut self) {
         match self.type_id {
            | TypeId::INT_TAG
            | TypeId::FLOAT_TAG
            | TypeId::UINT_TAG
            | TypeId::BOOL_TAG
            | TypeId::UNIT_TAG
             => (),

            _ => unsafe { ManuallyDrop::drop(&mut self.data.obj) }
        }
    }
}


macro_rules! def_data_is_as {
    ($ty: ty, $is: ident, $as: ident, $tag: ident, $field: ident) => {
        ///
        /// Compares the type id against 
        /// the given types type id.
        ///
        #[inline(always)]
        pub fn $is(&self) -> bool { self.type_id == TypeId::$tag }

        ///
        /// Accesses the given types union
        /// field. Without debug assertions
        /// using this on a value that is not
        /// the given type will cause UB
        ///
        /// # In Debug Mode:
        /// Asserts that the value is of
        /// the given type.
        ///
        #[inline(always)]
        pub fn $as(&self) -> $ty {
            debug_assert!(self.$is());
            unsafe { self.data.$field }
        }
    }
    
}


impl Data {
    def_data_is_as!(i64 , is_int  , as_int  , INT_TAG  , int  );
    def_data_is_as!(f64 , is_float, as_float, FLOAT_TAG, float);
    def_data_is_as!(u64 , is_uint , as_uint , UINT_TAG , uint );
    def_data_is_as!(bool, is_bool , as_bool , BOOL_TAG , bool );
    def_data_is_as!(Unit, is_unit , as_unit , UNIT_TAG , unit );


    fn new(data: DataUnion, metadata: DataMetadata, type_id: TypeId) -> Self {
        Self {
            type_id,
            metadata,
            data,
        }
    }
    

    pub fn new_unit() -> Self {
        Self::new(DataUnion { unit: Unit::new() }, DataMetadata(0), TypeId::UNIT_TAG)
    }

    pub fn new_int(val: i64) -> Self {
        Self::new(DataUnion { int: val }, DataMetadata(0), TypeId::INT_TAG)
    }

    pub fn new_uint(val: u64) -> Self {
        Self::new(DataUnion { uint: val }, DataMetadata(0), TypeId::UINT_TAG)
    }

    pub fn new_bool(val: bool) -> Self {
        Self::new(DataUnion { bool: val }, DataMetadata(0), TypeId::BOOL_TAG)
    }

    pub fn new_float(val: f64) -> Self {
        Self::new(DataUnion { float: val }, DataMetadata(0), TypeId::FLOAT_TAG)
    }

    pub fn new_obj(val: Object, type_id: TypeId) -> Self {
        Self::new(DataUnion { obj: ManuallyDrop::new(val) }, DataMetadata(0), type_id)
    }

    
    ///
    /// Compares the type id against 
    /// the given types type id.
    ///
    #[inline(always)]
    pub fn is_obj(&self) -> bool { self.type_id.0 >= 256 }

    ///
    /// Accesses the given types union
    /// field. Without debug assertions
    /// using this on a value that is not
    /// the given type will cause UB
    ///
    /// # In Debug Mode:
    /// Asserts that the value is of
    /// the given type.
    ///
    #[inline(always)]
    pub fn as_obj(&self) -> &Object {
        debug_assert!(self.is_obj());
        unsafe { &self.data.obj }
    }

    ///
    /// Accesses the given types union
    /// field. Without debug assertions
    /// using this on a value that is not
    /// the given type will cause UB
    ///
    /// # In Debug Mode:
    /// Asserts that the value is of
    /// the given type.
    ///
    #[inline(always)]
    pub fn as_obj_mut(&mut self) -> &mut Object {
        debug_assert!(self.is_obj());
        unsafe { &mut self.data.obj }
    }
}


impl Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Data {{ type_id: {:?}, metadata: {:?}, data: {{ {}: {} }} }}",
            self.type_id,
            self.metadata,
            match self.type_id {
                TypeId::UNIT_TAG => "unit",
                TypeId::INT_TAG => "int",
                TypeId::UINT_TAG => "uint",
                TypeId::BOOL_TAG => "bool",
                TypeId::FLOAT_TAG => "float",
                _ => "obj",
            },
            unsafe { match self.type_id {
                TypeId::UNIT_TAG => "unit".to_string(),
                TypeId::INT_TAG => self.data.int.to_string(),
                TypeId::UINT_TAG => self.data.uint.to_string(),
                TypeId::BOOL_TAG => self.data.bool.to_string(),
                TypeId::FLOAT_TAG => self.data.float.to_string(),
                _ => 0.to_string(),
            } },
        )
    }
}


impl Object {
    pub fn as_str(&self) -> &str {
        match self {
            Object::String(v) => v,

            _ => unreachable!()
        }
    }

    
    pub fn as_str_mut(&mut self) -> &mut Rc<str> {
        match self {
            Object::String(v) => v,

            _ => unreachable!()
        }
    }

    
    pub fn as_struct(&self) -> &Rc<[Data]> {
        match self {
            Object::Structure(v) => v,

            _ => unreachable!()
        }
    }

    
    pub fn as_struct_mut(&mut self) -> &mut [Data] {
        match self {
            Object::Structure(v) => {
                make_mut_slice(v)
            },

            _ => unreachable!()
        }
    }
}


#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Unit { 
    _filler: u8  // Cos C doesn't have zero sized types
}


impl Unit {
    pub extern "C" fn new() -> Unit { Unit { _filler: 0 } }
}

impl Default for Unit {
    fn default() -> Self { Self::new() }
}


#[derive(Debug)]
pub struct Structure<'a> {
    name: &'a str,
    type_id: TypeId,
    fields: Vec<(&'a str, TypeId)>,
    kind: StructureKind,
}


impl Structure<'_> {
    pub fn name(&self) -> &str { &self.name }
    pub fn type_id(&self) -> TypeId { self.type_id }
    pub fn fields(&self) -> &[(&str, TypeId)] { &self.fields }
    pub fn kind(&self) -> StructureKind { self.kind }
}


#[derive(Clone, Copy, Debug)]
pub enum StructureKind {
    Normal,
    Resource,
    Component,
}


#[derive(Debug)]
pub struct FunctionDebugInfo<'a> {
    name: &'a str,
    return_type: TypeId,
    arguments: Vec<FunctionArgument<'a>>,
    is_system: bool,
}


impl FunctionDebugInfo<'_> {
    pub fn name(&self) -> &str { &self.name }
    pub fn return_type(&self) -> TypeId { self.return_type }
    pub fn arguments(&self) -> &[FunctionArgument] { &self.arguments }
    pub fn is_system(&self) -> bool { self.is_system }
}


#[derive(Debug)]
pub struct FunctionArgument<'a> {
    name: &'a str,
    data_type: TypeId,
    is_inout: bool,
}


#[derive(Clone, Copy, Debug)]
pub struct FunctionIndex(usize);


pub struct CompilerMetadata {
    pub num_of_functions: usize,
    pub num_of_structs: usize,
}


fn make_mut_slice<T: Clone>(rc: &mut Rc<[T]>) -> &mut [T] {
    if Rc::get_mut(rc).is_none() {
        *rc = Rc::from(rc.as_ref());
    }

    let r = Rc::get_mut(rc);
    debug_assert!(r.is_some());

    unsafe { r.unwrap_unchecked() }
}
