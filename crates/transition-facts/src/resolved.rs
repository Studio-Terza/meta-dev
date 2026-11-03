use lint_layout::{Name, Word};

use crate::endpoint::{Endpoint, Nature, Unnamed};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Resolved {
    Itself,
    Found(Endpoint),
}

impl Resolved {
    pub(crate) fn interface(name: Word<'static>) -> Resolved {
        Resolved::Found(Endpoint::new(Nature::Interface, Name::from(name)))
    }

    pub(crate) fn tupled(elements: Vec<Resolved>, selfish: Nature) -> Resolved {
        let unit = elements.first().map(|_| ());
        unit.map_or_else(
            || Resolved::Found(Endpoint::from(Unnamed::Unit)),
            |()| {
                elements
                    .into_iter()
                    .find(|element| element.nature(selfish) == Nature::Primitive)
                    .unwrap_or_else(|| Resolved::Found(Endpoint::from(Unnamed::Opaque)))
            },
        )
    }

    pub(crate) fn resolved_with<'found>(&'found self, selfish: &'found Endpoint) -> &'found Endpoint {
        match self {
            Resolved::Itself => selfish,
            Resolved::Found(found) => found,
        }
    }

    pub(crate) fn map_found(self, located: impl FnOnce(Endpoint) -> Endpoint) -> Resolved {
        match self {
            Resolved::Itself => Resolved::Itself,
            Resolved::Found(found) => Resolved::Found(located(found)),
        }
    }

    pub(crate) fn or_opaque(self) -> Endpoint {
        match self {
            Resolved::Itself => Endpoint::from(Unnamed::Opaque),
            Resolved::Found(found) => found,
        }
    }

    fn nature(&self, selfish: Nature) -> Nature {
        match self {
            Resolved::Itself => selfish,
            Resolved::Found(found) => found.nature(),
        }
    }
}
