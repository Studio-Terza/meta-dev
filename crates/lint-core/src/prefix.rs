use crate::matching::Matching;
use crate::name::Name;

#[derive(Debug)]
pub struct Prefix;

impl Matching for Prefix {
    fn holds(name: &Name, word: &str) -> bool {
        name.spelled().get(..word.len()) == Some(word)
    }
}
