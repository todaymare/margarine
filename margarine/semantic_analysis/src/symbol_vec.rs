use common::{string_map::StringIndex, source::SourceRange};
use parser::{DataType, nodes::{StructKind, EnumMapping, FunctionArgument, Node}};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Default, Debug, Hash)]
pub struct SymbolId(pub usize);


#[derive(Debug,PartialEq)]
pub struct Symbols<'a>{
    vec: Vec<Symbol<'a>>,
}


impl<'a> Symbols<'a>{
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    
    #[inline(always)]
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            vec: Vec::with_capacity(cap)
        }
    }

    
    #[inline(always)]
    pub fn push(&mut self, value: Symbol<'a>) -> SymbolId {
        self.vec.push(value);
        SymbolId(self.vec.len()-1)
    }

    
    #[inline(always)]
    pub fn get(&self, index: SymbolId) -> Option<&Symbol<'a> >{
        self.vec.get(index.0)
    }

    
    #[inline(always)]
    pub fn get_mut(&mut self, index: SymbolId) -> Option<&mut Symbol<'a> >{
        self.vec.get_mut(index.0)
    }

    
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.vec.len()
    }

    
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.vec.capacity()
    }

    
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.vec.is_empty()
    }

    
    #[inline(always)]
    pub fn as_slice(&self) ->  &[Symbol<'a>] {
        &self.vec
    }


    
    #[inline(always)]
    pub fn as_mut(&mut self) ->  &mut [Symbol<'a>] {
        &mut self.vec
    }
}


impl<'a> core::ops::Index<SymbolId> for Symbols<'a> {
    type Output = Symbol<'a>;


    fn index(&self, key: SymbolId) ->  &Self::Output {
        &self.vec[key.0]
    }
}


impl<'a> core::ops::IndexMut<SymbolId> for Symbols<'a> {
    fn index_mut(&mut self, key: SymbolId) ->  &mut Self::Output {
        &mut self.vec[key.0]
    }

}


#[derive(Debug, PartialEq)]
pub enum Symbol<'arena> {
    Structure(Structure<'arena>),
    Enum(Enum<'arena>),
    Function(Function<'arena>),
}


impl Symbol<'_> {
    pub fn full_name(&self) -> StringIndex {
        match self {
            Symbol::Structure(v) => v.full_name,
            Symbol::Enum(v) => v.full_name,
            Symbol::Function(v) => v.full_name,
        }
    }


    pub fn type_name(&self) -> &'static str {
        match self {
            Symbol::Structure(_) => "structure",
            Symbol::Enum(_) => "enum",
            Symbol::Function(_) => "function",
        }
    }


    pub fn range(&self) -> SourceRange {
        match self {
            Symbol::Structure(v) => v.source,
            Symbol::Enum(v) => v.source,
            Symbol::Function(v) => v.source,
        }
    }
}


#[derive(Debug, PartialEq)]
pub struct Structure<'arena> {
    pub name: StringIndex,
    pub full_name: StringIndex,
    pub fields: &'arena mut [(StringIndex, DataType<'arena>, SourceRange)],
    pub kind: StructKind,
    pub source: SourceRange
}


#[derive(Debug, PartialEq)]
pub struct Enum<'arena> {
    pub name: StringIndex,
    pub full_name: StringIndex,
    pub mappings: &'arena [EnumMapping<'arena>],
    pub source: SourceRange,
}


#[derive(Debug, PartialEq)]
pub struct Function<'arena> {
    pub name: StringIndex,
    pub full_name: StringIndex,
    pub args: &'arena mut [FunctionArgument<'arena>],
    pub is_extern: Option<(StringIndex, StringIndex)>,
    pub is_system: bool,
    pub is_anonymous: bool,
    pub is_enum_variant_function: Option<u16>,
    pub return_type: DataType<'arena>,
    pub source: SourceRange,

    pub body: &'arena mut [Node<'arena>],
}


