use lint_core::Site;
use lint_layout::Name;

use crate::via::Via;

#[derive(Clone, Debug)]
pub struct Dependency<'file> {
    name: Name,
    qualifier: Option<Name>,
    site: Site<'file>,
    via: Via,
}

impl<'file> Dependency<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, via: Via) -> Dependency<'file> {
        Dependency { name, qualifier: None, site, via }
    }

    pub(crate) fn qualified(mut self, qualifier: Option<Name>) -> Dependency<'file> {
        self.qualifier = qualifier;
        self
    }

    #[must_use]
    pub fn qualifier(&self) -> Option<&Name> {
        self.qualifier.as_ref()
    }
}

impl AsRef<Name> for Dependency<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for Dependency<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl AsRef<Via> for Dependency<'_> {
    fn as_ref(&self) -> &Via {
        &self.via
    }
}
