use lint_core::Site;
use lint_layout::Name;
use lint_placement::CrateName;

#[derive(Clone, Debug)]
pub struct TraitFact {
    krate: CrateName,
    name: Name,
    site: Site,
}

impl TraitFact {
    pub(crate) fn new(krate: CrateName, name: Name, site: Site) -> TraitFact {
        TraitFact { krate, name, site }
    }
}

impl AsRef<CrateName> for TraitFact {
    fn as_ref(&self) -> &CrateName {
        &self.krate
    }
}

impl AsRef<Name> for TraitFact {
    fn as_ref(&self) -> &Name {
        &self.name
    }
}

impl AsRef<Site> for TraitFact {
    fn as_ref(&self) -> &Site {
        &self.site
    }
}
