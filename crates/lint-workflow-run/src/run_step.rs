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
    const KEYWORDS: [&'static str; 7] = ["if", "for", "while", "until", "case", "[", "[["];

    pub(crate) fn finding(&self, next: Option<&str>) -> Option<Finding> {
        match self {
            RunStep::Absent => None,
            RunStep::Block => Some(Finding::Block),
            RunStep::Opening => Text::new(next.unwrap_or_default()).names_key().map_or(Some(Finding::Block), |_| None),
            RunStep::Inline(command) => Self::inline_finding(command),
        }
    }

    fn inline_finding(command: &str) -> Option<Finding> {
        let characters = command.parse::<Characters>().ok()?;
        let operator = Self::OPERATORS.iter().find(|symbol| command.contains(**symbol));
        let keyword = Self::KEYWORDS.iter().find(|word| {
            Text::new(command)
                .after(Text::new(word))
                .is_some_and(|rest| rest.first_letter().is_none_or(char::is_whitespace))
        });
        if characters > Self::LIMIT {
            Some(Finding::TooLong { characters, limit: Self::LIMIT })
        } else if let Some(symbol) = operator {
            Some(Finding::Operator { symbol })
        } else {
            keyword.map(|word| Finding::Control { word })
        }
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
