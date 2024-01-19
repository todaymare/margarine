use common::string_map::StringIndex;
use wasm::WasmFunctionBuilder;

use super::ty::Type;

#[derive(Debug, Clone, Copy)] 
pub struct TypeSymbol<'a> {
    display_name: StringIndex,

    align: usize,
    size: usize,

    kind: TypeSymbolKind<'a>,
}


impl<'a> TypeSymbol<'a> {
    pub fn new(display_name: StringIndex, align: usize,
               size: usize, kind: TypeSymbolKind<'a>) -> Self {
        Self { display_name, align, size, kind }
    }

    #[inline(always)]
    pub fn display_name(self) -> StringIndex { self.display_name }
    #[inline(always)]
    pub fn align(self) -> usize { self.align }
    #[inline(always)]
    pub fn size(self) -> usize { self.size }
    #[inline(always)]
    pub fn kind(self) -> TypeSymbolKind<'a> { self.kind }
}


#[derive(Debug, Clone, Copy)]
pub enum TypeSymbolKind<'a> {
    Struct(TypeStruct<'a>),
    Enum(TypeEnum<'a>),
}


//
// Struct
//
#[derive(Debug, Clone, Copy)]
pub struct TypeStruct<'a> {
    pub fields: &'a [StructField],
    pub status: TypeStructStatus,
}

impl<'a> TypeStruct<'a> {
    pub fn new(fields: &'a [StructField], status: TypeStructStatus) -> Self { Self { fields, status } }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeStructStatus {
    User,
    Tuple,
    Rc,
    RcMut,
}


#[derive(Debug, Clone, Copy)]
pub struct StructField {
    pub name: StringIndex,
    pub ty: Type,
    pub offset: usize,
}

impl StructField {
    pub fn new(name: StringIndex, ty: Type, offset: usize) -> Self {
        Self { name, ty, offset }
    }
}


//
// Enum
//
#[derive(Debug, Clone, Copy)]
pub struct TypeEnum<'a> {
    status: TypeEnumStatus,
    kind: TypeEnumKind<'a>,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeEnumStatus {
    User,
    Result,
    Option,
}


#[derive(Debug, Clone, Copy)]
pub enum TypeEnumKind<'a> {
    TaggedUnion(TypeTaggedUnion<'a>),
    Tag(TypeTag<'a>),
}


#[derive(Debug, Clone, Copy)]
pub struct TypeTaggedUnion<'a> {
    union_offset: u32,
    mappings: &'a [TaggedUnionField]
}


#[derive(Debug, Clone, Copy)]
pub struct TaggedUnionField {
    name: StringIndex,
    ty: Option<Type>,
}


#[derive(Debug, Clone, Copy)]
pub struct TypeTag<'a> {
    tags: &'a [StringIndex]
}


impl<'a> TypeEnum<'a> {
    pub fn new(status: TypeEnumStatus, kind: TypeEnumKind<'a>) -> Self { Self { status, kind } }

    pub fn get_tag(self, wasm: &mut WasmFunctionBuilder) {
        match self.kind {
            TypeEnumKind::TaggedUnion(_) => {
                wasm.i32_read();
            },


            TypeEnumKind::Tag(_) => (),
        }
    }


    #[inline(always)]
    pub fn kind(self) -> TypeEnumKind<'a> {
        self.kind
    }


    #[inline(always)]
    pub fn status(self) -> TypeEnumStatus {
        self.status
    }
}


impl<'a> TypeEnumKind<'a> {
    pub fn get_tag(self, wasm: &mut WasmFunctionBuilder) {
        match self {
            TypeEnumKind::TaggedUnion(_) => wasm.i32_read(),
            TypeEnumKind::Tag(_) => (), // value on the stack is already the tag
        }
    }
}


impl<'a> TypeTaggedUnion<'a> {
    pub fn new(union_offset: u32, mappings: &'a [TaggedUnionField]) -> Self { Self { union_offset, mappings } }
    pub fn fields(self) -> &'a [TaggedUnionField] { self.mappings }
    pub fn union_offset(self) -> u32 { self.union_offset }
}


impl TaggedUnionField {
    pub fn new(name: StringIndex, ty: Option<Type>) -> Self { Self { name, ty } }
    pub fn ty(self) -> Option<Type> { self.ty }
    pub fn name(self) -> StringIndex { self.name }
}


impl<'a> TypeTag<'a> {
    pub fn new(tags: &'a [StringIndex]) -> Self { Self { tags } }
    pub fn fields(self) -> &'a [StringIndex] { self.tags }
}
