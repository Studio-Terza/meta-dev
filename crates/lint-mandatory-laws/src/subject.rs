use lint_core::SourceFile;
use lint_layout::Name;

use crate::package::Package;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Subject {
    package: Package,
    name: Name,
}

impl Subject {
    pub(crate) fn new(file: &SourceFile, name: Name) -> Subject {
        Subject { package: Package::containing(file), name }
    }

    pub(crate) fn package(&self) -> &Package {
        &self.package
    }

    pub(crate) fn owner(&self) -> &Name {
        &self.name
    }
}
