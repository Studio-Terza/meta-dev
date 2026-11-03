mod boundary;

use crate::matching::Matching;
use crate::name::Name;
use crate::word::Word;
use boundary::Letters;

#[derive(Debug)]
pub struct CamelWord;

impl Name {
    pub fn humps(&self) -> impl Iterator<Item = Name> {
        self.words().into_iter().flat_map(|chunk| Letters::humps(&chunk.0)).map(|hump| Name::from(Name(hump).lowered()))
    }
}

impl Matching for CamelWord {
    fn holds<'word>(name: &Name, word: Word<'word>) -> Option<Word<'word>> {
        name.humps().find_map(|hump| (hump == word).then_some(word))
    }
}
