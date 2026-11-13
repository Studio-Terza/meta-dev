use std::fmt;

use lint_transition_graph::Ground;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Trait,
    PortTrait,
    StoreTrait,
    GenericImpl,
    Port,
    Store,
}

impl Kind {
    #[must_use]
    pub fn of_trait(ground: Option<Ground>) -> Kind {
        match ground {
            Some(Ground::Ports) => Kind::PortTrait,
            Some(Ground::Store) => Kind::StoreTrait,
            Some(Ground::Core | Ground::Boundary) | None => Kind::Trait,
        }
    }

    #[must_use]
    pub fn implementer(self) -> Option<Kind> {
        match self {
            Kind::PortTrait => Some(Kind::Port),
            Kind::StoreTrait => Some(Kind::Store),
            Kind::Trait | Kind::GenericImpl | Kind::Port | Kind::Store => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    ImplementedBy,
    Implements,
}

impl Kind {
    #[must_use]
    pub fn relation(self) -> Relation {
        match self {
            Kind::Trait | Kind::PortTrait | Kind::StoreTrait => Relation::ImplementedBy,
            Kind::GenericImpl | Kind::Port | Kind::Store => Relation::Implements,
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relation::ImplementedBy => formatter.write_str("implemented by"),
            Relation::Implements => formatter.write_str("implements"),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Kind::Trait => formatter.write_str("trait"),
            Kind::PortTrait => formatter.write_str("port trait"),
            Kind::StoreTrait => formatter.write_str("store trait"),
            Kind::GenericImpl => formatter.write_str("generic impl"),
            Kind::Port => formatter.write_str("port"),
            Kind::Store => formatter.write_str("store"),
        }
    }
}
