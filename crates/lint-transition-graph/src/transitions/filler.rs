use graph::{Domain, Implementation, ItemPath, Model, Resolved};
use lint_layout::{Name, Word, words};

use crate::transitions::endpoint::Endpoint;
use crate::transitions::implementer::Implementer;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Origin {
    Workspace,
    Foreign,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Declares {
    Nothing,
    Semigroup,
    Action,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Implemented {
    name: Name,
    declares: Declares,
    origin: Origin,
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
    const ACT: [Word<'static>; 2] = words!["algebra", "Act"];

    pub(crate) fn read(model: &Model, implementation: &Implementation) -> Option<Implemented> {
        let name = implementation.interface_name()?.clone();
        let matched = |path: &[Word<'static>], kind: Declares| {
            let exported = model.exported(Domain::Types, &path.iter().copied().collect::<ItemPath>());
            exported.filter(|found| implementation.implemented() == Some(*found)).map(|_| kind)
        };
        let declares = matched(&Self::SEMIGROUP, Declares::Semigroup)
            .or_else(|| matched(&Self::ACT, Declares::Action))
            .unwrap_or(Declares::Nothing);
        let origin =
            implementation.implemented().and_then(Resolved::crate_item).map_or(Origin::Foreign, |_| Origin::Workspace);
        Some(Implemented { name, declares, origin })
    }
}

#[derive(Debug)]
pub enum Subjected {
    Subject(Endpoint),
    Bare(Endpoint),
}

impl Subjected {
    #[must_use]
    pub fn endpoint(&self) -> &Endpoint {
        match self {
            Subjected::Subject(endpoint) | Subjected::Bare(endpoint) => endpoint,
        }
    }

    #[must_use]
    pub fn subject(&self) -> Option<&Endpoint> {
        match self {
            Subjected::Subject(endpoint) => Some(endpoint),
            Subjected::Bare(_) => None,
        }
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

    pub(crate) fn subjected(&self, selfish: Endpoint) -> Subjected {
        match self {
            Filler::Free => Subjected::Bare(selfish),
            Filler::Inherent | Filler::Implemented(_) | Filler::Provided => Subjected::Subject(selfish),
        }
    }
}

impl Filler {
    pub(crate) fn foreign_trait(&self) -> Option<&Name> {
        match self {
            Filler::Implemented(implemented) => implemented.foreign(),
            Filler::Free | Filler::Inherent | Filler::Provided => None,
        }
    }
}

impl Implemented {
    fn foreign(&self) -> Option<&Name> {
        match self.origin {
            Origin::Foreign => Some(&self.name),
            Origin::Workspace => None,
        }
    }
}
