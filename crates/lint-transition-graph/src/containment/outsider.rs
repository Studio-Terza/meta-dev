use std::fmt;

use lint_core::Site;
use lint_layout::Name;

use super::member::Member;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Outsider<'found, 'file> {
    holder: Member<'found>,
    name: Name,
    site: Site<'file>,
}

impl<'found, 'file> Outsider<'found, 'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, holder: Member<'found>) -> Outsider<'found, 'file> {
        Outsider { holder, name, site }
    }

    pub(crate) fn holder(&self) -> Member<'found> {
        self.holder
    }
}

impl fmt::Display for Outsider<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "impl for {} at {}", self.name, self.site)
    }
}
