use std::collections::BTreeSet;
use std::fmt;

use lint_core::TypeName;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Edge {
    source: TypeName,
    target: TypeName,
}

impl Edge {
    pub fn new(source: TypeName, target: TypeName) -> Edge {
        Edge { source, target }
    }

    pub fn source(&self) -> &TypeName {
        &self.source
    }

    pub fn target_from(&self, seen: &BTreeSet<TypeName>) -> Option<TypeName> {
        seen.contains(&self.source).then(|| self.target.clone())
    }

    pub fn reversed(&self) -> Edge {
        Edge { source: self.target.clone(), target: self.source.clone() }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} -> {}", self.source, self.target)
    }
}
