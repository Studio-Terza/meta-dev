use lint_layout::Name;

use crate::boundary::ItemId;
use crate::external::External;
use crate::item_path::ItemPath;
use crate::member::Member;
use crate::primitive::Primitive;
use crate::unresolved::Unresolved;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Resolved {
    Std(ItemPath),
    StdTrait(ItemPath),
    Crate(ItemId),
    Member(Member),
    External(External),
    Primitive(Primitive),
    Param(Name),
    Unknown(ItemPath, Unresolved),
}

impl Resolved {
    #[must_use]
    pub fn std(&self) -> Option<&ItemPath> {
        match self {
            Resolved::Std(path) | Resolved::StdTrait(path) => Some(path),
            Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn joined(&self, segment: &Name) -> Option<Resolved> {
        match self {
            Resolved::Std(path) | Resolved::StdTrait(path) => Some(Resolved::Std(path.joined(segment))),
            Resolved::External(external) => Some(Resolved::External(external.joined(segment))),
            Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn external(&self) -> Option<&External> {
        match self {
            Resolved::External(external) => Some(external),
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn crate_item(&self) -> Option<ItemId> {
        match self {
            Resolved::Crate(item) => Some(*item),
            Resolved::Member(_)
            | Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn member(&self) -> Option<&Member> {
        match self {
            Resolved::Member(member) => Some(member),
            Resolved::Crate(_)
            | Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn primitive(&self) -> Option<Primitive> {
        match self {
            Resolved::Primitive(primitive) => Some(*primitive),
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..) => None,
        }
    }

    #[must_use]
    pub fn known(&self) -> Option<&Resolved> {
        match self {
            Resolved::Unknown(..) => None,
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_) => Some(self),
        }
    }

    #[must_use]
    pub fn unresolved(&self) -> Option<Unresolved> {
        match self {
            Resolved::Unknown(_, cause) => Some(*cause),
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_) => None,
        }
    }

    #[must_use]
    pub fn parameter(&self) -> Option<&Name> {
        match self {
            Resolved::Param(name) => Some(name),
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Primitive(_)
            | Resolved::Unknown(..) => None,
        }
    }
}
