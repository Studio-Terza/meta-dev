use newtype::Wrapping;

use crate::matching::{Fit, Matching};
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds(name: &Name, word: Word<'_>) -> Fit {
        Fit::from(Some(name).filter(|spelled| AsRef::<str>::as_ref(*spelled) == word.into_inner()))
    }
}
