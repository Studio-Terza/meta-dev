use lint_layout::Name;

use crate::endpoint::Endpoint;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Filler {
    Free,
    Inherent,
    Implemented(Name),
    Provided(Name),
}

impl Filler {
    #[must_use]
    pub fn trait_name(&self) -> Option<&Name> {
        match self {
            Filler::Implemented(name) | Filler::Provided(name) => Some(name),
            Filler::Free | Filler::Inherent => None,
        }
    }

    pub(crate) fn implementer(&self, selfish: &Endpoint) -> Option<Endpoint> {
        match self {
            Filler::Implemented(_) => Some(selfish.clone()),
            Filler::Free | Filler::Inherent | Filler::Provided(_) => None,
        }
    }
}
