use std::fmt;
use std::path::Path;

use crate::characters::Characters;

#[derive(Clone, Copy, Debug, Default)]
pub struct Text<'text>(&'text str);

impl<'text> Text<'text> {
    pub(crate) const EXPANSION: &'static str = "${";
    const OPEN: char = '{';
    const CLOSE: char = '}';

    pub(crate) const fn new(text: &'text str) -> Text<'text> {
        Text(text)
    }

    pub(crate) fn after(self, prefix: Text<'_>) -> Option<Text<'text>> {
        self.0.get(..prefix.0.len()).filter(|start| *start == prefix.0).and(self.0.get(prefix.0.len()..)).map(Text)
    }

    pub(crate) fn after_word(self, word: &str) -> Option<Text<'text>> {
        self.after(Text::new(word)).filter(|rest| rest.first_letter().is_none_or(char::is_whitespace))
    }

    pub(crate) fn modified_expansion(self) -> Option<&'text str> {
        self.0
            .match_indices(Self::EXPANSION)
            .find_map(|(at, found)| self.0.get(at + found.len()..).and_then(Self::modifier_in))
    }

    fn modifier_in(rest: &'text str) -> Option<&'text str> {
        let end = rest.match_indices(Self::CLOSE).next().map_or(rest.len(), |(at, _)| at);
        rest.get(..end)
            .filter(|_| Text(rest).first_letter() != Some(Self::OPEN))
            .filter(|name| name.chars().any(|letter| !letter.is_alphanumeric() && letter != '_'))
    }

    fn first_letter(self) -> Option<char> {
        self.0.chars().next()
    }

    pub(crate) fn first_word_outside(self, known: &[&str]) -> Option<&'text str> {
        let start = self.without_leading_blanks();
        let end = start.char_indices().find(|(_, letter)| letter.is_whitespace()).map_or(start.len(), |(at, _)| at);
        start.get(..end).filter(|word| !word.is_empty() && !known.contains(word))
    }

    pub(crate) fn count(self) -> Characters {
        Characters::new(self.0.chars().count())
    }

    pub(crate) fn first_inside(self, symbols: &[&'static str]) -> Option<&'static str> {
        symbols.iter().copied().find(|symbol| self.0.contains(*symbol))
    }

    pub(crate) fn trimmed(self) -> Text<'text> {
        let blanks_gone = self.without_leading_blanks();
        let end = blanks_gone
            .char_indices()
            .rev()
            .find(|(_, letter)| !letter.is_whitespace())
            .map_or(0, |(at, letter)| at + letter.len_utf8());
        Text(blanks_gone.get(..end).unwrap_or_default())
    }

    fn without_leading_blanks(self) -> &'text str {
        self.0
            .char_indices()
            .find(|(_, letter)| !letter.is_whitespace())
            .and_then(|(at, _)| self.0.get(at..))
            .unwrap_or_default()
    }

    pub(crate) fn path(self) -> &'text Path {
        Path::new(self.0)
    }

    pub(crate) fn spans_lines(self) -> bool {
        self.0.contains('\n')
    }
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
