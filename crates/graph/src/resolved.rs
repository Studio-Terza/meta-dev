use algebra::{Act, Semigroup};
use lint_layout::Name;

use crate::boundary::ItemId;
use crate::external::External;
use crate::item_path::ItemPath;
use crate::member::Member;
use crate::primitive::Primitive;
use crate::unresolved::Unresolved;

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
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

impl Act<ItemPath> for Resolved {
    fn acted(self, by: ItemPath) -> Option<Resolved> {
        match self {
            Resolved::Std(path) => Some(Resolved::Std(path.combine(by))),
            Resolved::StdTrait(path) => {
                Some(by.last().map_or_else(
                    || Resolved::StdTrait(path.clone()),
                    |_| Resolved::Std(path.clone().combine(by.clone())),
                ))
            }
            Resolved::External(external) => Some(Resolved::External(external.combine(External::from(by)))),
            unpathed @ (Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::Primitive(_)
            | Resolved::Param(_)
            | Resolved::Unknown(..)) => by.last().map_or(Some(unpathed), |_| None),
        }
    }
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
    pub fn std_trait(&self) -> Option<StdTrait> {
        match self {
            Resolved::StdTrait(_) => Some(StdTrait),
            Resolved::Std(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
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
    pub fn nominal(&self) -> Option<Nominal<'_>> {
        match self {
            Resolved::Param(_) | Resolved::Primitive(_) => None,
            Resolved::Std(_)
            | Resolved::StdTrait(_)
            | Resolved::Crate(_)
            | Resolved::Member(_)
            | Resolved::External(_)
            | Resolved::Unknown(..) => Some(Nominal(self)),
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
    pub fn unknown(&self) -> Option<UnknownPath<'_>> {
        match self {
            Resolved::Unknown(path, _) => Some(UnknownPath(path)),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StdTrait;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnknownPath<'path>(&'path ItemPath);

impl<'path> From<UnknownPath<'path>> for &'path ItemPath {
    fn from(unknown: UnknownPath<'path>) -> &'path ItemPath {
        unknown.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Nominal<'resolved>(&'resolved Resolved);
