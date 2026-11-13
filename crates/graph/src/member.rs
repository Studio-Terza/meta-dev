use crate::boundary::ItemId;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Member {
    owner: ItemId,
}

impl From<ItemId> for Member {
    fn from(owner: ItemId) -> Member {
        Member { owner }
    }
}

impl Member {
    #[must_use]
    pub fn owner(&self) -> ItemId {
        self.owner
    }
}
