mod critical;
mod reach;

use std::collections::{BTreeMap, BTreeSet};
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
        let reached: BTreeMap<UnitKey, Reach> = self.reaches().map(|found| (found.key, found.reach)).collect();
        let longest = self
            .weights
            .keys()
            .filter_map(|key| reached.get(key).map(|reach| Weighed::new(*reach.as_ref(), key.clone())))
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

    fn reaches(&self) -> impl Iterator<Item = Reached> + '_ {
        let pending = self.weights.keys().rev().cloned().map(Step::Enter).collect();
        Reaching { dag: self, pending, entered: BTreeSet::new(), memo: BTreeMap::new() }.flatten()
    }

    fn weight(&self, key: &UnitKey) -> Seconds {
        self.weights.get(key).copied().unwrap_or_default()
    }
}

#[derive(Debug)]
struct Reached {
    key: UnitKey,
    reach: Reach,
}

#[derive(Debug)]
enum Step {
    Enter(UnitKey),
    Exit(Entered),
}

#[derive(Debug)]
struct Entered {
    key: UnitKey,
}

#[derive(Debug, Default)]
struct Opened {
    pushed: Vec<Step>,
    entered: Option<UnitKey>,
    reached: Option<Reached>,
}

#[derive(Debug)]
struct Reaching<'dag, 'crates> {
    dag: &'dag UnitDag<'crates>,
    pending: Vec<Step>,
    entered: BTreeSet<UnitKey>,
    memo: BTreeMap<UnitKey, Seconds>,
}

impl Step {
    fn opened(self, reaching: &Reaching<'_, '_>) -> Opened {
        match self {
            Step::Enter(key) => reaching.entering(key),
            Step::Exit(key) => reaching.exiting(key),
        }
    }
}

impl Reaching<'_, '_> {
    fn entering(&self, key: UnitKey) -> Opened {
        self.entered.get(&key).map_or_else(
            || {
                let before = key.role().before(&key, self.dag.crates);
                let pushed = iter::once(Step::Exit(Entered { key: key.clone() }))
                    .chain(before.into_iter().rev().map(Step::Enter))
                    .collect();
                Opened { pushed, entered: Some(key), reached: None }
            },
            |_| Opened::default(),
        )
    }

    fn exiting(&self, exit: Entered) -> Opened {
        let Entered { key } = exit;
        let reach = Reach::after(self.longest(&key), self.dag.weight(&key));
        Opened { pushed: Vec::new(), entered: None, reached: Some(Reached { key, reach }) }
    }

    fn longest(&self, key: &UnitKey) -> Option<Weighed> {
        key.role()
            .before(key, self.dag.crates)
            .into_iter()
            .filter_map(|earlier| self.memo.get(&earlier).map(|seconds| Weighed::new(*seconds, earlier)))
            .max_by(Weighed::compared)
    }
}

impl Iterator for Reaching<'_, '_> {
    type Item = Option<Reached>;

    fn next(&mut self) -> Option<Option<Reached>> {
        let Opened { pushed, entered, reached } = self.pending.pop()?.opened(self);
        self.pending.extend(pushed);
        self.entered.extend(entered);
        self.memo.extend(reached.iter().map(|found| (found.key.clone(), *found.reach.as_ref())));
        Some(reached)
    }
}
