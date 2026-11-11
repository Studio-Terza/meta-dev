use std::collections::{BTreeMap, BTreeSet, btree_map};

use crate::crate_name::CrateName;

#[derive(Debug)]
pub struct Stale<'meta>(BTreeMap<&'meta CrateName, BTreeSet<&'meta CrateName>>);

impl<'meta> FromIterator<(&'meta CrateName, &'meta CrateName)> for Stale<'meta> {
    fn from_iter<Found: IntoIterator<Item = (&'meta CrateName, &'meta CrateName)>>(found: Found) -> Stale<'meta> {
        Stale(found.into_iter().fold(BTreeMap::new(), |mut gathered, (declarer, owned)| {
            gathered.entry(declarer).or_insert_with(BTreeSet::new).insert(owned);
            gathered
        }))
    }
}

impl<'meta> FromIterator<Stale<'meta>> for Stale<'meta> {
    fn from_iter<Found: IntoIterator<Item = Stale<'meta>>>(found: Found) -> Stale<'meta> {
        Stale(found.into_iter().flat_map(|stale| stale.0).fold(BTreeMap::new(), |mut merged, (declarer, owned)| {
            merged.entry(declarer).or_insert_with(BTreeSet::new).extend(owned);
            merged
        }))
    }
}

impl<'meta> IntoIterator for Stale<'meta> {
    type Item = (&'meta CrateName, BTreeSet<&'meta CrateName>);
    type IntoIter = btree_map::IntoIter<&'meta CrateName, BTreeSet<&'meta CrateName>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
