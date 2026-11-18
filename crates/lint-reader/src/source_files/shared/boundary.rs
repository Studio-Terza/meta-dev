use std::collections::BTreeMap;

use file_ports::Size;

use super::protocol::{Assigned, Due};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Partition(usize);

impl Partition {
    pub const MAIN: Partition = Partition(0);
}

impl From<usize> for Partition {
    fn from(index: usize) -> Partition {
        Partition(index)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Hands(Vec<Partition>);

impl FromIterator<Partition> for Hands {
    fn from_iter<Found: IntoIterator<Item = Partition>>(found: Found) -> Hands {
        Hands(found.into_iter().collect())
    }
}

impl Hands {
    pub(super) fn dealt<'tree>(
        &self,
        due: Vec<Due<'tree>>,
        weighed: impl Fn(&Due<'tree>) -> Size,
    ) -> Vec<Assigned<'tree>> {
        let loads: BTreeMap<Partition, Size> = self.0.iter().map(|hand| (*hand, Size::default())).collect();
        due.into_iter()
            .scan(loads, |tally, found| {
                let lightest = tally.iter().min_by_key(|(hand, load)| (**load, **hand)).map(|(hand, _)| *hand);
                let hand = found.owner().or(lightest).unwrap_or(Partition::MAIN);
                let load = tally.entry(hand).or_default();
                *load = [*load, weighed(&found)].into_iter().sum();
                Some(found.assigned(hand))
            })
            .collect()
    }

    pub(super) fn live(&self) -> impl Iterator<Item = Partition> + '_ {
        self.0.iter().copied()
    }
}
