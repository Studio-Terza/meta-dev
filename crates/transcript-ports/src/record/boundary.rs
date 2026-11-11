mod entry;

use grammar_lines::Lines;
use text_lines::LossyText;

use crate::{Line, Unrecognised};
use entry::Entry;

impl Line {
    pub(crate) fn parsed(complete: &[u8]) -> Vec<Line> {
        LossyText::from(complete)
            .as_ref()
            .parse::<Lines>()
            .into_iter()
            .flatten()
            .filter(|line| line.chars().next().is_some())
            .filter_map(|line| Line::read(&line))
            .collect()
    }

    fn read(line: &str) -> Option<Line> {
        serde_json::from_str::<Entry>(line).map_or_else(|_refused| Line::unreadable(), Option::<Line>::from)
    }

    fn unreadable() -> Option<Line> {
        Unrecognised::unreadable().map(Line::Unrecognised)
    }
}
