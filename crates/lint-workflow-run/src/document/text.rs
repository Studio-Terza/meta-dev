mod expansion;

use std::fmt;
use std::path::Path;

use grammar_program::Program;
use newtype::{Newtype, Wrapping};

use crate::characters::Characters;
use crate::document::lines::Lines;
use crate::excerpt::Excerpt;
use crate::word::Word;
use expansion::Expansion;

#[derive(Newtype, Clone, Copy, Debug, Default)]
pub struct Text<'text>(&'text str);

impl<'text> Text<'text> {
    pub(crate) const EXPANSION: Word = Word::new("${");

    pub(crate) fn after(self, prefix: Text<'_>) -> Option<Text<'text>> {
        self.0.get(..prefix.0.len()).filter(|start| *start == prefix.0).and(self.0.get(prefix.0.len()..)).map(Text)
    }

    pub(crate) fn leading(self, word: Word) -> Option<Word> {
        self.after(Text::from(word)).filter(|rest| rest.0.chars().next().is_none_or(char::is_whitespace)).map(|_| word)
    }

    pub(crate) fn modified_expansion(self) -> Option<Word> {
        let expansion = Self::EXPANSION.into_inner();
        self.0
            .match_indices(expansion)
            .find_map(|(at, found)| {
                self.0.get(at + found.len()..).map(Text).and_then(|rest| Expansion::try_from(rest).ok())
            })
            .map(|_| Self::EXPANSION)
    }

    pub(crate) fn first_word_outside(self, known: &[Word]) -> Option<Excerpt> {
        self.0
            .parse::<Program>()
            .ok()
            .map(Program::into_inner)
            .filter(|word| known.iter().all(|shell| shell.into_inner() != *word))
            .map(Excerpt::new)
    }

    pub(crate) fn count(self) -> Characters {
        Characters::new(self.0.chars().count())
    }

    pub(crate) fn trimmed(self) -> Text<'text> {
        let leading: usize = self.0.chars().take_while(|letter| letter.is_whitespace()).map(char::len_utf8).sum();
        let blanks_gone = self.0.get(leading..).unwrap_or_default();
        let trailing: usize =
            blanks_gone.chars().rev().take_while(|letter| letter.is_whitespace()).map(char::len_utf8).sum();
        Text(blanks_gone.get(..blanks_gone.len() - trailing).unwrap_or_default())
    }

    pub(crate) fn path(self) -> &'text Path {
        Path::new(self.0)
    }

    pub(crate) fn lines(self) -> Lines {
        Lines::new(self.0.chars().filter(|letter| *letter == '\n').count() + 1)
    }
}

impl From<Word> for Text<'static> {
    fn from(word: Word) -> Text<'static> {
        Text(word.into_inner())
    }
}

impl From<Text<'_>> for Excerpt {
    fn from(text: Text<'_>) -> Excerpt {
        Excerpt::new(text.0.to_owned())
    }
}

impl fmt::Display for Text<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}
