use std::collections::BTreeSet;

use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Debug)]
pub(crate) struct Declaration<'file> {
    name: Name,
    site: Site<'file>,
}

impl<'file> Declaration<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>) -> Declaration<'file> {
        Declaration { name, site }
    }

    pub(crate) fn unconsumed(self, consumers: &BTreeSet<Name>) -> Option<Declaration<'file>> {
        (!consumers.contains(&self.name)).then_some(self)
    }
}

impl AsRef<Name> for Declaration<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for Declaration<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
