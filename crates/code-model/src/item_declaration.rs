use crate::boundary::ItemId;
use crate::region::Region;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemDeclaration {
    item: ItemId,
    region: Region,
}

impl ItemDeclaration {
    #[must_use]
    pub fn new(item: ItemId, region: Region) -> ItemDeclaration {
        ItemDeclaration { item, region }
    }

    #[must_use]
    pub fn declared(&self) -> ItemId {
        self.item
    }
}

impl AsRef<Region> for ItemDeclaration {
    fn as_ref(&self) -> &Region {
        &self.region
    }
}
