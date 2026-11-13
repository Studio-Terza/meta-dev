use lint_core::Site;
use lint_layout::Name;

use crate::consumers::Consumers;

#[derive(Clone, Debug)]
pub(crate) struct Declaration<'file> {
    name: Name,
    provided: Vec<Name>,
    site: Site<'file>,
}

impl<'file> Declaration<'file> {
    pub(crate) fn new(name: Name, provided: Vec<Name>, site: Site<'file>) -> Declaration<'file> {
        Declaration { name, provided, site }
    }

    pub(crate) fn unconsumed(self, consumers: &Consumers) -> Option<Declaration<'file>> {
        consumers.consuming(&self.name, &self.provided).is_none().then_some(self)
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
