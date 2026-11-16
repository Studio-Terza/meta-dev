use std::fmt::Debug;

use newtype::Enumerated;
use port_core::Op;

use crate::store_failure::Unmoved;
use crate::{Batch, Prepared, Statements, StoreFailure, Written};

pub trait Lifecycle: Enumerated + Copy + Eq + Debug + 'static {
    const EDGES: &'static [Edge<Self>];

    #[must_use]
    fn states() -> &'static [Self] {
        Self::VARIANTS
    }
}

pub trait Staged {
    type Key: Debug;

    type State: Lifecycle;

    fn moved(key: &Self::Key, from: Self::State, to: Self::State) -> Prepared<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Edge<State> {
    from: State,
    to: State,
}

#[derive(Debug, Op)]
pub struct Moved<S: Staged> {
    key: S::Key,
    edge: Edge<S::State>,
}

impl<State> Edge<State> {
    #[must_use]
    pub const fn new(from: State, to: State) -> Edge<State> {
        Edge { from, to }
    }
}

impl<S: Staged> Moved<S> {
    #[must_use]
    pub fn new(key: S::Key, edge: Edge<S::State>) -> Moved<S> {
        Moved { key, edge }
    }
}

impl<S: Staged> Batch for Moved<S> {
    type Failure = StoreFailure;

    fn written(self, statements: &mut Statements<'_>) -> Result<Written, StoreFailure> {
        S::State::EDGES.iter().find(|listed| **listed == self.edge).ok_or(StoreFailure::Unmoved(Unmoved::Unlisted))?;
        let Edge { from, to } = self.edge;
        let written = statements.written(S::moved(&self.key, from, to))?;
        Some(written).filter(|moved| *moved != Written::default()).ok_or(StoreFailure::Unmoved(Unmoved::Elsewhere))
    }
}
