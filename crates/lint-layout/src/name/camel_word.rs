use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;

#[derive(Debug)]
pub struct CamelWord;

impl Matching for CamelWord {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.humps().find_map(|hump| (*hump.as_ref() == word).then_some(word))
    }
}
