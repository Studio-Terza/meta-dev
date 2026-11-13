use lint_layout::{Name, Word};

use crate::transitions::endpoint::{Endpoint, Nature, Unnamed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Resolved {
    Itself,
    Found(Endpoint),
}

impl Resolved {
    pub(crate) fn interface(name: Word<'static>) -> Resolved {
        Resolved::Found(Endpoint::new(Nature::Interface, Name::from(name)))
    }

    pub(crate) fn resolved_with<'found>(&'found self, selfish: &'found Endpoint) -> &'found Endpoint {
        match self {
            Resolved::Itself => selfish,
            Resolved::Found(found) => found,
        }
    }

    pub(crate) fn or_opaque(self) -> Endpoint {
        match self {
            Resolved::Itself => Endpoint::from(Unnamed::Opaque),
            Resolved::Found(found) => found,
        }
    }

    pub(crate) fn nature(&self, selfish: Nature) -> Nature {
        match self {
            Resolved::Itself => selfish,
            Resolved::Found(found) => found.nature(),
        }
    }
}
