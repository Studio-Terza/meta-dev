use std::fmt;
use std::path::PathBuf;

use lint_core::Site;
use lint_layout::Name;
use lint_placement::{CrateName, ModulePath};

use crate::claim::Claim;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TestId {
    package: CrateName,
    target: PathBuf,
    module: ModulePath,
    name: Name,
}

impl TestId {
    pub(crate) fn new(package: CrateName, target: PathBuf, module: ModulePath, name: Name) -> TestId {
        TestId { package, target, module, name }
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
    claims: Vec<Claim>,
}

impl<'file> Test<'file> {
    pub(crate) fn new(id: TestId, site: Site<'file>, claims: Vec<Claim>) -> Test<'file> {
        Test { id, site, claims }
    }

    #[must_use]
    pub fn package(&self) -> &CrateName {
        &self.id.package
    }

    #[must_use]
    pub fn claims(&self) -> &[Claim] {
        &self.claims
    }
}

impl<'file> AsRef<Site<'file>> for Test<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Test<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.id)
    }
}
