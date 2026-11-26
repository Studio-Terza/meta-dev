use std::collections::BTreeSet;
use std::fmt;
use std::ops::Range;

use lint_layout::Name;
use newtype::proptest::arbitrary::any;
use newtype::proptest::collection::{btree_set, vec};
use newtype::proptest::strategy::Strategy;

use crate::type_ref::TypeRef;

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Property {
    Associative,
    Commutative,
    Identity,
    Inverse,
    Idempotent,
    Absorbing,
    Distributive,
    Preserves,
    Acting,
}

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq)]
pub enum LawKind {
    Roundtrip,
    Crud,
    Law,
    Proptest,
    Property(Property),
}

#[derive(newtype::Arbitrary, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    Input,
    Output,
    Edge,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
pub enum OperationOwner {
    Typed(TypeRef),
    Operand,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Operation {
    owner: OperationOwner,
    member: Name,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct Route {
    side: Side,
    operation: Option<Operation>,
    fallback: Option<OperationOwner>,
}

#[derive(newtype::Arbitrary, Clone, Debug, PartialEq, Eq)]
#[newtype(generate = Law::sampled)]
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
            LawKind::Roundtrip | LawKind::Crud | LawKind::Law | LawKind::Proptest => None,
        }
    }

    #[must_use]
    pub fn titled(self) -> Option<LawKind> {
        match self {
            LawKind::Roundtrip | LawKind::Crud | LawKind::Law | LawKind::Property(_) => Some(self),
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
    const SAMPLED: Range<usize> = 0..3;

    fn sampled() -> impl Strategy<Value = Law> {
        Strategy::prop_map(
            (
                any::<LawKind>(),
                any::<Name>(),
                btree_set(any::<Name>(), Law::SAMPLED),
                vec(any::<Route>(), Law::SAMPLED),
            ),
            |(kind, title, mentions, routes)| Law::new(kind, title, mentions, routes),
        )
    }

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
