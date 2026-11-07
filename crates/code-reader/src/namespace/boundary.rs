use std::iter;
use std::ops::ControlFlow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopeId(usize);

impl ScopeId {
    pub(super) const fn after<Row>(rows: &[Row]) -> Self {
        Self(rows.len())
    }

    pub(super) fn row<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }

    pub(super) fn ids<Row>(rows: &[Row]) -> impl Iterator<Item = Self> {
        (0..rows.len()).map(ScopeId)
    }

    pub(super) fn row_mut<Row>(self, rows: &mut [Row]) -> Option<&mut Row> {
        rows.get_mut(self.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlobId(usize);

impl GlobId {
    pub(super) fn ids<Glob>(globs: &[Glob]) -> impl Iterator<Item = Self> {
        (0..globs.len()).map(GlobId)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Reach(usize);

impl Reach {
    pub(super) const FIRST: Self = Self(0);

    pub(super) const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

pub(super) trait Lattice: PartialEq + Sized {
    fn joined(&self, computed: Self) -> Self;

    fn settled(self, advanced: impl Fn(&Self) -> Self) -> Self {
        Self::reached(iter::repeat(()).try_fold(self, |state, ()| {
            let next = state.joined(advanced(&state));
            state.stepped(next)
        }))
    }

    fn stepped(self, next: Self) -> ControlFlow<Self, Self> {
        match [self].into_iter().find(|state| *state == next) {
            Some(_) => ControlFlow::Break(next),
            None => ControlFlow::Continue(next),
        }
    }

    fn reached(settling: ControlFlow<Self, Self>) -> Self {
        match settling {
            ControlFlow::Break(state) | ControlFlow::Continue(state) => state,
        }
    }
}

impl<Entry: Lattice> Lattice for Vec<Entry> {
    fn joined(&self, computed: Self) -> Self {
        self.iter().zip(computed).map(|(held, found)| held.joined(found)).collect()
    }
}
