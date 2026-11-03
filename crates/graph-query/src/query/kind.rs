use std::fmt;

use transition_facts::Ground;

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
        formatter.write_str(match self {
            Relation::ImplementedBy => "implemented by",
            Relation::Implements => "implements",
        })
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Kind::Trait => "trait",
            Kind::PortTrait => "port trait",
            Kind::StoreTrait => "store trait",
            Kind::GenericImpl => "generic impl",
            Kind::Port => "port",
            Kind::Store => "store",
        })
    }
}
