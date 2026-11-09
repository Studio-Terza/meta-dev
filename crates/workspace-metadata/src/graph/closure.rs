use std::collections::BTreeSet;
use std::iter;

use newtype::Newtype;

use crate::package_id::PackageId;

use super::requires::Requires;

#[derive(Newtype, Debug)]
pub(super) struct Closure<'meta>(BTreeSet<&'meta PackageId>);

impl<'meta> Closure<'meta> {
    pub(super) fn reached(requires: &Requires<'meta>, start: BTreeSet<&'meta PackageId>) -> Closure<'meta> {
        Closure(
            iter::successors(Some((start.clone(), start)), |(seen, frontier)| {
                let reached: BTreeSet<&PackageId> = frontier.iter().flat_map(|id| requires.direct(id)).collect();
                let next: BTreeSet<&PackageId> = reached.difference(seen).copied().collect();
                Some(next)
                    .filter(|unseen| unseen.first().is_some())
                    .map(|unseen| (seen.union(&unseen).copied().collect(), unseen))
            })
            .last()
            .map(|(seen, _)| seen)
            .unwrap_or_default(),
        )
    }
}
