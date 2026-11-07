use std::fmt;

use newtype::{Arbitrary, Newtype};

use super::Flagged;

#[derive(Newtype, Arbitrary, Debug, Default, PartialEq, Eq)]
pub struct Lines(Vec<String>);

impl From<Flagged> for Lines {
    fn from(flagged: Flagged) -> Lines {
        Lines(flagged.into_inner().iter().map(ToString::to_string).collect())
    }
}

impl<const WANTED: usize> From<[&str; WANTED]> for Lines {
    fn from(wanted: [&str; WANTED]) -> Lines {
        Lines(wanted.iter().map(ToString::to_string).collect())
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}
