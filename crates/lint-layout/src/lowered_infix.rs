use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct LoweredInfix;

impl Matching for LoweredInfix {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.lowered().contains(word)
    }
}
