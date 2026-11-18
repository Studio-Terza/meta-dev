use std::collections::BTreeSet;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::role::Role;

use super::expanders::Expanders;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Shared<'table>(&'table CrateName);

impl AsRef<CrateName> for Shared<'_> {
    fn as_ref(&self) -> &CrateName {
        self.0
    }
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

    pub fn shares(&self, package: &CrateName) -> Option<Shared<'_>> {
        self.shared.get(package).map(Shared)
    }

    pub fn dependents(&self) -> Option<Role> {
        self.dependents
    }

    pub fn expanders(&self) -> Option<&Expanders> {
        self.expanders.as_ref()
    }
}
