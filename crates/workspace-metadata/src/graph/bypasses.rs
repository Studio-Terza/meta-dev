mod dependency_keys;

use std::collections::{BTreeMap, BTreeSet, btree_map};

use crate::crate_name::CrateName;
pub use dependency_keys::DependencyKeys;

type Required<'meta> = BTreeMap<&'meta CrateName, Bypass<'meta>>;

#[derive(Debug, Default)]
pub struct Bypass<'meta> {
    owners: BTreeSet<&'meta CrateName>,
    keys: DependencyKeys<'meta>,
}

impl<'meta> Bypass<'meta> {
    pub(super) fn new(owners: BTreeSet<&'meta CrateName>, keys: DependencyKeys<'meta>) -> Bypass<'meta> {
        Bypass { owners, keys }
    }

    #[must_use]
    pub fn owners(&self) -> &BTreeSet<&'meta CrateName> {
        &self.owners
    }

    #[must_use]
    pub fn keys(&self) -> &DependencyKeys<'meta> {
        &self.keys
    }
}

#[derive(Debug)]
pub struct Bypasses<'meta>(BTreeMap<&'meta CrateName, Required<'meta>>);

impl<'meta> FromIterator<(&'meta CrateName, &'meta CrateName, Bypass<'meta>)> for Bypasses<'meta> {
    fn from_iter<Found: IntoIterator<Item = (&'meta CrateName, &'meta CrateName, Bypass<'meta>)>>(
        found: Found,
    ) -> Bypasses<'meta> {
        Bypasses(found.into_iter().fold(BTreeMap::new(), |mut gathered, (dependent, required, bypass)| {
            let merged = gathered.entry(dependent).or_insert_with(BTreeMap::new).entry(required).or_default();
            merged.owners.extend(bypass.owners);
            merged.keys.extend(bypass.keys.as_ref().iter().copied());
            gathered
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
