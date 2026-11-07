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
    pub const fn of_trait(ground: Option<Ground>) -> Self {
        match ground {
            Some(Ground::Ports) => Self::PortTrait,
            Some(Ground::Store) => Self::StoreTrait,
            Some(Ground::Core | Ground::Boundary) | None => Self::Trait,
        }
    }

    #[must_use]
    pub const fn implementer(self) -> Option<Self> {
        match self {
            Self::PortTrait => Some(Self::Port),
            Self::StoreTrait => Some(Self::Store),
            Self::Trait | Self::GenericImpl | Self::Port | Self::Store => None,
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
    pub const fn relation(self) -> Relation {
        match self {
            Self::Trait | Self::PortTrait | Self::StoreTrait => Relation::ImplementedBy,
            Self::GenericImpl | Self::Port | Self::Store => Relation::Implements,
        }
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ImplementedBy => formatter.write_str("implemented by"),
            Self::Implements => formatter.write_str("implements"),
        }
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trait => formatter.write_str("trait"),
            Self::PortTrait => formatter.write_str("port trait"),
            Self::StoreTrait => formatter.write_str("store trait"),
            Self::GenericImpl => formatter.write_str("generic impl"),
            Self::Port => formatter.write_str("port"),
            Self::Store => formatter.write_str("store"),
        }
    }
}
