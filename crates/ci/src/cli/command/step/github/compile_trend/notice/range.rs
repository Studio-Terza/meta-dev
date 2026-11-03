use std::fmt;

use crate::cli::command::step::seconds::Seconds;

#[derive(Clone, Copy, Debug)]
pub struct Range {
    least: Seconds,
    most: Seconds,
}

impl Range {
    pub(super) fn around(range: Option<Range>, total: Seconds) -> Range {
        range.map_or(Range { least: total, most: total }, |range| range.widened(total))
    }

    fn widened(self, total: Seconds) -> Range {
        Range {
            least: if total < self.least { total } else { self.least },
            most: if total > self.most { total } else { self.most },
        }
    }

    pub(super) fn below(self, total: Seconds) -> Option<Range> {
        (total > self.most).then_some(self)
    }
}

impl fmt::Display for Range {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}s to {}s", self.least, self.most)
    }
}
