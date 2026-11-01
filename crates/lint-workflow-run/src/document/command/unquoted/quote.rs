#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Quote {
    Outside,
    Single,
    Double,
}

impl Quote {
    pub(super) fn after(self, letter: char) -> Quote {
        match (self, letter) {
            (Quote::Outside, '\'') => Quote::Single,
            (Quote::Outside, '"') => Quote::Double,
            (Quote::Single, '\'') | (Quote::Double, '"') => Quote::Outside,
            _ => self,
        }
    }
}
