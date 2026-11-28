mod entry;

use crate::lossy_text::LossyText;
use grammar::lines::Lines;
use invariants::Ignoring;
use json_ports::{Document, JsonFailure};

use crate::{Ignored, Line, Turn, Unrecognised};
use entry::{Closing, Entry};

impl Line {
    pub(crate) fn parsed(complete: &[u8]) -> Vec<Line> {
        Lines::from(LossyText::from(complete).as_ref())
            .into_iter()
            .filter(|line| line.chars().next().is_some())
            .filter_map(|line| Line::read(&line))
            .collect()
    }

    fn read(line: &str) -> Option<Line> {
        Entry::lexed(line)
            .ignoring(Ignored::Unparsed)
            .map_or_else(|| Some(Line::Unrecognised(Unrecognised::unreadable())), Option::<Line>::from)
    }
}

impl Entry {
    fn lexed(line: &str) -> Result<Entry, JsonFailure> {
        Document::read(line).and_then(|document| Entry::read(document.object()))
    }
}

impl From<&str> for Turn {
    fn from(tail: &str) -> Turn {
        let closing = Lines::from(tail)
            .into_iter()
            .rev()
            .map(|line| {
                Entry::lexed(&line).ignoring(Ignored::Unparsed).map_or(Closing::Leaves, |entry| entry.closing())
            })
            .find(|closing| *closing != Closing::Trails);
        match closing {
            Some(Closing::Ends) => Turn::Ended,
            Some(Closing::Trails | Closing::Leaves) | None => Turn::Open,
        }
    }
}
