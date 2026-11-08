mod boundary;

use std::collections::BTreeSet;

use design_model::{Ordinal, Segment};
use grammar::notes::{Date, Quote};
use port_core::Op;

#[derive(Clone, Debug, Op)]
pub struct QuotedSegments(BTreeSet<Quote>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quoted {
    segment: Segment,
    occurrence: Ordinal,
    day: Date,
}

impl FromIterator<Quote> for QuotedSegments {
    fn from_iter<Quotes: IntoIterator<Item = Quote>>(quotes: Quotes) -> QuotedSegments {
        QuotedSegments(quotes.into_iter().collect())
    }
}

impl Quoted {
    #[must_use]
    pub fn day(&self) -> &Date {
        &self.day
    }

    #[must_use]
    pub fn occurrence(&self) -> Ordinal {
        self.occurrence
    }
}

impl AsRef<Segment> for Quoted {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}
