use common::{source::SourceRange, string_map::{OptStringIndex, StringIndex}, ImmutableData};

#[derive(Debug, PartialEq, Clone, Copy, ImmutableData)]
pub struct DataType<'a> {
    range: SourceRange,
    kind: DataTypeKind<'a>, 
}


impl<'a> DataType<'a> {
    pub fn new(source_range: SourceRange, kind: DataTypeKind<'a>) -> Self { 
        Self { range: source_range, kind } 
    }
}


#[derive(Debug, PartialEq, Clone, Copy)]
pub enum DataTypeKind<'a> {
    Unit,
    Never,
    //Hole,
    Tuple(&'a [(OptStringIndex, DataType<'a>)]),
    Within(StringIndex, &'a DataType<'a>),
    CustomType(StringIndex, &'a [DataType<'a>]),
}


impl<'a> DataTypeKind<'a> {
    pub fn is(&self, oth: &DataTypeKind<'a>) -> bool {
        self == &DataTypeKind::Never
        || oth == &DataTypeKind::Never
        || self == oth
    }
}


impl std::hash::Hash for DataTypeKind<'_> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            DataTypeKind::Unit => 102.hash(state),
            DataTypeKind::Never => 103.hash(state),
            //DataTypeKind::Hole => 104.hash(state),

            DataTypeKind::CustomType(v, gens) => {
                300.hash(state);
                v.hash(state);
                gens.iter().for_each(|x| x.kind().hash(state));
            },

            DataTypeKind::Within(name, dt) => {
                204.hash(state);
                name.hash(state);
                dt.kind().hash(state);
            },

            DataTypeKind::Tuple(v) => {
                205.hash(state);
                v.iter().for_each(|x| {
                    x.0.hash(state);
                    x.1.kind().hash(state);
                });
            },
        }
    }
}
