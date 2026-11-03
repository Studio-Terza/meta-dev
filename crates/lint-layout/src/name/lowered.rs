use crate::word::Word;
use newtype::Newtype;

#[derive(Debug, PartialEq, Eq)]
pub struct Lowered(String);

impl FromIterator<char> for Lowered {
    fn from_iter<Letters: IntoIterator<Item = char>>(letters: Letters) -> Lowered {
        Lowered(letters.into_iter().collect())
    }
}

impl PartialEq<Word<'_>> for Lowered {
    fn eq(&self, word: &Word<'_>) -> bool {
        self.0 == word.into_inner()
    }
}
