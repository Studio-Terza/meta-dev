mod boundary;

use std::collections::{BTreeMap, BTreeSet};

use crate::line_number::LineNumber;
use crate::name::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DependencySites {
    keys: BTreeMap<Name, BTreeSet<LineNumber>>,
    header: Option<LineNumber>,
}

impl DependencySites {
    #[must_use]
    pub fn header(&self) -> LineNumber {
        self.header.unwrap_or(LineNumber::FIRST)
    }
}
