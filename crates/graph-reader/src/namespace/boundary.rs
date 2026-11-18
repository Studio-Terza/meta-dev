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

    pub(super) fn row_after<Row>(self, base: ScopeId, rows: &[Row]) -> Option<&Row> {
        self.0.checked_sub(base.0).and_then(|index| rows.get(index))
    }

    pub(super) fn row_mut<Row>(self, rows: &mut [Row]) -> Option<&mut Row> {
        rows.get_mut(self.0)
    }

    pub(crate) fn above(self, base: ScopeId) -> ScopeId {
        ScopeId(self.0 + base.0)
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

pub(super) trait Lattice: Eq + Sized {
    fn joined(&self, computed: Self) -> Self;

    fn settled(self, advanced: impl Fn(&Self) -> Self) -> Self {
        Self::reached(iter::repeat(()).try_fold(self, |state, ()| {
            let next = state.joined(advanced(&state));
            Self::stepped(&state, next)
        }))
    }

    fn stepped(state: &Self, next: Self) -> ControlFlow<Self, Self> {
        match (next != *state).then_some(()) {
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

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScopeKey {
    within: std::sync::Arc<[lint_sweep::WalkPosition]>,
    at: Option<NodeKey>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct NodeKey {
    position: lint_sweep::WalkPosition,
    role: KeyRole,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum KeyRole {
    Scope,
    Owner,
    Filler,
    Subject,
}

impl ScopeKey {
    pub(crate) fn reached(within: &std::sync::Arc<[lint_sweep::WalkPosition]>, at: NodeKey) -> ScopeKey {
        ScopeKey { within: std::sync::Arc::clone(within), at: Some(at) }
    }
}

impl NodeKey {
    pub(crate) fn new(position: lint_sweep::WalkPosition, role: KeyRole) -> NodeKey {
        NodeKey { position, role }
    }
}

#[derive(Debug, Default)]
pub(crate) struct ScopeMap(std::collections::BTreeMap<ScopeKey, ScopeId>);

impl ScopeMap {
    pub(crate) fn scope(&self, key: &ScopeKey) -> ScopeId {
        self.0.get(key).copied().unwrap_or(ScopeId::FIRST)
    }

    pub(super) fn keyed(keys: impl IntoIterator<Item = ScopeKey>) -> ScopeMap {
        let top = [(ScopeKey::default(), ScopeId::FIRST)];
        ScopeMap(
            top.into_iter().chain(keys.into_iter().enumerate().map(|(index, key)| (key, ScopeId(index + 1)))).collect(),
        )
    }
}
