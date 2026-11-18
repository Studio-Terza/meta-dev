use crate::matching::{Fit, Matching};
use crate::name::Name;
use crate::word::Word;
use newtype::Wrapping;

#[derive(Debug)]
pub struct Prefix;

impl Matching for Prefix {
    fn holds(name: &Name, word: Word<'_>) -> Fit {
        let spelled = word.into_inner();
        Fit::from(name.0.get(..spelled.len()).filter(|head| *head == spelled))
    }
}
