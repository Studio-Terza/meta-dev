use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Prefix;

impl Matching for Prefix {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.without_prefix(word).map(|_| word)
    }
}
