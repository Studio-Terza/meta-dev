use std::fmt;

use lint_core::{Finding, Fix};
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Debug)]
pub(super) struct CoOwned<'file> {
    dependency: &'file Name,
    owner: &'file CrateName,
}

impl<'file> CoOwned<'file> {
    pub(super) fn new(dependency: &'file Name, owner: &'file CrateName) -> CoOwned<'file> {
        CoOwned { dependency, owner }
    }
}

impl fmt::Display for CoOwned<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "owns {}, which crate {} owns too", self.dependency, self.owner)
    }
}

impl Finding for CoOwned<'_> {
    fn fix(&self) -> Fix {
        Fix::new("leave the dependency to its one owner and call that crate's API; a dependency has one owner")
    }
}
