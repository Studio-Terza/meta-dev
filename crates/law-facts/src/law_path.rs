use std::fmt;

use lint_core::{Finding, Fix, Site};
use lint_layout::Name;

use crate::kind::{Kind, Property};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Input,
    Output,
    Edge,
}

impl Side {
    const PRESERVED: [Self; 3] = [Self::Input, Self::Output, Self::Edge];
    const SINGLE: [Self; 4] = [Self::Input; 4];

    pub(crate) fn checked_by(kind: Kind) -> &'static [Self] {
        match kind.property() {
            Some(Property::Preserves) => &Self::PRESERVED,
            Some(_) | None => &Self::SINGLE,
        }
    }
}

impl fmt::Display for Side {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input => formatter.write_str("input side"),
            Self::Output => formatter.write_str("output side"),
            Self::Edge => formatter.write_str("edge function"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Owner {
    Named(Name),
    Operand,
}

impl Owner {
    fn admitting<'name>(&self, owner: &'name Name) -> Option<&'name Name> {
        match self {
            Self::Named(named) => Some(owner).filter(|owner| *owner == named),
            Self::Operand => Some(owner),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LawPath {
    owner: Owner,
    member: Name,
}

impl LawPath {
    pub(crate) fn new(owner: Owner, member: Name) -> Self {
        Self { owner, member }
    }

    #[must_use]
    pub fn reaching(&self, owner: &Name, member: &Name) -> Option<&Self> {
        Some(self).filter(|path| path.owner.admitting(owner).is_some()).filter(|path| path.member == *member)
    }
}

impl fmt::Display for LawPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.owner {
            Owner::Named(owner) => write!(formatter, "{owner}::{}", self.member),
            Owner::Operand => write!(formatter, "{} of the type on its side", self.member),
        }
    }
}

#[derive(Debug)]
pub struct Route {
    side: Side,
    path: Option<LawPath>,
}

impl Route {
    pub(crate) fn new(side: Side, path: Option<LawPath>) -> Self {
        Self { side, path }
    }

    pub(crate) fn reaching(&self, owner: &Name, member: &Name) -> Option<&LawPath> {
        self.path.as_ref().and_then(|path| path.reaching(owner, member))
    }

    pub(crate) fn of_side(&self, side: Side) -> Option<&Self> {
        Some(self).filter(|route| route.side == side)
    }

    pub(crate) fn closure(&self) -> Option<&Self> {
        Some(self).filter(|route| route.path.is_none())
    }
}

impl fmt::Display for Route {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.path {
            Some(path) => write!(formatter, "{} resolves to {path}", self.side),
            None => write!(formatter, "{} is a closure the lint cannot resolve to an operation", self.side),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Unresolved<'file> {
    site: Site<'file>,
    name: Name,
}

impl<'file> Unresolved<'file> {
    pub(crate) fn new(site: Site<'file>, name: Name) -> Self {
        Unresolved { site, name }
    }
}

impl<'file> AsRef<Site<'file>> for Unresolved<'file> {
    fn as_ref(&self) -> &Site<'file> {
        &self.site
    }
}

impl fmt::Display for Unresolved<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "law {}! checks a closure the lint cannot resolve to an operation", self.name)
    }
}

impl Finding for Unresolved<'_> {
    fn fix(&self) -> Fix {
        Fix::new("pass the operation path (Type::function), or a closure that calls it on its parameters")
    }
}
