use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds(name: &Name, word: Word<'_>) -> bool {
        *name == word
    }
}
