use std::convert::Infallible;
use std::str::FromStr;

use crate::characters::Characters;
use crate::finding::Finding;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum RunStep {
    Absent,
    Block,
    Inline(String),
}

impl RunStep {
    const KEY: &'static str = "run:";
    const LIST_MARK: &'static str = "- ";
    const BLOCK_MARKS: [char; 2] = ['|', '>'];
    const QUOTES: [char; 2] = ['\'', '"'];
    const LIMIT: Characters = Characters::new(120);
    const OPERATORS: [&'static str; 7] = ["|", "&", ";", ">", "<", "`", "$("];
    const KEYWORDS: [&'static str; 7] = ["if", "for", "while", "until", "case", "[", "[["];

    pub(crate) fn finding(&self) -> Option<Finding> {
        match self {
            RunStep::Absent => None,
            RunStep::Block => Some(Finding::Block),
            RunStep::Inline(command) => Self::inline_finding(command),
        }
    }

    fn inline_finding(command: &str) -> Option<Finding> {
        let characters = command.parse::<Characters>().ok()?;
        let operator = Self::OPERATORS.iter().find(|symbol| command.contains(**symbol));
        let keyword =
            command.split_whitespace().next().and_then(|first| Self::KEYWORDS.iter().find(|word| **word == first));
        if characters > Self::LIMIT {
            Some(Finding::TooLong { characters, limit: Self::LIMIT })
        } else if let Some(symbol) = operator {
            Some(Finding::Operator { symbol })
        } else {
            keyword.map(|word| Finding::Control { word })
        }
    }

    fn unquoted(value: &str) -> &str {
        Self::QUOTES
            .iter()
            .find_map(|quote| value.strip_prefix(*quote).and_then(|inner| inner.strip_suffix(*quote)))
            .unwrap_or(value)
    }
}

impl FromStr for RunStep {
    type Err = Infallible;

    fn from_str(line: &str) -> Result<RunStep, Infallible> {
        let start = line.trim_start();
        let body = start.strip_prefix(Self::LIST_MARK).unwrap_or(start).trim_start();
        Ok(match body.strip_prefix(Self::KEY).map(str::trim) {
            None => RunStep::Absent,
            Some(value) if value.is_empty() || value.starts_with(Self::BLOCK_MARKS) => RunStep::Block,
            Some(value) => RunStep::Inline(Self::unquoted(value).to_owned()),
        })
    }
}
