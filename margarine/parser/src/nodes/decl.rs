use common::{source::SourceRange, string_map::{StringIndex, StringMap}, ImmutableData};
use errors::ErrorId;
use sti::define_key;

use crate::{nodes::NodeId, Block, DataType};

define_key!(pub DeclId(u32));

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Decl<'a> {
    Struct {
        name: StringIndex,
        header: SourceRange,
        fields: &'a [(StringIndex, DataType<'a>, SourceRange)],
        generics: &'a [DeclGeneric<'a>],
    },

    Enum {
        name: StringIndex,
        header: SourceRange,
        mappings: &'a [EnumMapping<'a>],
        generics: &'a [DeclGeneric<'a>],
    },

    Function {
        sig: FunctionSignature<'a>,
        body: Block<'a>,
    },
    
    Impl {
        data_type: DataType<'a>,
        gens: &'a [DeclGeneric<'a>],
        body: Block<'a>,
    },

    ImplTrait {
        header: SourceRange,
        trait_name: DataType<'a>,
        data_type: DataType<'a>,
        gens: &'a [DeclGeneric<'a>],
        body: Block<'a>,
    },

    Using {
        item: UseItem<'a>,
    },

    Module {
        name: StringIndex,
        header: SourceRange,
        body: Block<'a>,
        user_defined: bool,
    },

    ImportFile {
        name: StringIndex,
        body: &'a [NodeId],
    },

    ImportRepo {
        alias: StringIndex,
        repo: StringIndex,
    },

    Extern {
        functions: &'a [ExternFunction<'a>],
    },

    Trait {
        name: StringIndex,
        header: SourceRange,
        functions: &'a [FunctionSignature<'a>],
    },

    OpaqueType {
        name: StringIndex,
        header: SourceRange,
        gens: &'a [DeclGeneric<'a>],
    },

    Attribute {
        attr: StringIndex,
        attr_range: SourceRange,
        decl: DeclId,
    },

    Error(ErrorId),
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FunctionSignature<'a> {
    pub name       : StringIndex,
    pub source     : SourceRange,
    pub arguments  : &'a [FunctionArgument<'a>],
    pub generics   : &'a [DeclGeneric<'a>],
    pub return_type: DataType<'a>,
}


#[derive(Debug, Clone, Copy, PartialEq, ImmutableData)]
pub struct DeclGeneric<'a> {
    name: StringIndex,
    bounds: &'a [DataType<'a>],
}



impl<'a> FunctionSignature<'a> {
    pub fn new(
        name: StringIndex, 
        source: SourceRange, arguments: &'a [FunctionArgument<'a>], 
        generics: &'a [DeclGeneric<'a>], return_type: DataType<'a>) -> Self { 
        Self { name, source, arguments, return_type, generics }
    }
}


impl<'a> DeclGeneric<'a> {
    pub const T : Self = Self::new(StringMap::T, &[]);
    pub const A : Self = Self::new(StringMap::A, &[]);

    pub const fn new(name: StringIndex, bounds: &'a [DataType<'a>]) -> Self {
        Self { name, bounds }
    }
}


#[derive(Debug, PartialEq)]
pub struct ExternFunction<'a> {
    name: StringIndex,
    path: StringIndex,
    gens: &'a [DeclGeneric<'a>],
    args: &'a [FunctionArgument<'a>],
    return_type: DataType<'a>,
    source_range: SourceRange,
}

impl<'a> ExternFunction<'a> {
    pub(crate) fn new(name: StringIndex, path: StringIndex, gens: &'a [DeclGeneric<'a>], args: &'a [FunctionArgument<'a>], return_type: DataType<'a>, source_range: SourceRange) -> Self { 
        Self { name, gens, args, return_type, source_range, path } 
    }


    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn path(&self) -> StringIndex { self.path }
    #[inline(always)]
    pub fn gens(&self) -> &[DeclGeneric<'a>] { &self.gens }
    #[inline(always)]
    pub fn args(&self) -> &[FunctionArgument<'a>] { &self.args }
    #[inline(always)]
    pub fn return_type(&self) -> DataType<'a> { self.return_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }

}


#[derive(Debug, PartialEq)]
pub struct FunctionArgument<'a> {
    name: StringIndex,
    data_type: DataType<'a>,
    source_range: SourceRange,
}


impl<'arena> FunctionArgument<'arena> {
    pub fn new(name: StringIndex, data_type: DataType<'arena>, source_range: SourceRange) -> Self { 
        Self { name, data_type, source_range } 
    }


    #[inline(always)]
    pub fn data_type(&self) -> DataType<'arena> { self.data_type }
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
}


#[derive(Debug, PartialEq)]
pub struct EnumMapping<'a> {
    name: StringIndex,
    number: u16,
    data_type: DataType<'a>,
    source_range: SourceRange,
    is_implicit_unit: bool,
}

impl<'arena> EnumMapping<'arena> {
    pub fn new(name: StringIndex, number: u16, data_type: DataType<'arena>, source_range: SourceRange, is_implicit_unit: bool) -> Self { 
        if is_implicit_unit {
            assert!(data_type.kind().is(&crate::DataTypeKind::Unit));
        }

        Self { name, data_type, source_range, is_implicit_unit, number } 
    }

    
    #[inline(always)]
    pub fn name(&self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn data_type(&self) -> &DataType<'arena> { &self.data_type }
    #[inline(always)]
    pub fn range(&self) -> SourceRange { self.source_range }
    #[inline(always)]
    pub fn is_implicit_unit(&self) -> bool { self.is_implicit_unit }
    #[inline(always)]
    pub fn number(&self) -> u16 { self.number }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UseItem<'a> {
    kind: UseItemKind<'a>,
    name: StringIndex,
    range: SourceRange,
}

impl<'a> UseItem<'a> {
    pub fn new(name: StringIndex, kind: UseItemKind<'a>, range: SourceRange) -> Self { Self { kind, range, name } }
    #[inline(always)]
    pub fn name(self) -> StringIndex { self.name}
    #[inline(always)]
    pub fn kind(self) -> UseItemKind<'a> { self.kind }
    #[inline(always)]
    pub fn range(self) -> SourceRange { self.range }
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UseItemKind<'a> {
    List {
        list: &'a [UseItem<'a>],
    },
    BringName,
    All,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Attribute {
    Startup,
}
