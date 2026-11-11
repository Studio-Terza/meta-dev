mod critical;
mod reach;

use std::collections::BTreeMap;
use std::iter;

use super::crates::Crates;
use super::timed_unit::TimedUnit;
use super::unit_key::UnitKey;
use super::weighed::Weighed;
use crate::cli::command::step::seconds::Seconds;
pub(super) use critical::Critical;
use reach::Reach;

#[derive(Debug)]
pub(super) struct UnitDag<'crates> {
    crates: &'crates Crates,
    weights: BTreeMap<UnitKey, Seconds>,
    work: Seconds,
}

impl<'crates> UnitDag<'crates> {
    pub(super) fn weighed(crates: &'crates Crates, units: Vec<TimedUnit>) -> UnitDag<'crates> {
        let weighed: Vec<Weighed> = units.into_iter().map(|unit| unit.weighed(crates)).collect();
        let work = weighed.iter().map(|unit| *AsRef::<Seconds>::as_ref(unit)).sum();
        let weights = weighed.into_iter().fold(BTreeMap::new(), |weights, unit| unit.heaviest(weights));
        UnitDag { crates, weights, work }
    }

    pub(super) fn work(&self) -> Seconds {
        self.work
    }

    pub(super) fn critical(&self) -> Critical {
        let mut reached = BTreeMap::new();
        let longest = self
            .weights
            .keys()
            .map(|key| Weighed::new(self.reached(key, &mut reached), key.clone()))
            .max_by(Weighed::compared);
        longest.map_or_else(Critical::default, |last| {
            Critical::new(*AsRef::<Seconds>::as_ref(&last), self.chain(AsRef::<UnitKey>::as_ref(&last), &reached))
        })
    }

    fn chain(&self, last: &UnitKey, reached: &BTreeMap<UnitKey, Reach>) -> Vec<Weighed> {
        let mut chain: Vec<Weighed> = iter::successors(Some(last.clone()), |key| reached.get(key).and_then(Reach::via))
            .filter_map(|key| self.weights.get(&key).map(|seconds| Weighed::new(*seconds, key)))
            .collect();
        chain.reverse();
        chain
    }

    fn reached(&self, key: &UnitKey, reached: &mut BTreeMap<UnitKey, Reach>) -> Seconds {
        match reached.get(key) {
            Some(reach) => *reach.as_ref(),
            None => self.reaching(key, reached).kept(key, reached),
        }
    }

    fn reaching(&self, key: &UnitKey, reached: &mut BTreeMap<UnitKey, Reach>) -> Reach {
        let longest = key
            .role()
            .before(key, self.crates)
            .into_iter()
            .map(|before| Weighed::new(self.reached(&before, reached), before))
            .max_by(Weighed::compared);
        Reach::after(longest, self.weight(key))
    }

    fn weight(&self, key: &UnitKey) -> Seconds {
        self.weights.get(key).copied().unwrap_or_default()
    }
}
