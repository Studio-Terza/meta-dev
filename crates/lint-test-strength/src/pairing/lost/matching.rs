use std::collections::{BTreeMap, BTreeSet};

use newtype::{Arbitrary, Newtype};
use test_facts::Check;

use super::Lost;

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Slot(usize);

#[derive(Debug)]
pub(crate) struct Matching<'after> {
    after: &'after [Check],
    holders: BTreeMap<Slot, Check>,
}

impl<'after> Matching<'after> {
    pub(crate) fn unplaced<'tests, 'file>(
        before: Vec<Lost<'tests, 'file>>,
        after: &'after [Check],
    ) -> Vec<Lost<'tests, 'file>> {
        let mut matching = Matching { after, holders: BTreeMap::new() };
        before.into_iter().filter(|lost| matching.placed(lost.as_ref(), &mut BTreeSet::new()).is_none()).collect()
    }

    fn placed(&mut self, check: &Check, tried: &mut BTreeSet<Slot>) -> Option<Slot> {
        self.candidates(check).into_iter().find_map(|slot| self.taken(slot, check, tried))
    }

    fn candidates(&self, check: &Check) -> Vec<Slot> {
        self.after
            .iter()
            .enumerate()
            .filter_map(|(slot, stronger)| stronger.covers(check).map(|_| Slot::new(slot)))
            .collect()
    }

    fn taken(&mut self, slot: Slot, check: &Check, tried: &mut BTreeSet<Slot>) -> Option<Slot> {
        let fresh = Some(slot).filter(|slot| tried.insert(*slot))?;
        let freed = self.freed(fresh, self.holders.get(&fresh).cloned(), tried)?;
        self.holders.insert(freed, check.clone());
        Some(freed)
    }

    fn freed(&mut self, slot: Slot, holder: Option<Check>, tried: &mut BTreeSet<Slot>) -> Option<Slot> {
        match holder {
            None => Some(slot),
            Some(held) => self.placed(&held, tried).and(Some(slot)),
        }
    }
}
