use lint_layout::{Name, Word};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemPath {
    segments: Vec<Name>,
}

impl FromIterator<Name> for ItemPath {
    fn from_iter<Segments: IntoIterator<Item = Name>>(segments: Segments) -> ItemPath {
        ItemPath { segments: segments.into_iter().collect() }
    }
}

impl<'word> FromIterator<Word<'word>> for ItemPath {
    fn from_iter<Words: IntoIterator<Item = Word<'word>>>(words: Words) -> ItemPath {
        ItemPath { segments: words.into_iter().map(Name::from).collect() }
    }
}

impl ItemPath {
    #[must_use]
    pub fn joined(&self, segment: &Name) -> ItemPath {
        self.segments.iter().chain([segment]).cloned().collect()
    }

    #[must_use]
    pub fn last(&self) -> Option<&Name> {
        self.segments.last()
    }
}
