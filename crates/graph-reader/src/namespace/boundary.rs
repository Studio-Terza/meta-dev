use std::collections::BTreeSet;
use std::iter;
use std::ops::ControlFlow;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScopeId(usize);

impl ScopeId {
    pub(crate) const FIRST: ScopeId = ScopeId(0);

    pub(super) fn after<Row>(rows: &[Row]) -> ScopeId {
        ScopeId(rows.len())
    }

    pub(super) fn row<Row>(self, rows: &[Row]) -> Option<&Row> {
        rows.get(self.0)
    }

    pub(super) fn ids<Row>(rows: &[Row]) -> impl Iterator<Item = ScopeId> {
        (0..rows.len()).map(ScopeId)
    }

    pub(super) fn row_mut<Row>(self, rows: &mut [Row]) -> Option<&mut Row> {
        rows.get_mut(self.0)
    }

    pub(crate) fn above(self, base: ScopeId) -> ScopeId {
        ScopeId(self.0 + base.0)
    }

    pub(super) fn nested(self) -> ScopeId {
        ScopeId(self.0 + 1)
    }

    pub(super) fn below_top(self) -> Option<ScopeId> {
        self.0.checked_sub(1).map(ScopeId)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct GlobId(usize);

impl GlobId {
    pub(super) fn ids<Glob>(globs: &[Glob]) -> impl Iterator<Item = GlobId> {
        (0..globs.len()).map(GlobId)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct Reach(usize);

impl Reach {
    pub(super) const FIRST: Reach = Reach(0);

    pub(super) fn next(self) -> Reach {
        Reach(self.0 + 1)
    }
}

pub(super) trait Lattice: Ord + Clone {
    fn joined(&self, computed: Self) -> Self;

    fn settled(self, advanced: impl Fn(&Self) -> Self) -> Self {
        let mut reached = BTreeSet::from([self.clone()]);
        Self::reached(iter::repeat(()).try_fold(self, |state, ()| {
            let next = state.joined(advanced(&state));
            Self::stepped(&mut reached, next)
        }))
    }

    fn stepped(reached: &mut BTreeSet<Self>, next: Self) -> ControlFlow<Self, Self> {
        match reached.insert(next.clone()).then_some(()) {
            Some(()) => ControlFlow::Continue(next),
            None => ControlFlow::Break(next),
        }
    }

    fn reached(settling: ControlFlow<Self, Self>) -> Self {
        match settling {
            ControlFlow::Break(state) | ControlFlow::Continue(state) => state,
        }
    }
}

impl<Entry: Lattice> Lattice for Vec<Entry> {
    fn joined(&self, computed: Vec<Entry>) -> Vec<Entry> {
        self.iter().zip(computed).map(|(held, found)| held.joined(found)).collect()
    }
}
