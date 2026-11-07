use std::collections::BTreeMap;

use crate::package_id::PackageId;

use super::claim::Claim;

#[derive(Debug)]
pub(super) struct Claims<'unit, 'meta>(BTreeMap<&'meta PackageId, Vec<Claim<'unit, 'meta>>>);

impl<'unit, 'meta> FromIterator<Claim<'unit, 'meta>> for Claims<'unit, 'meta> {
    fn from_iter<Found: IntoIterator<Item = Claim<'unit, 'meta>>>(found: Found) -> Claims<'unit, 'meta> {
        Claims(found.into_iter().fold(BTreeMap::new(), |mut claims, claim| {
            claims.entry(claim.package()).or_insert_with(Vec::new).push(claim);
            claims
        }))
    }
}

impl<'meta> Claims<'_, 'meta> {
    pub(super) fn owner(&self, dependent: &PackageId, required: &PackageId) -> Option<&'meta PackageId> {
        let held = self.0.get(required)?;
        let owner = held.iter().find_map(Claim::open)?;
        held.iter().find_map(|claim| claim.unit().holds(dependent)).map_or(Some(owner), |_| None)
    }
}
