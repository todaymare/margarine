use common::string_map::StringIndex;
use parser::nodes::decl::Generic;
use wasm::WasmFunctionBuilder;

use crate::scope::ScopeId;

use super::ty::Type;

#[derive(Debug, Clone, Copy)] 
pub struct TypeSymbol<'a> {
    display_name: StringIndex,
    kind: TypeSymbolKind<'a>,
}


impl<'a> TypeSymbol<'a> {
    pub fn new(display_name: StringIndex,
               kind: TypeSymbolKind<'a>) -> Self {
        Self { display_name, kind }
    }

    #[inline(always)]
    pub fn display_name(self) -> StringIndex { self.display_name }
    #[inline(always)]
    pub fn kind(self) -> TypeSymbolKind<'a> { self.kind }

    #[inline(always)]
    pub fn as_concrete(self) -> ConcreteType<'a> {
        match self.kind {
            TypeSymbolKind::Template(..) => panic!(),

            TypeSymbolKind::Concrete(conc) => return conc,

            TypeSymbolKind::GenericPlaceholder => return ConcreteType {
                align: 1,
                size: 0,
                kind: ConcreteTypeKind::Struct(ConcreteTypeStruct::new(&[], TypeStructStatus::User)),
            },
        };
    }
}


#[derive(Debug, Clone, Copy)]
pub enum TypeSymbolKind<'a> {
    Concrete(ConcreteType<'a>),
    Template(TemplateType<'a>),
    GenericPlaceholder,
}


#[derive(Debug, Clone, Copy)]
pub struct ConcreteType<'a> {
    pub align: usize,
    pub size: usize,

    pub kind: ConcreteTypeKind<'a>
}

impl<'a> ConcreteType<'a> {
    pub fn new(align: usize, size: usize, kind: ConcreteTypeKind<'a>) -> Self { Self { align, size, kind } }
}


#[derive(Debug, Clone, Copy)]
pub struct TemplateType<'a> {
    pub generics: &'a [Generic],
    pub kind: TemplateTypeKind<'a>
}

impl<'a> TemplateType<'a> {
    pub fn new(generics: &'a [Generic], kind: TemplateTypeKind<'a>) -> Self { Self { generics, kind } }
}



#[derive(Debug, Clone, Copy)]
pub enum TemplateTypeKind<'a> {
    Struct(TemplateTypeStruct<'a>),
    Enum(()),
}


#[derive(Debug, Clone, Copy)]
pub enum ConcreteTypeKind<'a> {
    Struct(ConcreteTypeStruct<'a>),
    Enum(ConcreteTypeEnum<'a>),
}


//
// Struct
//
#[derive(Debug, Clone, Copy)]
pub struct TemplateTypeStruct<'a> {
    pub fields: &'a [StructField],
    pub status: TypeStructStatus,
}

impl<'a> TemplateTypeStruct<'a> {
    pub fn new(fields: &'a [StructField], status: TypeStructStatus) -> Self { Self { fields, status } }
}


#[derive(Debug, Clone, Copy)]
pub struct ConcreteTypeStruct<'a> {
    pub fields: &'a [(StructField, usize)],
    pub status: TypeStructStatus,
}

impl<'a> ConcreteTypeStruct<'a> {
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
pub struct ConcreteTypeEnum<'a> {
    status: ConcreteTypeEnumStatus,
    kind: ConcreteTypeEnumKind<'a>,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConcreteTypeEnumStatus {
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


impl<'a> ConcreteTypeEnum<'a> {
    pub fn new(status: ConcreteTypeEnumStatus, kind: ConcreteTypeEnumKind<'a>) -> Self { Self { status, kind } }

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
    pub fn status(self) -> ConcreteTypeEnumStatus {
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
