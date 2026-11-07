use std::collections::{BTreeMap, BTreeSet, btree_map};

use crate::crate_name::CrateName;

type Required<'meta> = BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>>;

#[derive(Debug)]
pub struct Bypasses<'meta>(BTreeMap<&'meta CrateName, Required<'meta>>);

impl<'meta> FromIterator<(&'meta CrateName, &'meta CrateName, BTreeSet<&'meta CrateName>)> for Bypasses<'meta> {
    fn from_iter<Found: IntoIterator<Item = (&'meta CrateName, &'meta CrateName, BTreeSet<&'meta CrateName>)>>(
        found: Found,
    ) -> Bypasses<'meta> {
        Bypasses(found.into_iter().fold(BTreeMap::new(), |mut found, (dependent, required, owners)| {
            found
                .entry(dependent)
                .or_insert_with(BTreeMap::new)
                .entry(required)
                .or_insert_with(BTreeSet::new)
                .extend(owners);
            found
        }))
    }
}

impl<'meta> IntoIterator for Bypasses<'meta> {
    type Item = (&'meta CrateName, Required<'meta>);
    type IntoIter = btree_map::IntoIter<&'meta CrateName, Required<'meta>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
