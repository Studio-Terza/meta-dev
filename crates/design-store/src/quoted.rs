mod boundary;

use std::collections::BTreeSet;

use design_model::Segment;
use grammar::notes::{Date, Quote};
use port_core::Op;

#[derive(Clone, Debug, Op)]
pub struct QuotedSegments(BTreeSet<Quote>);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Quoted {
    segment: Segment,
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
}

impl AsRef<Segment> for Quoted {
    fn as_ref(&self) -> &Segment {
        &self.segment
    }
}
