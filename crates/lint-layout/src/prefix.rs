use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Prefix;

impl Matching for Prefix {
    fn holds(name: &Name, word: Word<'_>) -> bool {
        name.without_prefix(word).is_some()
    }
}
