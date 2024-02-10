use common::string_map::StringIndex;
use wasm::WasmFunctionBuilder;

use super::ty::Type;

#[derive(Debug, Clone, Copy)] 
pub struct TypeSymbol<'a> {
    display_name: StringIndex,
    align: usize,
    size: usize,
    kind: TypeKind<'a>,
}


impl<'a> TypeSymbol<'a> {
    pub fn new(display_name: StringIndex,
               align: usize,
               size: usize,
               kind: TypeKind<'a>) -> Self {
        Self { display_name, kind, align, size }
    }

    #[inline(always)]
    pub fn display_name(self) -> StringIndex { self.display_name }
    #[inline(always)]
    pub fn kind(self) -> TypeKind<'a> { self.kind }
    #[inline(always)]
    pub fn align(self) -> usize  { self.align}
    #[inline(always)]
    pub fn size(self) -> usize { self.size }
}


#[derive(Debug, Clone, Copy)]
pub enum TypeKind<'a> {
    Struct(TypeStruct<'a>),
    Enum(TypeEnum<'a>),
}


//
// Struct
//
#[derive(Debug, Clone, Copy)]
pub struct TypeStruct<'a> {
    pub fields: &'a [(StructField, usize)],
    pub status: TypeStructStatus,
}

impl<'a> TypeStruct<'a> {
    pub fn new(fields: &'a [(StructField, usize)], status: TypeStructStatus) -> Self { Self { fields, status } }
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
}


impl StructField {
    pub fn new(name: StringIndex, ty: Type) -> Self {
        Self { name, ty }
    }
}


//
// Enum
//
#[derive(Debug, Clone, Copy)]
pub struct TypeEnum<'a> {
    status: TypeEnumStatus,
    kind: ConcreteTypeEnumKind<'a>,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TypeEnumStatus {
    User,
    Result,
    Option,
}


#[derive(Debug, Clone, Copy)]
pub enum ConcreteTypeEnumKind<'a> {
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
    pub fn new(status: TypeEnumStatus, kind: ConcreteTypeEnumKind<'a>) -> Self { Self { status, kind } }

    pub fn get_tag(self, wasm: &mut WasmFunctionBuilder) {
        match self.kind {
            ConcreteTypeEnumKind::TaggedUnion(_) => {
                wasm.i32_read();
            },


            ConcreteTypeEnumKind::Tag(_) => (),
        }
    }


    #[inline(always)]
    pub fn kind(self) -> ConcreteTypeEnumKind<'a> {
        self.kind
    }


    #[inline(always)]
    pub fn status(self) -> TypeEnumStatus {
        self.status
    }
}


impl<'a> ConcreteTypeEnumKind<'a> {
    pub fn get_tag(self, wasm: &mut WasmFunctionBuilder) {
        match self {
            ConcreteTypeEnumKind::TaggedUnion(_) => wasm.i32_read(),
            ConcreteTypeEnumKind::Tag(_) => (), // value on the stack is already the tag
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
