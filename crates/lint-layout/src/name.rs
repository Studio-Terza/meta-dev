use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use syn::Ident;

use crate::casing::Casing;
use crate::matching::Matching;
use crate::whole::Whole;
use crate::word::Word;
use newtype::Newtype;
use newtype::proptest::arbitrary::{Arbitrary, any};
use newtype::proptest::strategy::{BoxedStrategy, Strategy};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    #[must_use]
    pub fn length(&self) -> usize {
        self.0.chars().count()
    }

    #[must_use]
    pub fn matches<'word, Rule: Matching>(&self, words: &[Word<'word>]) -> Option<Word<'word>> {
        words.iter().find_map(|word| Rule::holds(self, *word))
    }

    pub(crate) fn spelled(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn lowered(&self) -> Name {
        Name(self.0.chars().flat_map(char::to_lowercase).collect())
    }

    #[must_use]
    pub fn contains<'word>(&self, word: Word<'word>) -> Option<Word<'word>> {
        self.0.contains(word.into_inner()).then_some(word)
    }

    #[must_use]
    pub fn prefixed(&self, word: Word<'_>) -> Name {
        Name([word.into_inner(), &self.0].concat())
    }

    #[must_use]
    pub fn without_prefix(&self, word: Word<'_>) -> Option<Name> {
        let spelled = word.into_inner();
        self.0
            .get(..spelled.len())
            .filter(|head| *head == spelled)
            .and_then(|_| self.0.get(spelled.len()..))
            .map(Name::from)
    }

    #[must_use]
    pub fn words(&self) -> Vec<Name> {
        let letters: Vec<char> = self.0.chars().collect();
        letters
            .chunk_by(|left, right| left.is_alphanumeric() && right.is_alphanumeric())
            .filter(|chunk| chunk.iter().all(|letter| letter.is_alphanumeric()))
            .map(|chunk| Name(chunk.iter().collect()))
            .collect()
    }

    #[must_use]
    pub fn camel_word<'word>(&self, words: &[Word<'word>]) -> Option<Word<'word>> {
        self.words()
            .iter()
            .flat_map(|chunk| Self::humps(&chunk.0))
            .find_map(|hump| hump.lowered().matches::<Whole>(words))
    }

    fn humps(chunk: &str) -> Vec<Name> {
        let letters: Vec<char> = chunk.chars().collect();
        let cuts = (0..letters.len()).filter(|index| Self::cut(&letters, *index).is_some());
        let bounds: Vec<usize> = cuts.chain(Some(letters.len())).collect();
        bounds
            .iter()
            .zip(bounds.iter().skip(1))
            .filter_map(|(start, end)| letters.get(*start..*end))
            .map(|hump| Name(hump.iter().collect()))
            .collect()
    }

    fn cut(letters: &[char], index: usize) -> Option<&char> {
        let at = |place: usize| letters.get(place).copied();
        let previous = index.checked_sub(1).and_then(at);
        let next = index.checked_add(1).and_then(at);
        (previous.is_none()
            || (at(index).is_some_and(char::is_uppercase)
                && previous.is_some_and(|before| {
                    before.is_lowercase()
                        || before.is_ascii_digit()
                        || (before.is_uppercase() && next.is_some_and(char::is_lowercase))
                })))
        .then(|| letters.get(index))
        .flatten()
    }

    #[must_use]
    pub fn casing(&self) -> Casing {
        match self.0.chars().next() {
            Some(first) if first.is_ascii_lowercase() => Casing::Lower,
            Some(first) if first.is_ascii_uppercase() => Self::capitalized(&self.0),
            _ => Casing::Other,
        }
    }

    fn capitalized(text: &str) -> Casing {
        if text.chars().skip(1).any(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit()) {
            Casing::Camel
        } else {
            Casing::Shouting
        }
    }
}

impl From<&str> for Name {
    fn from(text: &str) -> Name {
        Name(text.to_owned())
    }
}

impl PartialEq<Word<'_>> for Name {
    fn eq(&self, word: &Word<'_>) -> bool {
        self.0 == word.into_inner()
    }
}

impl PartialEq<&str> for Name {
    fn eq(&self, text: &&str) -> bool {
        self.0 == *text
    }
}

impl AsRef<Path> for Name {
    fn as_ref(&self) -> &Path {
        Path::new(&self.0)
    }
}

impl PartialEq<Ident> for Name {
    fn eq(&self, ident: &Ident) -> bool {
        *ident == self.0
    }
}

impl From<Word<'_>> for Name {
    fn from(word: Word<'_>) -> Name {
        Name(word.into_inner().to_owned())
    }
}

impl From<&Ident> for Name {
    fn from(ident: &Ident) -> Name {
        Name(ident.to_string())
    }
}

impl From<&OsStr> for Name {
    fn from(segment: &OsStr) -> Name {
        Name(segment.to_string_lossy().into_owned())
    }
}

impl fmt::Display for Name {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Arbitrary for Name {
    type Parameters = ();
    type Strategy = BoxedStrategy<Name>;

    fn arbitrary_with((): ()) -> BoxedStrategy<Name> {
        any::<String>().prop_map(Name).boxed()
    }
}
