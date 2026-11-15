use crate::item_path::ItemPath;
use crate::path_use::PathUse;
use crate::resolved::Resolved;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Dispatch {
    written: ItemPath,
    resolved: Resolved,
}

impl From<&PathUse> for Dispatch {
    fn from(used: &PathUse) -> Dispatch {
        Dispatch { written: used.written().clone(), resolved: used.resolved().clone() }
    }
}

impl AsRef<ItemPath> for Dispatch {
    fn as_ref(&self) -> &ItemPath {
        &self.written
    }
}

impl AsRef<Resolved> for Dispatch {
    fn as_ref(&self) -> &Resolved {
        &self.resolved
    }
}
