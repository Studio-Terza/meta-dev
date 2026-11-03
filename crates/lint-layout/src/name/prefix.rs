use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;
use newtype::Newtype;

#[derive(Debug)]
pub struct Prefix;

impl Matching for Prefix {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        let spelled = word.into_inner();
        name.0.get(..spelled.len()).filter(|head| *head == spelled).map(|_| word)
    }
}
