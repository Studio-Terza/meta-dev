mod entry;

use crate::lossy_text::LossyText;
use grammar::Lines;
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
        serde_json::from_str::<Entry>(line)
            .ignoring(Ignored::Unparsed)
            .map_or_else(|| Some(Line::Unrecognised(Unrecognised::unreadable())), Option::<Line>::from)
    }
}
