use newtype::Wrapping;

use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct Whole;

impl Matching for Whole {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        (AsRef::<str>::as_ref(name) == word.into_inner()).then_some(word)
    }
}
