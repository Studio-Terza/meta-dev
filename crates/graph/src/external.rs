use algebra::{Monoid, Semigroup};
use lint_layout::{Name, PackageName};

use crate::item_path::ItemPath;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct External {
    package: Option<PackageName>,
    path: ItemPath,
}

impl From<ItemPath> for External {
    fn from(path: ItemPath) -> External {
        External { package: None, path }
    }
}

impl Semigroup for External {
    fn combine(self, other: External) -> External {
        External { package: self.package.or(other.package), path: self.path.combine(other.path) }
    }
}

impl Monoid for External {}

impl External {
    #[must_use]
    pub fn new(package: Option<PackageName>, path: ItemPath) -> External {
        External { package, path }
    }

    #[must_use]
    pub fn package(&self) -> Option<&PackageName> {
        self.package.as_ref()
    }

    #[must_use]
    pub fn path(&self) -> &ItemPath {
        &self.path
    }

    #[must_use]
    pub fn below_root(&self) -> &[Name] {
        self.path.as_ref().get(1..).unwrap_or_default()
    }
}
