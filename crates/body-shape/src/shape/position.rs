#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Position {
    Named,
    BeforeLiteral,
    Plain,
}

impl Position {
    pub(crate) fn plain(self) -> Option<Self> {
        match self {
            Self::Named => None,
            Self::BeforeLiteral | Self::Plain => Some(self),
        }
    }
}
