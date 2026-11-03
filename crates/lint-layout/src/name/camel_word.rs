mod boundary;

use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;
use boundary::Letters;

#[derive(Debug)]
pub struct CamelWord;

impl Matching for CamelWord {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.words()
            .iter()
            .flat_map(|chunk| Letters::humps(&chunk.0).into_iter().map(Name))
            .find_map(|hump| (hump.lowered() == word).then_some(word))
    }
}
