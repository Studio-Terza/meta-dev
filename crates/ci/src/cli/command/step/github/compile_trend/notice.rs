mod growth;
mod range;

use std::cmp::Ordering;
use std::fmt;

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

    pub(super) fn new(
        today: &'builds Build,
        earlier: &'builds [Build],
        fortnight: &'builds Fortnight,
    ) -> Notice<'builds> {
        let range = earlier.iter().map(Build::total).map(Range::from).reduce(Range::spanning);
        Notice { today, earlier, fortnight, range }
    }

    pub(super) fn slower(&self) -> Option<&Notice<'builds>> {
        self.range.and_then(|range| range.below(self.today.total())).map(|_| self)
    }

    pub(super) fn reported<'notice>(&'notice self, merged: &'notice Text) -> Report<'notice> {
        Report { notice: self, merged }
    }

    fn grown(&self) -> Vec<Growth<'_>> {
        let mut grown: Vec<Growth<'_>> = self
            .today
            .crates()
            .iter()
            .filter_map(|(name, today)| Growth::against(name, *today, self.earlier).grew())
            .collect();
        grown.sort_by(|left, right| right.grown().partial_cmp(&left.grown()).unwrap_or(Ordering::Equal));
        grown.into_iter().take(Self::GROWN_LEN).collect()
    }

    fn compared(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "The cold build, clippy and test compile from scratch, took {}s in all today",
            self.today.total()
        )?;
        match self.range {
            Some(range) => write!(formatter, "; the {} runs of {} took {range}", self.earlier.len(), self.fortnight),
            None => write!(formatter, "; no run of {} to compare with", self.fortnight),
        }
    }
}

impl fmt::Display for Notice<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.compared(formatter)?;
        formatter.write_str(", so no trend issue.")
    }
}

impl fmt::Display for Report<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.notice.compared(formatter)?;
        writeln!(formatter, ": slower than every one of them.")?;
        writeln!(formatter)?;
        writeln!(formatter, "Crates that grew most, seconds today against their average over those runs:")?;
        self.notice.grown().iter().try_for_each(|growth| writeln!(formatter, "{growth}"))?;
        writeln!(formatter)?;
        writeln!(formatter, "Pull requests merged into main in {}, as context:", self.notice.fortnight)?;
        formatter.write_str(self.merged.as_ref())
    }
}
