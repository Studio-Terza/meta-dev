use std::collections::BTreeSet;

use serde::Deserialize;

use crate::crate_name::CrateName;

#[derive(Clone, Debug, Deserialize)]
pub struct PortTable {
    #[serde(default)]
    owns: BTreeSet<CrateName>,
    #[serde(default)]
    shared: BTreeSet<CrateName>,
}

impl PortTable {
    pub fn owns(&self) -> &BTreeSet<CrateName> {
        &self.owns
    }

    pub fn shares(&self, package: &CrateName) -> Option<&CrateName> {
        self.shared.get(package)
    }
}
