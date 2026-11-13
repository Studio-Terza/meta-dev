use std::collections::BTreeSet;
use std::fmt;

use lint_core::TypeName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge<'found> {
    source: &'found TypeName,
    target: &'found TypeName,
}

impl<'found> Edge<'found> {
    pub fn new(source: &'found TypeName, target: &'found TypeName) -> Edge<'found> {
        Edge { source, target }
    }

    pub fn target_from(&self, seen: &BTreeSet<&TypeName>) -> Option<&'found TypeName> {
        seen.contains(self.source).then_some(self.target)
    }

    #[must_use]
    pub fn reversed(self) -> Edge<'found> {
        Edge { source: self.target, target: self.source }
    }
}

impl fmt::Display for Edge<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.source, self.target)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeKey {
    source: TypeName,
    target: TypeName,
}

impl EdgeKey {
    pub fn source(&self) -> &TypeName {
        &self.source
    }
}

impl From<Edge<'_>> for EdgeKey {
    fn from(edge: Edge<'_>) -> EdgeKey {
        EdgeKey { source: edge.source.clone(), target: edge.target.clone() }
    }
}

impl<'key> From<&'key EdgeKey> for Edge<'key> {
    fn from(key: &'key EdgeKey) -> Edge<'key> {
        Edge { source: &key.source, target: &key.target }
    }
}
