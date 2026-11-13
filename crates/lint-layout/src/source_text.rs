use newtype::{Arbitrary, Newtype, Wrapping};
use toml::Spanned;

use crate::line_number::LineNumber;
use crate::word::Word;

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub struct SourceText(String);

impl AsRef<str> for SourceText {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl SourceText {
    pub(crate) fn line_of<Key>(&self, key: &Spanned<Key>) -> LineNumber {
        LineNumber::from(self.0.get(..key.span().start).unwrap_or_default().matches('\n').count() + 1)
    }

    #[must_use]
    pub fn mentioning(&self, words: &[Word<'_>]) -> Option<&SourceText> {
        words.iter().find(|word| self.0.contains(word.into_inner())).map(|_| self)
    }
}
