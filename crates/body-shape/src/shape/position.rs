#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Position {
    Named,
    BeforeLiteral,
    Plain,
}

impl Position {
    pub(crate) fn plain(self) -> Option<Position> {
        match self {
            Position::Named => None,
            Position::BeforeLiteral | Position::Plain => Some(self),
        }
    }
}
