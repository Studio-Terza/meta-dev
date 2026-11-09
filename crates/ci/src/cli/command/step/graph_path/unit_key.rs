use std::fmt;

use super::package::Package;
use super::unit_role::UnitRole;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct UnitKey {
    package: Package,
    role: UnitRole,
}

impl UnitKey {
    pub(super) fn new(package: Package, role: UnitRole) -> UnitKey {
        UnitKey { package, role }
    }

    pub(super) fn package(&self) -> &Package {
        &self.package
    }

    pub(super) fn role(&self) -> &UnitRole {
        &self.role
    }

    pub(super) fn sibling(&self, role: UnitRole) -> UnitKey {
        UnitKey { package: self.package.clone(), role }
    }
}

impl fmt::Display for UnitKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{}", self.package, self.role)
    }
}
