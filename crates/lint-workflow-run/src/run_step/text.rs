use std::fmt;

#[derive(Clone, Copy, Debug)]
pub struct Text<'text>(&'text str);

impl<'text> Text<'text> {
    pub(crate) const fn new(text: &'text str) -> Text<'text> {
        Text(text)
    }

    pub(crate) fn after_blanks(self) -> Text<'text> {
        Text(
            self.0
                .char_indices()
                .find(|(_, letter)| !letter.is_whitespace())
                .and_then(|(at, _)| self.0.get(at..))
                .unwrap_or_default(),
        )
    }

    pub(crate) fn after(self, prefix: Text<'_>) -> Option<Text<'text>> {
        self.0.get(..prefix.0.len()).filter(|start| *start == prefix.0).and(self.0.get(prefix.0.len()..)).map(Text)
    }

    pub(crate) fn names_key(self) -> Option<Text<'text>> {
        let start = self.after_blanks();
        let end = start.0.char_indices().find(|(_, letter)| letter.is_whitespace()).map_or(start.0.len(), |(at, _)| at);
        start.0.get(..end).filter(|word| word.char_indices().last().is_some_and(|(_, last)| last == ':')).map(Text)
    }

    pub(crate) fn first_letter(self) -> Option<char> {
        self.0.chars().next()
    }

    pub(crate) fn unquoted(self, quotes: &[char]) -> Text<'text> {
        self.0
            .chars()
            .next()
            .zip(self.0.chars().last())
            .filter(|(opening, closing)| opening == closing && quotes.contains(opening))
            .and_then(|(opening, closing)| self.0.get(opening.len_utf8()..self.0.len() - closing.len_utf8()))
            .map_or(self, Text)
    }
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
