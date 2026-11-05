use std::fmt;

use lint_core::{Finding, Fix, Site};
use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unresolved<'file> {
    site: Site<'file>,
    name: Name,
}

impl<'file> Unresolved<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name) -> Unresolved<'file> {
        Unresolved { site, name }
    }
}

impl<'file> AsRef<Site<'file>> for Unresolved<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Unresolved<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "law {}! checks a closure the lint cannot resolve to an operation", self.name)
    }
}

impl Finding for Unresolved<'_> {
    fn fix(&self) -> Fix {
        Fix::new("pass the operation path (Type::function), or a closure that calls it on its parameters")
    }
}
