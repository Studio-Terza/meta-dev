use lint_layout::Name;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TraitPath {
    segments: Vec<Name>,
}

impl FromIterator<Name> for TraitPath {
    fn from_iter<Segments: IntoIterator<Item = Name>>(segments: Segments) -> TraitPath {
        TraitPath { segments: segments.into_iter().collect() }
    }
}

impl TraitPath {
    #[must_use]
    pub fn last(&self) -> Option<&Name> {
        self.segments.last()
    }
}
