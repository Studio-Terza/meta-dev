use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct LoweredInfix;

impl Matching for LoweredInfix {
    fn holds(name: &Name, word: Word<'_>) -> bool {
        name.lowered().contains(word)
    }
}
