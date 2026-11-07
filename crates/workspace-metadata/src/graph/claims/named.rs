use std::collections::{BTreeMap, BTreeSet};

use newtype::Newtype;

use crate::package_id::PackageId;

use super::claim::Claim;

#[derive(Newtype, Debug)]
pub(super) struct Named<'unit, 'meta>(BTreeMap<&'meta PackageId, Vec<Claim<'unit, 'meta>>>);

impl<'meta> Named<'_, 'meta> {
    pub(super) fn owners(&self, dependent: &PackageId, required: &PackageId) -> Option<BTreeSet<&'meta PackageId>> {
        self.0.get(required).map(|named| {
            let held: BTreeSet<&PackageId> =
                named.iter().filter(|claim| claim.unit().holds(dependent).is_some()).filter_map(Claim::open).collect();
            named.iter().filter_map(Claim::open).filter(|owner| held.get(owner).is_none()).collect()
        })
    }
}
