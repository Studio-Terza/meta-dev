use super::Quote;

impl Quote {
    const TRANSITIONS: [(Quote, char, Quote); 4] = [
        (Quote::Outside, '\'', Quote::Single),
        (Quote::Outside, '"', Quote::Double),
        (Quote::Single, '\'', Quote::Outside),
        (Quote::Double, '"', Quote::Outside),
    ];

    pub(in crate::document::command::unquoted) fn after(self, letter: char) -> Quote {
        Self::TRANSITIONS
            .into_iter()
            .filter(|(from, _, _)| *from == self)
            .find(|(_, mark, _)| *mark == letter)
            .map_or(self, |(_, _, to)| to)
    }
}
