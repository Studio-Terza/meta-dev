use crate::matching::Matching;
use crate::name::Name;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds(name: &Name, word: &str) -> bool {
        name.spelled() == word
    }
}
