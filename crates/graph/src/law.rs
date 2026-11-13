use std::collections::BTreeSet;
use std::fmt;

use lint_layout::Name;

use crate::type_ref::TypeRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    Associative,
    Commutative,
    Identity,
    Inverse,
    Idempotent,
    Absorbing,
    Distributive,
    Preserves,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LawKind {
    Roundtrip,
    Law,
    Proptest,
    Property(Property),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Input,
    Output,
    Edge,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OperationOwner {
    Typed(TypeRef),
    Operand,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    owner: OperationOwner,
    member: Name,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Route {
    side: Side,
    operation: Option<Operation>,
    fallback: Option<OperationOwner>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Law {
    kind: LawKind,
    title: Name,
    mentions: BTreeSet<Name>,
    routes: Vec<Route>,
}

impl LawKind {
    #[must_use]
    pub fn property(self) -> Option<Property> {
        match self {
            LawKind::Property(property) => Some(property),
            LawKind::Roundtrip | LawKind::Law | LawKind::Proptest => None,
        }
    }

    #[must_use]
    pub fn titled(self) -> Option<LawKind> {
        match self {
            LawKind::Roundtrip | LawKind::Law | LawKind::Property(_) => Some(self),
            LawKind::Proptest => None,
        }
    }
}

impl Side {
    const PRESERVED: [Side; 3] = [Side::Input, Side::Output, Side::Edge];
    const SINGLE: [Side; 4] = [Side::Input; 4];

    #[must_use]
    pub fn checked_by(kind: LawKind) -> &'static [Side] {
        match kind.property() {
            Some(Property::Preserves) => &Self::PRESERVED,
            Some(_) | None => &Self::SINGLE,
        }
    }
}

impl Operation {
    #[must_use]
    pub fn new(owner: OperationOwner, member: Name) -> Operation {
        Operation { owner, member }
    }

    #[must_use]
    pub fn owner(&self) -> &OperationOwner {
        &self.owner
    }
}

impl AsRef<Name> for Operation {
    fn as_ref(&self) -> &Name {
        &self.member
    }
}

impl Route {
    #[must_use]
    pub fn new(side: Side, operation: Option<Operation>, fallback: Option<OperationOwner>) -> Route {
        Route { side, operation, fallback }
    }

    #[must_use]
    pub fn fallback(&self) -> Option<&OperationOwner> {
        self.fallback.as_ref()
    }

    #[must_use]
    pub fn side(&self) -> Side {
        self.side
    }

    #[must_use]
    pub fn operation(&self) -> Option<&Operation> {
        self.operation.as_ref()
    }
}

impl Law {
    #[must_use]
    pub fn new(kind: LawKind, title: Name, mentions: BTreeSet<Name>, routes: Vec<Route>) -> Law {
        Law { kind, title, mentions, routes }
    }

    #[must_use]
    pub fn kind(&self) -> LawKind {
        self.kind
    }

    #[must_use]
    pub fn mentions(&self) -> &BTreeSet<Name> {
        &self.mentions
    }

    #[must_use]
    pub fn routes(&self) -> &[Route] {
        &self.routes
    }
}

impl AsRef<Name> for Law {
    fn as_ref(&self) -> &Name {
        &self.title
    }
}

impl fmt::Display for Side {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Side::Input => formatter.write_str("input side"),
            Side::Output => formatter.write_str("output side"),
            Side::Edge => formatter.write_str("edge function"),
        }
    }
}

impl OperationOwner {
    #[must_use]
    pub fn typed(&self) -> Option<&TypeRef> {
        match self {
            OperationOwner::Typed(ty) => Some(ty),
            OperationOwner::Operand => None,
        }
    }
}
