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
