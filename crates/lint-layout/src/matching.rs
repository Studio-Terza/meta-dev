use crate::name::Name;
use crate::word::Word;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Fitted<'word>(Word<'word>);

impl<'word> From<Word<'word>> for Fitted<'word> {
    fn from(word: Word<'word>) -> Fitted<'word> {
        Fitted(word)
    }
}

pub trait Matching {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Fitted<'word>>;
}
