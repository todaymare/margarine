use common::{source::SourceRange, string_map::StringIndex};

use super::Node;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct AttributeNode<'a> {
    kind: Attribute<'a>,
    node: Node<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Attribute<'a> {
    subs: &'a [Attribute<'a>],
    name: StringIndex,
    range: SourceRange,
}

impl<'a> Attribute<'a> {
    pub fn new(name: StringIndex, subs: &'a [Attribute<'a>], range: SourceRange) -> Self { Self { subs, range, name } }
    #[inline(always)]
    pub fn name(self) -> StringIndex { self.name }
    #[inline(always)]
    pub fn range(self) -> SourceRange { self.range }
    #[inline(always)]
    pub fn subs(self) -> &'a [Attribute<'a>] { self.subs }
}


impl<'a> AttributeNode<'a> {
    pub fn new(attr: Attribute<'a>, node: Node<'a>) -> Self {
        Self {
            kind: attr,
            node,
        }
    }

    #[inline(always)]
    pub fn range(self) -> SourceRange { SourceRange::new(self.attr().range().start(), self.node().range().end())}
    #[inline(always)]
    pub fn node(self) -> Node<'a> { self.node }
    #[inline(always)]
    pub fn attr(self) -> Attribute<'a> { self.kind }
}
