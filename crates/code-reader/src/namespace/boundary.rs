use super::entries::Entries;
use super::scope::ScopeRow;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ScopeId(usize);

impl ScopeId {
    pub(super) fn after(rows: &[ScopeRow]) -> ScopeId {
        ScopeId(rows.len())
    }

    pub(super) fn row(self, rows: &[ScopeRow]) -> Option<&ScopeRow> {
        rows.get(self.0)
    }

    pub(super) fn entries_mut(self, rows: &mut [ScopeRow]) -> Option<&mut Entries> {
        rows.get_mut(self.0).map(ScopeRow::entries_mut)
    }
}
