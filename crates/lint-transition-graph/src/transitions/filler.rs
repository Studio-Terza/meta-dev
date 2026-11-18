use graph::{Domain, Implementation, ItemPath, Model};
use lint_layout::{Name, Word, words};

use crate::transitions::endpoint::Endpoint;
use crate::transitions::implementer::Implementer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Declares {
    Nothing,
    Semigroup,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Implemented {
    name: Name,
    declares: Declares,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Filler {
    Free,
    Inherent,
    Implemented(Implemented),
    Provided,
}

impl Implemented {
    const SEMIGROUP: [Word<'static>; 2] = words!["algebra", "Semigroup"];

    pub(crate) fn read(model: &Model, implementation: &Implementation) -> Option<Implemented> {
        let name = implementation.interface_name()?.clone();
        let semigroup: ItemPath = Self::SEMIGROUP.into_iter().collect();
        let exported = model.exported(Domain::Types, &semigroup);
        let declares = exported
            .filter(|found| implementation.implemented() == Some(*found))
            .map_or(Declares::Nothing, |_| Declares::Semigroup);
        Some(Implemented { name, declares })
    }
}

impl Filler {
    pub(crate) fn trait_name<'name>(&'name self, selfish: &'name Name) -> Option<&'name Name> {
        match self {
            Filler::Implemented(implemented) => Some(&implemented.name),
            Filler::Provided => Some(selfish),
            Filler::Free | Filler::Inherent => None,
        }
    }

    pub(crate) fn declares(&self) -> Declares {
        match self {
            Filler::Implemented(implemented) => implemented.declares,
            Filler::Free | Filler::Inherent | Filler::Provided => Declares::Nothing,
        }
    }

    pub(crate) fn implementer<'selfish>(&self, selfish: &'selfish Endpoint) -> Option<Implementer<'selfish>> {
        match self {
            Filler::Implemented(_) => Some(Implementer::new(selfish)),
            Filler::Free | Filler::Inherent | Filler::Provided => None,
        }
    }

    pub(crate) fn owned(&self) -> Option<&Filler> {
        match self {
            Filler::Free => None,
            Filler::Inherent | Filler::Implemented(_) | Filler::Provided => Some(self),
        }
    }
}
