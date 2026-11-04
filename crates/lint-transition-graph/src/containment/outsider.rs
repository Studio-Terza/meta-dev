use std::fmt;

use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Outsider<'found, 'file> {
    name: &'found Name,
    site: Site<'file>,
}

impl<'found, 'file> Outsider<'found, 'file> {
    pub(crate) fn new(name: &'found Name, site: Site<'file>) -> Outsider<'found, 'file> {
        Outsider { name, site }
    }
}

impl fmt::Display for Outsider<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "impl for {} at {}", self.name, self.site)
    }
}
