use std::collections::BTreeSet;
use std::fmt;
use std::ops::{BitAnd, BitOr};

use super::binder::Occupancy;
use super::term::Operator;
use crate::boundary::Quantity;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Span {
    low: Quantity,
    high: Quantity,
}

impl Span {
    pub(crate) fn between(low: Quantity, high: Quantity) -> Option<Span> {
        Some(Span { low, high }).filter(|span| span.low <= span.high)
    }

    fn overlap(self, other: Span) -> Option<Span> {
        Span::between(self.low.max(other.low), self.high.min(other.high))
    }
}

impl fmt::Display for Span {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..={}", self.low, self.high)
    }
}

#[derive(Debug, Default)]
struct Merged(Vec<Span>);

impl Merged {
    fn appended(mut self, next: Span) -> Merged {
        let last = self.0.pop();
        let joined = last.and_then(|held| Merged::joined(held, next));
        self.0.extend(joined.map_or_else(|| last.into_iter().chain([next]).collect(), |span| vec![span]));
        self
    }

    fn joined(last: Span, next: Span) -> Option<Span> {
        let reach = last.high.combined(Operator::Add, Quantity::UNIT).unwrap_or(last.high);
        (next.low <= reach).then(|| Span { low: last.low, high: last.high.max(next.high) })
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Ranges(BTreeSet<Span>);

impl Ranges {
    pub(crate) fn spanning(spans: impl IntoIterator<Item = Span>) -> Ranges {
        let sorted: BTreeSet<Span> = spans.into_iter().collect();
        Ranges(sorted.into_iter().fold(Merged::default(), Merged::appended).0.into_iter().collect())
    }

    pub(crate) fn between(low: Quantity, high: Quantity) -> Ranges {
        Ranges::spanning(Span::between(low, high))
    }

    pub(crate) fn encloses(&self, inner: &Ranges) -> Option<&Ranges> {
        inner.0.iter().all(|span| self.0.iter().any(|outer| outer.overlap(*span) == Some(*span))).then_some(self)
    }

    pub(crate) fn occupancy(&self) -> Occupancy {
        self.0.first().map_or(Occupancy::Vacant, |_| Occupancy::Inhabited)
    }

    pub(crate) fn admits(&self, value: Quantity) -> Option<Quantity> {
        self.encloses(&Ranges::between(value, value)).map(|_| value)
    }
}

impl BitAnd for &Ranges {
    type Output = Ranges;

    fn bitand(self, other: &Ranges) -> Ranges {
        Ranges::spanning(self.0.iter().flat_map(|mine| other.0.iter().filter_map(|theirs| mine.overlap(*theirs))))
    }
}

impl BitOr for &Ranges {
    type Output = Ranges;

    fn bitor(self, other: &Ranges) -> Ranges {
        Ranges::spanning(self.0.iter().chain(&other.0).copied())
    }
}

impl fmt::Display for Ranges {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(Span::to_string).collect();
        formatter.write_str(&spelled.join(" or "))
    }
}
