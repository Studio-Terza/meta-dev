use lint_core::Site;
use lint_layout::Name;

use crate::via::Via;

#[derive(Clone, Debug)]
pub struct Dependency<'file> {
    name: Name,
    written: Vec<Name>,
    site: Site<'file>,
    via: Via,
}

impl<'file> Dependency<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, via: Via) -> Dependency<'file> {
        Dependency { name, written: Vec::new(), site, via }
    }

    pub(crate) fn qualified(mut self, written: Vec<Name>) -> Dependency<'file> {
        self.written = written;
        self
    }

    #[must_use]
    pub fn qualifier(&self) -> Option<&Name> {
        self.written.first()
    }

    #[must_use]
    pub fn written(&self) -> &[Name] {
        &self.written
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
