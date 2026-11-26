use lint_transition_graph::Ground;
use strum::Display;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    #[strum(serialize = "trait")]
    Trait,
    #[strum(serialize = "port trait")]
    PortTrait,
    #[strum(serialize = "store trait")]
    StoreTrait,
    #[strum(serialize = "generic impl")]
    GenericImpl,
    #[strum(serialize = "port")]
    Port,
    #[strum(serialize = "store")]
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

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    #[strum(serialize = "implemented by")]
    ImplementedBy,
    #[strum(serialize = "implements")]
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
