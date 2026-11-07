use super::Quote;

impl Quote {
    const TRANSITIONS: [(Self, char, Self); 4] = [
        (Self::Outside, '\'', Self::Single),
        (Self::Outside, '"', Self::Double),
        (Self::Single, '\'', Self::Outside),
        (Self::Double, '"', Self::Outside),
    ];

    pub(in crate::document::command::unquoted) fn after(self, letter: char) -> Self {
        Self::TRANSITIONS
            .into_iter()
            .filter(|(from, _, _)| *from == self)
            .find(|(_, mark, _)| *mark == letter)
            .map_or(self, |(_, _, to)| to)
    }
}
