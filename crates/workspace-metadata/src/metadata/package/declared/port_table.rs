use std::collections::BTreeSet;

use serde::Deserialize;

use crate::crate_name::CrateName;
use crate::role::Role;

use super::expanders::Expanders;

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(transparent)]
pub struct Sharing(BTreeSet<CrateName>);

impl<'sharing> IntoIterator for &'sharing Sharing {
    type Item = &'sharing CrateName;
    type IntoIter = std::collections::btree_set::Iter<'sharing, CrateName>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortTable {
    #[serde(default)]
    owns: BTreeSet<CrateName>,
    #[serde(default)]
    shared: Sharing,
    dependents: Option<Role>,
    expanders: Option<Expanders>,
}

impl PortTable {
    pub fn owns(&self) -> &BTreeSet<CrateName> {
        &self.owns
    }

    pub fn shared(&self) -> &Sharing {
        &self.shared
    }

    pub fn dependents(&self) -> Option<Role> {
        self.dependents
    }

    pub fn expanders(&self) -> Option<&Expanders> {
        self.expanders.as_ref()
    }
}
