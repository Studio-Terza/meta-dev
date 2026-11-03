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

    pub fn source(&self) -> &'found TypeName {
        self.source
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
