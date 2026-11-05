use lint_layout::{Name, Word};

use super::super::super::unit::Unit;
use super::super::place::Place;
use super::super::target::Target;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Head<'found> {
    Root,
    Itself,
    Parent,
    Local,
    Krate(Unit<'found>),
    External,
    Own,
    Unknown,
}

impl<'found> Head<'found> {
    const PARENT: Word<'static> = Word::new("super");
    pub(super) const KEYWORDS: [(Word<'static>, Head<'static>); 3] =
        [(Word::new("crate"), Head::Root), (Word::new("self"), Head::Itself), (Self::PARENT, Head::Parent)];

    pub(super) fn target(self, place: &Place<'found>, path: &[Name]) -> Option<Target<'found>> {
        match self {
            Head::Root => path.split_first().map(|(_, rest)| place.target(Some(rest.to_vec()))),
            Head::Itself => path.split_first().map(|(_, rest)| place.within(place.module(), rest)),
            Head::Parent => Some(Self::parented(place, path)),
            Head::Local => Some(place.within(place.module(), path)),
            Head::Krate(unit) => path.split_first().map(|(_, rest)| Target::new(unit, Some(rest.to_vec()), None)),
            Head::Own => Some(place.target(None)),
            Head::External | Head::Unknown => None,
        }
    }

    fn parented(place: &Place<'found>, path: &[Name]) -> Target<'found> {
        let here = place.module();
        let parents = path.iter().take_while(|step| **step == Self::PARENT).count();
        let base = here.get(..here.len().saturating_sub(parents)).unwrap_or_default();
        place.within(base, path.get(parents..).unwrap_or_default())
    }
}
