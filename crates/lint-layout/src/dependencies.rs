use std::collections::BTreeMap;

use crate::name::Name;
use crate::package_name::PackageName;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Dependencies(BTreeMap<Name, PackageName>);

impl FromIterator<(Name, PackageName)> for Dependencies {
    fn from_iter<Declared: IntoIterator<Item = (Name, PackageName)>>(declared: Declared) -> Dependencies {
        Dependencies(declared.into_iter().collect())
    }
}

impl Dependencies {
    #[must_use]
    pub fn package(&self, krate: &Name) -> Option<&PackageName> {
        self.0.get(krate)
    }
}
