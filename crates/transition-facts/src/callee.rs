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
    const OWN_TYPE: Word<'static> = Word::new("Self");

    #[must_use]
    pub fn owner<'name>(&'name self, caller: Option<&'name Name>, earlier: &[Option<&'name Name>]) -> Owner<'name> {
        match self {
            Callee::Method { receiver, .. } => receiver.owner(caller, earlier),
            Callee::Path { owner, .. } => Self::named(owner.as_ref(), caller),
        }
    }

    fn named<'name>(owner: Option<&'name Name>, caller: Option<&'name Name>) -> Owner<'name> {
        owner.filter(|owner| owner.casing() != Casing::Lower).map_or(Owner::Free, |owner| {
            owner
                .matches::<Whole>(&[Self::OWN_TYPE])
                .map_or(Owner::Type(owner), |_| caller.map_or(Owner::Unknown, Owner::Type))
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner<'name> {
    Type(&'name Name),
    Free,
    Unknown,
}

impl<'name> Owner<'name> {
    #[must_use]
    pub fn admits(self, subject: Option<&Name>) -> Option<Owner<'name>> {
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
            Receiver::Owner => caller.map_or(Owner::Unknown, Owner::Type),
            Receiver::Bound(name) => Owner::Type(name),
            Receiver::Call(order) => order.typed_in(earlier).map_or(Owner::Unknown, Owner::Type),
            Receiver::Unknown => Owner::Unknown,
        }
    }
}

impl AsRef<Name> for Callee {
    fn as_ref(&self) -> &Name {
        match self {
            Callee::Method { name, .. } | Callee::Path { name, .. } => name,
        }
    }
}

impl fmt::Display for Callee {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Callee::Method { receiver: Receiver::Owner, name } => write!(formatter, "self.{name}"),
            Callee::Method { receiver: Receiver::Bound(owner), name } => write!(formatter, "{owner}.{name}"),
            Callee::Method { receiver: Receiver::Call(order), name } => {
                write!(formatter, "#{}.{name}", order.count())
            }
            Callee::Method { receiver: Receiver::Unknown, name } => write!(formatter, "?.{name}"),
            Callee::Path { owner: Some(owner), name } => write!(formatter, "{owner}::{name}"),
            Callee::Path { owner: None, name } => write!(formatter, "{name}"),
        }
    }
}
