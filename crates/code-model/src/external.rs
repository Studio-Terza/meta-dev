use lint_layout::{Name, PackageName};

use crate::item_path::ItemPath;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct External {
    package: Option<PackageName>,
    path: ItemPath,
}

impl External {
    #[must_use]
    pub fn new(package: Option<PackageName>, path: ItemPath) -> External {
        External { package, path }
    }

    #[must_use]
    pub fn joined(&self, segment: &Name) -> External {
        External { package: self.package.clone(), path: self.path.joined(segment) }
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
    pub fn item(&self) -> ItemPath {
        self.path.as_ref().iter().skip(1).cloned().collect()
    }
}
