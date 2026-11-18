use std::collections::BTreeSet;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::role::Role;

use super::expanders::Expanders;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sharing {
    Shared,
    Unshared,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortTable {
    #[serde(default)]
    owns: BTreeSet<CrateName>,
    #[serde(default)]
    shared: BTreeSet<CrateName>,
    dependents: Option<Role>,
    expanders: Option<Expanders>,
}

impl PortTable {
    pub fn owns(&self) -> &BTreeSet<CrateName> {
        &self.owns
    }

    pub fn sharing(&self, package: &CrateName) -> Sharing {
        self.shared.get(package).map_or(Sharing::Unshared, |_| Sharing::Shared)
    }

    pub fn dependents(&self) -> Option<Role> {
        self.dependents
    }

    pub fn expanders(&self) -> Option<&Expanders> {
        self.expanders.as_ref()
    }
}
