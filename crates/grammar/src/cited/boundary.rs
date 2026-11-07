#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cited {
    prefix: String,
    ordinal: usize,
}

impl Cited {
    pub(crate) fn read(cited: &str) -> Option<Cited> {
        let dot = cited.char_indices().find(|(_, character)| *character == '.').map(|(dot, _)| dot)?;
        let ordinal = cited.get(dot.saturating_add(1)..)?.parse().ok()?;
        cited.get(..dot).map(|prefix| Cited { prefix: prefix.to_owned(), ordinal })
    }

    #[must_use]
    pub fn prefix(&self) -> &str {
        &self.prefix
    }

    #[must_use]
    pub fn ordinal(&self) -> usize {
        self.ordinal
    }
}
