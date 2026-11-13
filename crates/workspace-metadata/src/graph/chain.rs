mod boundary;

use std::collections::{BTreeMap, BTreeSet};

use crate::crate_name::CrateName;
use crate::package_id::PackageId;
pub use boundary::Chained;

#[derive(Debug)]
pub(super) struct Chain<'meta> {
    below: BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>,
}

impl<'meta> From<BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>> for Chain<'meta> {
    fn from(below: BTreeMap<&'meta PackageId, BTreeSet<&'meta PackageId>>) -> Chain<'meta> {
        Chain { below }
    }
}

impl<'meta> Chain<'meta> {
    pub(super) fn placed(&self, named: impl Fn(&PackageId) -> Option<&'meta CrateName>) -> Vec<Chained<'meta>> {
        let mut ordered: Vec<(&&'meta PackageId, &BTreeSet<&'meta PackageId>)> = self.below.iter().collect();
        ordered.sort_by_key(|(_, below)| below.len());
        let depths =
            ordered.iter().fold(BTreeMap::new(), |mut depths: BTreeMap<&'meta PackageId, usize>, (id, below)| {
                let deepest = below.iter().filter_map(|other| depths.get(other)).max().copied().unwrap_or_default();
                depths.insert(**id, deepest + 1);
                depths
            });
        self.below
            .iter()
            .filter_map(|(id, below)| {
                let above = self.below.values().filter(|others| others.contains(id)).count();
                Some(Chained::new(named(id)?, depths.get(id).copied().unwrap_or_default(), below.len(), above))
            })
            .collect()
    }
}
