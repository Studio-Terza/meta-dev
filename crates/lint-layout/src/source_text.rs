use newtype::{Arbitrary, Newtype};
use toml::Spanned;

use crate::line_number::LineNumber;

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
}
