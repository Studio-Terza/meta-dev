use lint_layout::Name;

use crate::boundary::ItemId;
use crate::item_path::ItemPath;
use crate::primitive::Primitive;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resolved {
    Std(ItemPath),
    StdTrait(ItemPath),
    Crate(ItemId),
    External(ItemPath),
    Primitive(Primitive),
    Param(Name),
    Unknown(ItemPath),
}

impl Resolved {
    #[must_use]
    pub const fn std(&self) -> Option<&ItemPath> {
        match self {
            Self::Std(path) | Self::StdTrait(path) => Some(path),
            Self::Crate(_) | Self::External(_) | Self::Primitive(_) | Self::Param(_) | Self::Unknown(_) => None,
        }
    }

    #[must_use]
    pub const fn crate_item(&self) -> Option<ItemId> {
        match self {
            Self::Crate(item) => Some(*item),
            Self::Std(_)
            | Self::StdTrait(_)
            | Self::External(_)
            | Self::Primitive(_)
            | Self::Param(_)
            | Self::Unknown(_) => None,
        }
    }

    #[must_use]
    pub const fn primitive(&self) -> Option<Primitive> {
        match self {
            Self::Primitive(primitive) => Some(*primitive),
            Self::Std(_)
            | Self::StdTrait(_)
            | Self::Crate(_)
            | Self::External(_)
            | Self::Param(_)
            | Self::Unknown(_) => None,
        }
    }

    pub(crate) const fn parameter(&self) -> Option<&Name> {
        match self {
            Self::Param(name) => Some(name),
            Self::Std(_)
            | Self::StdTrait(_)
            | Self::Crate(_)
            | Self::External(_)
            | Self::Primitive(_)
            | Self::Unknown(_) => None,
        }
    }
}
