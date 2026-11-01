use std::fmt;

use newtype::Newtype;

use crate::flagged::Flagged;

#[derive(Newtype, Debug, Default, PartialEq, Eq)]
pub struct Lines(Vec<String>);

impl PartialEq<Flagged> for Lines {
    fn eq(&self, flagged: &Flagged) -> bool {
        self.0.iter().map(String::as_str).eq(flagged.into_inner().iter().copied())
    }
}

impl<const WANTED: usize> PartialEq<[&str; WANTED]> for Lines {
    fn eq(&self, wanted: &[&str; WANTED]) -> bool {
        self.0 == *wanted
    }
}

impl fmt::Display for Lines {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?}", self.0)
    }
}
