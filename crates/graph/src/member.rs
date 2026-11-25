use lint_layout::Name;

use crate::boundary::ItemId;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    owner: ItemId,
    name: Name,
}

impl Member {
    #[must_use]
    pub fn new(owner: ItemId, name: Name) -> Member {
        Member { owner, name }
    }

    #[must_use]
    pub fn owner(&self) -> ItemId {
        self.owner
    }
}
