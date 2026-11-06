use std::iter::Sum;

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

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Measure(usize);

impl Measure {
    pub(super) const ZERO: Measure = Measure(0);
    pub(super) const ONE: Measure = Measure(1);
    pub(super) const TWO: Measure = Measure(2);
}

impl From<usize> for Measure {
    fn from(count: usize) -> Measure {
        Measure(count)
    }
}

impl Sum for Measure {
    fn sum<Measures: Iterator<Item = Measure>>(measures: Measures) -> Measure {
        Measure(measures.map(|measure| measure.0).sum())
    }
}
