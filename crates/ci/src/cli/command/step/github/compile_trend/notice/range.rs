use std::fmt;

use newtype::Newtype;

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
            least: Seconds::new(self.least.into_inner().min(total.into_inner())),
            most: Seconds::new(self.most.into_inner().max(total.into_inner())),
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
