use algebra::{Monoid, Semigroup};
use lint_layout::{Name, PathTail, Word};
use newtype::Arbitrary;
use newtype::Wrapping;

#[derive(Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment<'name>(&'name Name);

impl<'name> From<&'name Name> for Segment<'name> {
    fn from(name: &'name Name) -> Segment<'name> {
        Segment(name)
    }
}

impl From<Segment<'_>> for ItemPath {
    fn from(segment: Segment<'_>) -> ItemPath {
        ItemPath { segments: vec![segment.0.clone()] }
    }
}

impl Semigroup for ItemPath {
    fn combine(self, other: ItemPath) -> ItemPath {
        ItemPath { segments: self.segments.combine(other.segments) }
    }
}

impl Monoid for ItemPath {}

impl ItemPath {
    #[must_use]
    pub fn last(&self) -> Option<&Name> {
        self.segments.last()
    }

    #[must_use]
    pub fn ends_with(&self, tail: PathTail<'_>) -> Option<&ItemPath> {
        let words = tail.into_inner();
        let start = self.segments.len().checked_sub(words.len())?;
        let ending = self.segments.get(start..)?;
        let mismatch =
            ending.iter().zip(words).find(|(segment, word)| AsRef::<str>::as_ref(*segment) != word.into_inner());
        mismatch.map_or(Some(self), |_| None)
    }
}

impl AsRef<[Name]> for ItemPath {
    fn as_ref(&self) -> &[Name] {
        &self.segments
    }
}
