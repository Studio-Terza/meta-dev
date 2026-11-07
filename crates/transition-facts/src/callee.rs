use std::fmt;

use lint_layout::{Casing, Name, Whole, Word};
use newtype::{Arbitrary, Counter, Newtype};

#[derive(Newtype, Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Order(usize);

impl Order {
    fn typed_in<'name>(self, earlier: &[Option<&'name Name>]) -> Option<&'name Name> {
        earlier.get(self.count()).copied().flatten()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Receiver {
    Owner,
    Bound(Name),
    Call(Order),
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Callee {
    Method { receiver: Receiver, name: Name },
    Path { owner: Option<Name>, name: Name },
}

impl Callee {
    #[must_use]
    pub fn owner<'name>(&'name self, caller: Option<&'name Name>, earlier: &[Option<&'name Name>]) -> Owner<'name> {
        match self {
            Self::Method { receiver, .. } => receiver.owner(caller, earlier),
            Self::Path { owner, .. } => owner
                .as_ref()
                .filter(|owner| owner.casing() != Casing::Lower)
                .map_or(Owner::Free, |owner| Owner::typed(owner, caller)),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner<'name> {
    Type(&'name Name),
    Free,
    Unknown,
}

impl<'name> Owner<'name> {
    const OWN_TYPE: Word<'static> = Word::new("Self");

    fn typed(owner: &'name Name, caller: Option<&'name Name>) -> Self {
        owner
            .matches::<Whole>(&[Self::OWN_TYPE])
            .map_or(Owner::Type(owner), |_| caller.map_or(Owner::Unknown, Owner::Type))
    }

    #[must_use]
    pub fn admits(self, subject: Option<&Name>) -> Option<Self> {
        match self {
            Owner::Type(owner) => subject.filter(|subject| *subject == owner).map(|_| self),
            Owner::Free => subject.map_or(Some(self), |_| None),
            Owner::Unknown => None,
        }
    }
}

impl Receiver {
    fn owner<'name>(&'name self, caller: Option<&'name Name>, earlier: &[Option<&'name Name>]) -> Owner<'name> {
        match self {
            Self::Owner => caller.map_or(Owner::Unknown, Owner::Type),
            Self::Bound(name) => Owner::typed(name, caller),
            Self::Call(order) => order.typed_in(earlier).map_or(Owner::Unknown, Owner::Type),
            Self::Unknown => Owner::Unknown,
        }
    }
}

impl AsRef<Name> for Callee {
    fn as_ref(&self) -> &Name {
        match self {
            Self::Method { name, .. } | Self::Path { name, .. } => name,
        }
    }
}

impl fmt::Display for Callee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Method { receiver: Receiver::Owner, name } => write!(formatter, "self.{name}"),
            Self::Method { receiver: Receiver::Bound(owner), name } => write!(formatter, "{owner}.{name}"),
            Self::Method { receiver: Receiver::Call(order), name } => {
                write!(formatter, "#{}.{name}", order.count())
            }
            Self::Method { receiver: Receiver::Unknown, name } => write!(formatter, "?.{name}"),
            Self::Path { owner: Some(owner), name } => write!(formatter, "{owner}::{name}"),
            Self::Path { owner: None, name } => write!(formatter, "{name}"),
        }
    }
}
