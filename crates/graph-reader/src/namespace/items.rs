use graph::{Issued, ItemId};

use super::boundary::ScopeId;
use super::entries::DeclaredId;

#[derive(Debug, Default)]
pub(crate) struct ItemIds(Vec<Vec<Option<ItemId>>>);

impl From<&Issued> for ItemIds {
    fn from(issued: &Issued) -> ItemIds {
        ItemIds(issued.scopes().to_vec())
    }
}

impl ItemIds {
    pub(super) fn issued(&self, scope: ScopeId, item: DeclaredId) -> Option<ItemId> {
        scope.row(&self.0).and_then(|declared| item.row(declared)).copied().flatten()
    }
}
