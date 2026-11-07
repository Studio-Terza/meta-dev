use lint_layout::{Name, Word};

use crate::endpoint::{Endpoint, Nature, Unnamed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Resolved {
    Itself,
    Found(Endpoint),
}

impl Resolved {
    pub(crate) fn interface(name: Word<'static>) -> Self {
        Self::Found(Endpoint::new(Nature::Interface, Name::from(name)))
    }

    pub(crate) fn tupled(elements: Vec<Self>, selfish: Nature) -> Self {
        let unit = elements.first().map(|_| ());
        unit.map_or_else(
            || Self::Found(Endpoint::from(Unnamed::Unit)),
            |()| {
                elements
                    .into_iter()
                    .find(|element| element.nature(selfish) == Nature::Primitive)
                    .unwrap_or_else(|| Self::Found(Endpoint::from(Unnamed::Opaque)))
            },
        )
    }

    pub(crate) fn resolved_with<'found>(&'found self, selfish: &'found Endpoint) -> &'found Endpoint {
        match self {
            Self::Itself => selfish,
            Self::Found(found) => found,
        }
    }

    pub(crate) fn map_found(self, located: impl FnOnce(Endpoint) -> Endpoint) -> Self {
        match self {
            Self::Itself => Self::Itself,
            Self::Found(found) => Self::Found(located(found)),
        }
    }

    pub(crate) fn or_opaque(self) -> Endpoint {
        match self {
            Self::Itself => Endpoint::from(Unnamed::Opaque),
            Self::Found(found) => found,
        }
    }

    fn nature(&self, selfish: Nature) -> Nature {
        match self {
            Self::Itself => selfish,
            Self::Found(found) => found.nature(),
        }
    }
}
