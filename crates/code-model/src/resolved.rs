use lint_layout::Name;

use crate::boundary::ItemId;
use crate::item_path::ItemPath;
use crate::primitive::Primitive;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolved {
    Std(ItemPath),
    Crate(ItemId),
    External(ItemPath),
    Primitive(Primitive),
    Param(Name),
    Unknown(ItemPath),
}

impl Resolved {
    #[must_use]
    pub fn std(&self) -> Option<&ItemPath> {
        match self {
            Resolved::Std(path) => Some(path),
            Resolved::Crate(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(_) => None,
        }
    }

    #[must_use]
    pub fn crate_item(&self) -> Option<ItemId> {
        match self {
            Resolved::Crate(item) => Some(*item),
            Resolved::Std(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(_) => None,
        }
    }

    #[must_use]
    pub fn primitive(&self) -> Option<Primitive> {
        match self {
            Resolved::Primitive(primitive) => Some(*primitive),
            Resolved::Std(_)
            | Resolved::Crate(_)
            | Resolved::External(_)
            | Resolved::Param(_)
            | Resolved::Unknown(_) => None,
        }
    }

    pub(crate) fn parameter(&self) -> Option<&Name> {
        match self {
            Resolved::Param(name) => Some(name),
            Resolved::Std(_)
            | Resolved::Crate(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Unknown(_) => None,
        }
    }
}
