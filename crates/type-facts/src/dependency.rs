use lint_core::Site;
use lint_layout::Name;

use crate::held::Held;
use crate::via::Via;

#[derive(Clone, Debug)]
pub struct Dependency<'file> {
    name: Name,
    written: Vec<Name>,
    site: Site<'file>,
    via: Via,
    held: Held,
}

impl<'file> Dependency<'file> {
    pub(crate) fn new(name: Name, site: Site<'file>, via: Via, held: Held) -> Dependency<'file> {
        Dependency { name, written: Vec::new(), site, via, held }
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

    #[must_use]
    pub fn held(&self) -> Held {
        self.held
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
