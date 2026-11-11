use std::collections::BTreeSet;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::role::Role;

use super::expanders::Expanders;

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

    pub fn shares(&self, package: &CrateName) -> Option<&CrateName> {
        self.shared.get(package)
    }

    pub fn dependents(&self) -> Option<Role> {
        self.dependents
    }

    pub fn expanders(&self) -> Option<&Expanders> {
        self.expanders.as_ref()
    }
}
