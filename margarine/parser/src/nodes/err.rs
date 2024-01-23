use common::source::SourceRange;
use errors::ErrorId;


#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ErrorNode {
    id: ErrorId,
    source_range: SourceRange,
}


impl ErrorNode {
    pub fn new(id: ErrorId, source_range: SourceRange) -> Self {
        Self { id, source_range }
    }

    pub fn range(self) -> SourceRange { self.source_range }
    pub fn id(self) -> ErrorId { self.id }
}

