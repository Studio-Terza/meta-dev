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
    pub(super) fn dealt<'tree>(&self, due: Vec<Due<'tree>>) -> Vec<Assigned<'tree>> {
        let live = &self.0;
        due.into_iter()
            .scan(0, |dealt: &mut usize, found| {
                let hand = found.owner().unwrap_or_else(|| {
                    let turn = live.get(*dealt % live.len().max(1)).copied().unwrap_or(Partition::MAIN);
                    *dealt += 1;
                    turn
                });
                Some(found.assigned(hand))
            })
            .collect()
    }

    pub(super) fn live(&self) -> impl Iterator<Item = Partition> + '_ {
        self.0.iter().copied()
    }
}
