use std::collections::BTreeSet;
use std::fmt;
use std::iter;

use lint_core::{Finding, Fix};
use workspace_metadata::CrateName;

#[derive(Debug)]
pub(super) struct ForeignDependency<'file> {
    dependency: &'file CrateName,
    owners: BTreeSet<&'file CrateName>,
}

impl<'file> ForeignDependency<'file> {
    pub(super) fn new(dependency: &'file CrateName, owners: BTreeSet<&'file CrateName>) -> ForeignDependency<'file> {
        ForeignDependency { dependency, owners }
    }
}

impl fmt::Display for ForeignDependency<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "depends on {}", self.dependency)?;
        iter::once(", in the closure of")
            .chain(iter::repeat(", and of"))
            .zip(&self.owners)
            .try_for_each(|(lead, owner)| write!(formatter, "{lead} what crate {owner} owns"))
    }
}

impl Finding for ForeignDependency<'_> {
    fn fix(&self) -> Fix {
        Fix::new(
            "depend on the owning crate and call its API, or have its unit declare the package in [package.metadata.port] shared",
        )
    }
}
