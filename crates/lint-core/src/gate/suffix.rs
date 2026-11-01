use crate::matching::Matching;
use crate::name::Name;

#[derive(Debug)]
pub struct Suffix;

impl Matching for Suffix {
    fn holds(name: &Name, word: &str) -> bool {
        name.spelled().len().checked_sub(word.len()).and_then(|start| name.spelled().get(start..)) == Some(word)
    }
}
