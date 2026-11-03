use lint_layout::Name;

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
}
