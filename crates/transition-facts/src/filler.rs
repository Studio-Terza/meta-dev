use lint_layout::Name;

use crate::endpoint::Endpoint;
use crate::implementer::Implementer;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Filler {
    Free,
    Inherent,
    Implemented(Name),
    Provided,
}

impl Filler {
    pub(crate) fn trait_name<'name>(&'name self, selfish: &'name Name) -> Option<&'name Name> {
        match self {
            Self::Implemented(name) => Some(name),
            Self::Provided => Some(selfish),
            Self::Free | Self::Inherent => None,
        }
    }

    pub(crate) fn implementer<'selfish>(&self, selfish: &'selfish Endpoint) -> Option<Implementer<'selfish>> {
        match self {
            Self::Implemented(_) => Some(Implementer::new(selfish)),
            Self::Free | Self::Inherent | Self::Provided => None,
        }
    }

    pub(crate) fn owned(&self) -> Option<&Self> {
        match self {
            Self::Free => None,
            Self::Inherent | Self::Implemented(_) | Self::Provided => Some(self),
        }
    }
}
