use newtype::Wrapping;

use crate::matching::{Fitted, Matching};
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Fitted<'word>> {
        (AsRef::<str>::as_ref(name) == word.into_inner()).then_some(Fitted::from(word))
    }
}
