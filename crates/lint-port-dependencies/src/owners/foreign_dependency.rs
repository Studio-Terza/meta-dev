use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug)]
pub(super) struct ForeignDependency<'file> {
    dependency: &'file Name,
    owner: &'file CrateName,
}

impl<'file> ForeignDependency<'file> {
    pub(super) fn new(dependency: &'file Name, owner: &'file CrateName) -> ForeignDependency<'file> {
        ForeignDependency { dependency, owner }
    }
}

impl fmt::Display for ForeignDependency<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "depends on {}, which crate {} owns", self.dependency, self.owner)
    }
}

impl Finding for ForeignDependency<'_> {
    fn fix(&self) -> Fix {
        Fix::new("depend on the owning crate and call its API instead of the owned dependency")
    }
}
