use std::fmt;

use lint_core::{Finding, Fix};
use workspace_metadata::CrateName;

#[derive(Debug)]
pub(super) struct StaleOwns<'file> {
    owned: &'file CrateName,
}

impl<'file> StaleOwns<'file> {
    pub(super) fn new(owned: &'file CrateName) -> StaleOwns<'file> {
        StaleOwns { owned }
    }
}

impl fmt::Display for StaleOwns<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "owns {}, which nothing in its unit depends on", self.owned)
    }
}

impl Finding for StaleOwns<'_> {
    fn fix(&self) -> Fix {
        Fix::new("drop the entry, or move it to the owns of the unit whose dependencies reach the package")
    }
}
