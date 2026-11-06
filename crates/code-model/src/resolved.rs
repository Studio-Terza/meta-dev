use crate::item_path::ItemPath;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    Std(ItemPath),
    Crate(ItemPath),
    External(ItemPath),
    Unknown(ItemPath),
}

impl Resolved {
    #[must_use]
    pub fn std(&self) -> Option<&ItemPath> {
        match self {
            Resolved::Std(path) => Some(path),
            Resolved::Crate(_) | Resolved::External(_) | Resolved::Unknown(_) => None,
        }
    }
}
