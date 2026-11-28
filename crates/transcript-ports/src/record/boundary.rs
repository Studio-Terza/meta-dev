mod entry;

use crate::boundary::LossyText;
use grammar_lines::Lines;
use invariants::Ignoring;

use crate::{Ignored, Line, Unrecognised};
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
        serde_json::from_str::<serde_json::Value>(line)
            .and_then(|value| Entry::read(&value))
            .ignoring(Ignored::Unparsed)
            .map_or_else(|| Some(Line::Unrecognised(Unrecognised::unreadable())), Option::<Line>::from)
    }
}
