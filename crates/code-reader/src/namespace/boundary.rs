use std::iter;
use std::ops::ControlFlow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopeId(usize);

impl ScopeId {
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct GlobId(usize);

impl GlobId {
    pub(super) fn ids<Glob>(globs: &[Glob]) -> impl Iterator<Item = GlobId> {
        (0..globs.len()).map(GlobId)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct Rounds(usize);

impl Rounds {
    pub(super) fn bounding(steps: impl Iterator<Item = ()>) -> Rounds {
        Rounds(iter::once(()).chain(steps).count())
    }

    pub(super) fn settled<State: PartialEq>(self, start: State, advanced: impl Fn(&State) -> State) -> State {
        Rounds::reached(iter::repeat_n((), self.0).try_fold(start, |state, ()| {
            let next = advanced(&state);
            Rounds::stepped(state, next)
        }))
    }

    fn stepped<State: PartialEq>(state: State, next: State) -> ControlFlow<State, State> {
        match [state].into_iter().find(|state| *state == next) {
            Some(_) => ControlFlow::Break(next),
            None => ControlFlow::Continue(next),
        }
    }

    fn reached<State>(settling: ControlFlow<State, State>) -> State {
        match settling {
            ControlFlow::Break(state) | ControlFlow::Continue(state) => state,
        }
    }
}
