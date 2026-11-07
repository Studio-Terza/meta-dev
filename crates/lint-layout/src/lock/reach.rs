use std::collections::BTreeSet;

use super::locked::Locked;

#[derive(Debug)]
pub(super) struct Reach<'lock> {
    seen: BTreeSet<&'lock Locked>,
    frontier: BTreeSet<&'lock Locked>,
}

impl<'lock> Reach<'lock> {
    pub(super) fn grown(&self, lock: &'lock [Locked]) -> Option<Reach<'lock>> {
        let required: BTreeSet<&Locked> = self.frontier.iter().flat_map(|locked| locked.requirements(lock)).collect();
        let frontier: BTreeSet<&Locked> = required.difference(&self.seen).copied().collect();
        let seen = self.seen.union(&frontier).copied().collect();
        frontier.first().copied().map(|_| Reach { seen, frontier })
    }
}

impl<'lock> From<&'lock Locked> for Reach<'lock> {
    fn from(root: &'lock Locked) -> Reach<'lock> {
        Reach { seen: BTreeSet::from([root]), frontier: BTreeSet::from([root]) }
    }
}

impl<'lock> From<Reach<'lock>> for BTreeSet<&'lock Locked> {
    fn from(reach: Reach<'lock>) -> BTreeSet<&'lock Locked> {
        reach.seen
    }
}
