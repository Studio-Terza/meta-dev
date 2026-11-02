use crate::name::Name;
use crate::word::Word;

pub trait Matching {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>>;
}
