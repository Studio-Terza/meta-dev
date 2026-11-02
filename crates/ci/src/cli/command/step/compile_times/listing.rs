mod entry;
mod rows;

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fmt;

use super::unit::Unit;
use crate::cli::command::step::seconds::Seconds;
use entry::Entry;
use rows::Rows;
use text_lines::OneLine;

#[derive(Debug)]
pub struct Listing(Vec<Entry>);

impl Listing {
    pub(super) fn totalled(units: &[Unit]) -> Listing {
        let totals = units.iter().fold(BTreeMap::<OneLine, Seconds>::new(), |mut totals, unit| {
            let total = totals.entry(unit.label().clone()).or_default();
            *total = [*total, unit.took()].into_iter().sum();
            totals
        });
        let mut longest: Vec<Entry> = totals.into_iter().map(|(label, seconds)| Entry::new(label, seconds)).collect();
        longest.sort_by(|left, right| right.partial_cmp(left).unwrap_or(Ordering::Equal));
        Listing(longest)
    }
}

impl fmt::Display for Listing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total: Seconds = self.0.iter().map(Entry::took).sum();
        writeln!(formatter, "compile time per crate, seconds, longest first, {total} in all:")?;
        Rows::LISTED.kept(self.0.iter()).try_for_each(|entry| writeln!(formatter, "{entry}"))
    }
}
