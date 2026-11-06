use lint_layout::Name;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ItemPath {
    segments: Vec<Name>,
}

impl FromIterator<Name> for ItemPath {
    fn from_iter<Segments: IntoIterator<Item = Name>>(segments: Segments) -> ItemPath {
        ItemPath { segments: segments.into_iter().collect() }
    }
}

impl AsRef<[Name]> for ItemPath {
    fn as_ref(&self) -> &[Name] {
        &self.segments
    }
}

impl ItemPath {
    #[must_use]
    pub fn last(&self) -> Option<&Name> {
        self.segments.last()
    }
}
