use crate::matching::Matching;
use crate::name::Name;

#[derive(Debug)]
pub struct LoweredInfix;

impl Matching for LoweredInfix {
    fn holds(name: &Name, word: &str) -> bool {
        name.lowered().contains(word)
    }
}
