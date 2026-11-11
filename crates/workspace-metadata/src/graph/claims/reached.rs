use std::collections::{BTreeMap, BTreeSet};

use newtype::Newtype;

use crate::package_id::PackageId;

use super::claim::Claim;

#[derive(Newtype, Debug)]
pub(super) struct Reached<'unit, 'meta>(BTreeMap<&'meta PackageId, Vec<Claim<'unit, 'meta>>>);

impl<'meta> Reached<'_, 'meta> {
    pub(super) fn owners(&self, dependent: &PackageId, required: &PackageId) -> BTreeSet<&'meta PackageId> {
        let held = self.0.get(required).map_or(&[][..], Vec::as_slice);
        held.iter()
            .find_map(|claim| claim.unit().holds(dependent))
            .map_or_else(|| held.iter().filter_map(Claim::open).collect(), |_| BTreeSet::new())
    }
}
