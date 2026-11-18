use std::fmt;

use crate::name::Name;
use crate::word::Word;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fitted<'word>(Word<'word>);

impl<'word> From<Word<'word>> for Fitted<'word> {
    fn from(word: Word<'word>) -> Fitted<'word> {
        Fitted(word)
    }
}

impl fmt::Display for Fitted<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

pub trait Matching {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Fitted<'word>>;
}
