mod growth;
mod range;

use std::cmp::Ordering;
use std::fmt;

use algebra::Semigroup;

use super::Pace;
use super::build::Build;
use super::fortnight::Fortnight;
use crate::cli::ports::Text;
use growth::Growth;
use range::Range;

#[derive(Debug)]
pub struct Notice<'builds> {
    today: &'builds Build,
    earlier: &'builds [Build],
    fortnight: &'builds Fortnight,
    range: Option<Range>,
}

#[derive(Debug)]
pub struct Report<'notice> {
    notice: &'notice Notice<'notice>,
    merged: &'notice Text,
}

impl<'builds> Notice<'builds> {
    const GROWN_LEN: usize = 5;
    const TOOK: &'static str = "The cold build, clippy and test compile from scratch, took ";

    pub(super) fn new(
        today: &'builds Build,
        earlier: &'builds [Build],
        fortnight: &'builds Fortnight,
    ) -> Notice<'builds> {
        let range = earlier.iter().map(Build::total).map(Range::from).reduce(Range::combine);
        Notice { today, earlier, fortnight, range }
    }

    pub(super) fn pace(&self) -> Pace {
        self.range.map_or(Pace::Steady, |range| range.pace(self.today.total()))
    }

    pub(super) fn reported<'notice>(&'notice self, merged: &'notice Text) -> Report<'notice> {
        Report { notice: self, merged }
    }

    fn grown(&self) -> Vec<Growth<'_>> {
        let mut grown: Vec<Growth<'_>> = self
            .today
            .crates()
            .iter()
            .filter_map(|(name, today)| Growth::against(name, *today, self.earlier))
            .collect();
        grown.sort_by(|left, right| right.grown().partial_cmp(&left.grown()).unwrap_or(Ordering::Equal));
        grown.into_iter().take(Self::GROWN_LEN).collect()
    }
}

#[derive(Debug)]
struct Compared<'notice, 'builds>(&'notice Notice<'builds>);

impl fmt::Display for Compared<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0.range {
            Some(range) => write!(
                formatter,
                "{}{}s in all today; the {} runs of {} took {range}",
                Notice::TOOK,
                self.0.today.total(),
                self.0.earlier.len(),
                self.0.fortnight
            ),
            None => write!(
                formatter,
                "{}{}s in all today; no run of {} to compare with",
                Notice::TOOK,
                self.0.today.total(),
                self.0.fortnight
            ),
        }
    }
}

impl fmt::Display for Notice<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}, so no trend issue.", Compared(self))
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}: slower than every one of them.", Compared(self.notice))?;
        writeln!(formatter)?;
        writeln!(formatter, "Crates that grew most, seconds today against their average over those runs:")?;
        self.notice.grown().iter().try_for_each(|growth| writeln!(formatter, "{growth}"))?;
        writeln!(formatter)?;
        writeln!(formatter, "Pull requests merged into main in {}, as context:", self.notice.fortnight)?;
        formatter.write_str(self.merged.as_ref())
    }
}
