mod lost;

use std::collections::{BTreeMap, BTreeSet};

use test_facts::{Check, Test, TestId, Tests};

pub(crate) use lost::Lost;

#[derive(Debug)]
pub(crate) struct Pairing<'tests, 'file> {
    before: BTreeMap<&'tests TestId, &'tests Test<'file>>,
    after: BTreeMap<&'tests TestId, &'tests Test<'file>>,
}

impl<'tests, 'file> Pairing<'tests, 'file> {
    pub(crate) fn across(before: &'tests Tests<'file>, after: &'tests Tests<'file>) -> Pairing<'tests, 'file> {
        Pairing {
            before: before.all().map(|test| (test.identity(), test)).collect(),
            after: after.all().map(|test| (test.identity(), test)).collect(),
        }
    }

    pub(crate) fn lost(&self) -> Vec<Lost<'tests, 'file>> {
        let before: BTreeSet<&TestId> = self.before.keys().copied().collect();
        let after: BTreeSet<&TestId> = self.after.keys().copied().collect();
        let kept = before.intersection(&after).filter_map(|id| self.before.get(id).zip(self.after.get(id)));
        let weakened = kept.flat_map(|(was, now)| Lost::unmatched(&[*was], now.checks()));
        let gone: Vec<&Test<'file>> = before.difference(&after).filter_map(|id| self.before.get(id).copied()).collect();
        let added: Vec<&Test<'file>> = after.difference(&before).filter_map(|id| self.after.get(id).copied()).collect();
        weakened
            .chain(
                Departure::grouped(&gone, &added)
                    .iter()
                    .flat_map(|departure| Lost::unmatched(&departure.gone, &departure.landed)),
            )
            .collect()
    }
}

#[derive(Debug)]
struct Departure<'tests, 'file> {
    gone: Vec<&'tests Test<'file>>,
    landed: Vec<Check>,
}

impl<'tests, 'file> Departure<'tests, 'file> {
    fn grouped(gone: &[&'tests Test<'file>], added: &[&'tests Test<'file>]) -> Vec<Departure<'tests, 'file>> {
        let packages: BTreeSet<_> = gone.iter().map(|test| test.identity().package()).collect();
        packages
            .into_iter()
            .map(|package| Departure {
                gone: gone.iter().copied().filter(|test| test.identity().package() == package).collect(),
                landed: added
                    .iter()
                    .filter(|test| test.identity().package() == package)
                    .flat_map(|test| test.checks().iter().cloned())
                    .collect(),
            })
            .collect()
    }
}
