use std::fmt;

use port_core::Snapshot;

use crate::{Scale, Within};

#[derive(Clone, Debug)]
pub struct Bounded<D> {
    scale: Scale,
    done: D,
    snapshot: Snapshot,
    within: Vec<Within>,
}

impl<D> Bounded<D> {
    pub(crate) fn new(scale: Scale, done: D, snapshot: Snapshot, within: Vec<Within>) -> Bounded<D> {
        Bounded { scale, done, snapshot, within }
    }

    pub(crate) fn scale(&self) -> Scale {
        self.scale
    }

    pub(crate) fn snapshot(&self) -> &Snapshot {
        &self.snapshot
    }
}

impl<D: fmt::Display> fmt::Display for Bounded<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "  scale {}: {};", self.scale, self.done)?;
        self.within.iter().try_for_each(|within| write!(formatter, " {within};"))
    }
}
