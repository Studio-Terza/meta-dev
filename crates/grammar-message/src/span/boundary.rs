use newtype::Wrapping;

use super::Span;

impl Span {
    #[must_use]
    pub fn slice(self, text: &str) -> &str {
        text.get(self.start.into_inner()..self.end.into_inner()).unwrap_or_default()
    }
}
