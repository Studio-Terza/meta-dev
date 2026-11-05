use std::fmt;
use std::path::PathBuf;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::{CrateName, ModulePath};

use crate::check::Check;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TestId {
    package: CrateName,
    target: PathBuf,
    module: ModulePath,
    name: Name,
}

impl TestId {
    pub(crate) fn new(package: CrateName, target: PathBuf, module: ModulePath, name: Name) -> TestId {
        TestId { package, target, module, name }
    }

    #[must_use]
    pub fn package(&self) -> &CrateName {
        &self.package
    }
}

impl fmt::Display for TestId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({})", self.name, self.target.display())?;
        self.module.segments().iter().try_for_each(|segment| write!(formatter, " in {segment}"))?;
        write!(formatter, " of {}", self.package)
    }
}

#[derive(Debug)]
pub struct Test<'file> {
    id: TestId,
    site: Site<'file>,
    checks: Vec<Check>,
}

impl<'file> Test<'file> {
    pub(crate) fn new(id: TestId, site: Site<'file>, checks: Vec<Check>) -> Test<'file> {
        Test { id, site, checks }
    }

    #[must_use]
    pub fn identity(&self) -> &TestId {
        &self.id
    }

    #[must_use]
    pub fn checks(&self) -> &[Check] {
        &self.checks
    }
}

impl<'file> AsRef<Site<'file>> for Test<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}
