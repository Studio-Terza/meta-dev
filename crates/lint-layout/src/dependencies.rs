use std::collections::BTreeMap;

use crate::crate_key::CrateKey;
use crate::name::Name;
use crate::package_name::PackageName;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependencies(BTreeMap<CrateKey, PackageName>);

impl FromIterator<(CrateKey, PackageName)> for Dependencies {
    fn from_iter<Declared: IntoIterator<Item = (CrateKey, PackageName)>>(declared: Declared) -> Dependencies {
        Dependencies(declared.into_iter().collect())
    }
}

impl Dependencies {
    #[must_use]
    pub fn package(&self, krate: &Name) -> Option<&PackageName> {
        self.0.get(&CrateKey::new(krate.words()))
    }
}
