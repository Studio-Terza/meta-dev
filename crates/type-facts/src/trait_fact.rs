use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Debug)]
pub struct TraitFact<'file> {
    krate: CrateName,
    name: Name,
    site: Site<'file>,
}

impl<'file> TraitFact<'file> {
    pub(crate) fn new(krate: CrateName, name: Name, site: Site<'file>) -> TraitFact<'file> {
        TraitFact { krate, name, site }
    }
}

impl AsRef<CrateName> for TraitFact<'_> {
    fn as_ref(&self) -> &CrateName {
        &self.krate
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
