use code_model::ItemId;

use super::boundary::ScopeId;
use super::entries::DeclaredId;

#[derive(Debug, Default)]
pub(crate) struct ItemIds(Vec<Vec<Option<ItemId>>>);

impl FromIterator<Vec<Option<ItemId>>> for ItemIds {
    fn from_iter<Scopes: IntoIterator<Item = Vec<Option<ItemId>>>>(scopes: Scopes) -> Self {
        Self(scopes.into_iter().collect())
    }
}

impl ItemIds {
    pub(super) fn issued(&self, scope: ScopeId, item: DeclaredId) -> Option<ItemId> {
        scope.row(&self.0).and_then(|declared| item.row(declared)).copied().flatten()
    }
}
