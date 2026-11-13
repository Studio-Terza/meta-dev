use std::fmt;

use crate::crate_name::CrateName;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Chained<'meta> {
    depth: usize,
    below: usize,
    above: usize,
    name: &'meta CrateName,
}

impl<'meta> Chained<'meta> {
    pub(crate) fn new(name: &'meta CrateName, depth: usize, below: usize, above: usize) -> Chained<'meta> {
        Chained { depth, below, above, name }
    }

    #[must_use]
    pub fn name(&self) -> &'meta CrateName {
        self.name
    }
}

impl fmt::Display for Chained<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "depth {} on the compile chain, {} workspace crates below it, {} above it",
            self.depth, self.below, self.above
        )
    }
}
