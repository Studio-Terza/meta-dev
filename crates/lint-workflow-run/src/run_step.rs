mod text;

use std::convert::Infallible;
use std::str::FromStr;

use crate::characters::Characters;
use crate::finding::Finding;
use text::Text;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RunStep {
    Absent,
    Block,
    Opening,
    Inline(String),
}

impl RunStep {
    const KEY: Text<'static> = Text::new("run:");
    const LIST_MARK: Text<'static> = Text::new("- ");
    const BLOCK_MARKS: [char; 2] = ['|', '>'];
    const QUOTES: [char; 2] = ['\'', '"'];
    const LIMIT: Characters = Characters::new(120);
    const OPERATORS: [&'static str; 7] = ["|", "&", ";", ">", "<", "`", "$("];
    const OPENERS: [&'static str; 2] = ["!", "("];
    const COMMENT: char = '#';
    const KEYWORDS: [&'static str; 7] = ["if", "for", "while", "until", "case", "[", "[["];

    pub(crate) fn finding(&self, following: &[&str]) -> Option<Finding> {
        match self {
            RunStep::Absent => None,
            RunStep::Block => Some(Finding::Block),
            RunStep::Opening => Self::next_content(following).names_key().map_or(Some(Finding::Block), |_| None),
            RunStep::Inline(command) => Self::inline_finding(command),
        }
    }

    fn next_content<'lines>(following: &[&'lines str]) -> Text<'lines> {
        following
            .iter()
            .map(|line| Text::new(line).after_blanks())
            .find(|line| line.first_letter().is_some_and(|first| first != Self::COMMENT))
            .unwrap_or_default()
    }

    fn inline_finding(command: &str) -> Option<Finding> {
        let characters = command.parse::<Characters>().ok()?;
        (characters > Self::LIMIT)
            .then_some(Finding::TooLong { characters, limit: Self::LIMIT })
            .or_else(|| Self::operator_in(command).map(|symbol| Finding::Operator { symbol }))
            .or_else(|| Self::keyword_in(Text::new(command)).map(|word| Finding::Control { word }))
    }

    fn operator_in(command: &str) -> Option<&'static str> {
        Self::OPERATORS
            .iter()
            .copied()
            .find(|symbol| command.contains(*symbol))
            .or_else(|| Text::new(command).modified_expansion().map(|_| Text::EXPANSION))
    }

    fn keyword_in(text: Text<'_>) -> Option<&'static str> {
        Self::OPENERS
            .iter()
            .copied()
            .find(|word| text.after(Text::new(word)).is_some())
            .or_else(|| Self::KEYWORDS.iter().copied().find(|word| text.after_word(word).is_some()))
    }
}

impl FromStr for RunStep {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<RunStep, Infallible> {
        let start = Text::new(line).after_blanks();
        let body = start.after(Self::LIST_MARK).unwrap_or(start).after_blanks();
        Ok(match body.after(Self::KEY).map(Text::after_blanks) {
            None => RunStep::Absent,
            Some(value) if value.first_letter().is_none() => RunStep::Opening,
            Some(value) if value.first_letter().is_some_and(|first| Self::BLOCK_MARKS.contains(&first)) => {
                RunStep::Block
            }
            Some(value) => RunStep::Inline(value.unquoted(&Self::QUOTES).to_string()),
        })
    }
}
