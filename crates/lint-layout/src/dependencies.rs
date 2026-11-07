mod boundary;

use std::collections::{BTreeMap, BTreeSet};

use workspace_metadata::{CrateName, DependencyKeys};

use crate::line_number::LineNumber;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependencies {
    keys: BTreeMap<CrateName, BTreeSet<LineNumber>>,
    header: Option<LineNumber>,
}

impl Dependencies {
    #[must_use]
    pub fn sites(&self, keys: &DependencyKeys<'_>) -> BTreeSet<LineNumber> {
        keys.as_ref().iter().filter_map(|key| self.keys.get(*key)).flatten().copied().collect()
    }

    #[must_use]
    pub fn header(&self) -> LineNumber {
        self.header.unwrap_or(LineNumber::FIRST)
    }
}
