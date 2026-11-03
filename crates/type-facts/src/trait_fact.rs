use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

use crate::method::Method;

#[derive(Clone, Debug)]
pub struct TraitFact<'file> {
    krate: &'file CrateName,
    name: Name,
    site: Site<'file>,
    methods: Vec<Method>,
}

impl<'file> TraitFact<'file> {
    pub(crate) fn new(
        krate: &'file CrateName,
        name: Name,
        site: Site<'file>,
        methods: Vec<Method>,
    ) -> TraitFact<'file> {
        TraitFact { krate, name, site, methods }
    }
}

impl AsRef<[Method]> for TraitFact<'_> {
    fn as_ref(&self) -> &[Method] {
        &self.methods
    }
}

impl AsRef<CrateName> for TraitFact<'_> {
    fn as_ref(&self) -> &CrateName {
        self.krate
    }
}

impl AsRef<Name> for TraitFact<'_> {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl<'file> AsRef<Site<'file>> for TraitFact<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
