use std::fmt;

use lint_core::{Finding, Fix};
use workspace_metadata::CrateName;

#[derive(Debug)]
pub(super) struct ForeignDependency<'file> {
    dependency: &'file CrateName,
    owner: &'file CrateName,
}

impl<'file> ForeignDependency<'file> {
    pub(super) fn new(dependency: &'file CrateName, owner: &'file CrateName) -> ForeignDependency<'file> {
        ForeignDependency { dependency, owner }
    }
}

impl fmt::Display for ForeignDependency<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "depends on {}, in the closure of what crate {} owns", self.dependency, self.owner)
    }
}

impl Finding for ForeignDependency<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "depend on the owning crate and call its API, or have its unit declare the package in [package.metadata.port] shared",
        )
    }
}
