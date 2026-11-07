use std::fmt;

use lint_core::{Finding, Fix};
use lint_placement::CrateName;

#[derive(Debug)]
pub(super) struct Undeclared<'file> {
    reached: &'file workspace_metadata::CrateName,
    owner: &'file CrateName,
}

impl<'file> Undeclared<'file> {
    pub(super) fn new(reached: &'file workspace_metadata::CrateName, owner: &'file CrateName) -> Undeclared<'file> {
        Undeclared { reached, owner }
    }
}

impl fmt::Display for Undeclared<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} is reached only through packages crate {} owns, which does not declare it",
            self.reached, self.owner
        )
    }
}

impl Finding for Undeclared<'_> {
    fn fix(&self) -> Fix {
        Fix::new("add it to the owning crate's [package.metadata.port] owns so the family stays complete")
    }
}
