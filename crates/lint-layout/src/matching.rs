use crate::name::Name;
use crate::word::Word;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fit {
    Fits,
    Misses,
}

impl<Found> From<Option<Found>> for Fit {
    fn from(found: Option<Found>) -> Fit {
        found.map_or(Fit::Misses, |_| Fit::Fits)
    }
}

pub trait Matching {
    fn holds(name: &Name, word: Word<'_>) -> Fit;
}
