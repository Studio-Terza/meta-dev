use lint_layout::Name;

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
            Filler::Implemented(name) => Some(name),
            Filler::Provided => Some(selfish),
            Filler::Free | Filler::Inherent => None,
        }
    }

    pub(crate) fn owned(&self) -> Option<&Filler> {
        match self {
            Filler::Free => None,
            Filler::Inherent | Filler::Implemented(_) | Filler::Provided => Some(self),
        }
    }
}
