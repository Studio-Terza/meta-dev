use std::collections::BTreeSet;

use lint_core::Site;
use lint_layout::Name;

#[derive(Clone, Debug)]
pub(crate) struct Declaration {
    name: Name,
    site: Site,
}

impl Declaration {
    pub(crate) fn new(name: Name, site: Site) -> Declaration {
        Declaration { name, site }
    }

    pub(crate) fn unconsumed(self, consumers: &BTreeSet<Name>) -> Option<Declaration> {
        (!consumers.contains(&self.name)).then_some(self)
    }
}

impl AsRef<Name> for Declaration {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<Site> for Declaration {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
