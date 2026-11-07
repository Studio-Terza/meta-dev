use lint_layout::{Name, Word};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ItemPath {
    segments: Vec<Name>,
}

impl FromIterator<Name> for ItemPath {
    fn from_iter<Segments: IntoIterator<Item = Name>>(segments: Segments) -> Self {
        Self { segments: segments.into_iter().collect() }
    }
}

impl<'word> FromIterator<Word<'word>> for ItemPath {
    fn from_iter<Words: IntoIterator<Item = Word<'word>>>(words: Words) -> Self {
        Self { segments: words.into_iter().map(Name::from).collect() }
    }
}

impl ItemPath {
    #[must_use]
    pub fn joined(&self, segment: &Name) -> Self {
        self.segments.iter().chain([segment]).cloned().collect()
    }

    #[must_use]
    pub fn last(&self) -> Option<&Name> {
        self.segments.last()
    }
}

impl AsRef<[Name]> for ItemPath {
    fn as_ref(&self) -> &[Name] {
        &self.segments
    }
}
