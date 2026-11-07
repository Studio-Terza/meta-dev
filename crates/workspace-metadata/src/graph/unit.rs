use std::collections::{BTreeMap, BTreeSet};

use crate::crate_name::CrateName;
use crate::metadata::PortTable;
use crate::package_id::PackageId;

#[derive(Debug)]
pub(super) struct Unit<'meta> {
    members: BTreeSet<&'meta PackageId>,
    declarations: BTreeMap<&'meta PackageId, &'meta PortTable>,
}

impl<'meta> Unit<'meta> {
    pub(super) fn new(
        members: BTreeSet<&'meta PackageId>,
        declarations: BTreeMap<&'meta PackageId, &'meta PortTable>,
    ) -> Unit<'meta> {
        Unit { members, declarations }
    }

    pub(super) fn declarations(&self) -> &BTreeMap<&'meta PackageId, &'meta PortTable> {
        &self.declarations
    }

    pub(super) fn holds(&self, package: &PackageId) -> Option<&'meta PackageId> {
        self.members.get(package).copied()
    }

    pub(super) fn shares(&self, package: &CrateName) -> Option<&'meta CrateName> {
        self.declarations.values().find_map(|declared| declared.shares(package))
    }
}
