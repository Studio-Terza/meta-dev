mod camel_word;
mod lowered;
mod prefix;

use std::ffi::OsStr;
use std::fmt;
use std::path::Path;

use syn::Ident;

use crate::casing::Casing;
use crate::length::Length;
use crate::matching::Matching;
use crate::word::Word;
pub use camel_word::CamelWord;
pub use lowered::Lowered;
use newtype::Newtype;
use newtype::proptest::arbitrary::{Arbitrary, any};
use newtype::proptest::strategy::{BoxedStrategy, Strategy};
pub use prefix::Prefix;

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Name(String);

impl Name {
    #[must_use]
    pub fn length(&self) -> Length {
        Length::new(self.0.chars().count())
    }

    #[must_use]
    pub fn matches<'word, Rule: Matching>(&self, words: &[Word<'word>]) -> Option<Word<'word>> {
        words.iter().find_map(|word| Rule::holds(self, *word))
    }

    #[must_use]
    pub fn lowered(&self) -> Lowered {
        self.0.chars().flat_map(char::to_lowercase).collect()
    }

    #[must_use]
    pub fn reprefixed(&self, opening: Word<'_>, closing: Word<'_>) -> Option<Name> {
        let spelled = opening.into_inner();
        self.0
            .get(..spelled.len())
            .filter(|head| *head == spelled)
            .and_then(|_| self.0.get(spelled.len()..))
            .map(|rest| Name([closing.into_inner(), rest].concat()))
    }

    #[must_use]
    pub fn words(&self) -> Vec<Name> {
        let letters: Vec<char> = self.0.chars().collect();
        letters
            .chunk_by(|left, right| left.is_alphanumeric() == right.is_alphanumeric())
            .filter(|chunk| chunk.iter().all(|letter| letter.is_alphanumeric()))
            .map(|chunk| Name(chunk.iter().collect()))
            .collect()
    }

    #[must_use]
    pub fn casing(&self) -> Casing {
        self.0.chars().next().map_or(Casing::Other, |first| self.cased(Initial::from(first)))
    }

    fn cased(&self, initial: Initial) -> Casing {
        match initial {
            Initial::Lower => Casing::Lower,
            Initial::Upper => self
                .0
                .chars()
                .skip(1)
                .find(char::is_ascii_lowercase)
                .or_else(|| self.0.chars().skip(1).find(char::is_ascii_digit))
                .map_or(Casing::Shouting, |_| Casing::Camel),
            Initial::Other => Casing::Other,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Initial {
    Lower,
    Upper,
    Other,
}

impl From<char> for Initial {
    fn from(first: char) -> Initial {
        Some(first)
            .filter(char::is_ascii_lowercase)
            .map(|_| Initial::Lower)
            .or_else(|| Some(first).filter(char::is_ascii_uppercase).map(|_| Initial::Upper))
            .unwrap_or(Initial::Other)
    }
}
