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
    pub fn implemented(&self) -> Option<&Name> {
        match self {
            Filler::Implemented(name) => Some(name),
            Filler::Free | Filler::Inherent | Filler::Provided(_) => None,
        }
    }
}
