use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use newtype::Newtype;

use crate::boundary::Quantity;

use super::Cases;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Span {
    low: Quantity,
    high: Quantity,
}

impl Span {
    pub(crate) fn between(low: Quantity, high: Quantity) -> Option<Span> {
        Some(Span { low, high }).filter(|span| span.low <= span.high)
    }

    pub(crate) fn width(self) -> Cases {
        Cases::new(
            usize::try_from(
                self.high
                    .into_inner()
                    .saturating_sub(self.low.into_inner())
                    .saturating_add(Quantity::UNIT.into_inner()),
            )
            .unwrap_or(usize::MAX),
        )
    }

    fn enclosing(self, other: Span) -> Option<Span> {
        Some(self).filter(|span| span.low <= other.low).filter(|span| other.high <= span.high)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}..={}", self.low, self.high)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Spans(BTreeSet<Span>);

impl Spans {
    pub(crate) const NONE: Spans = Spans(BTreeSet::new());

    pub(crate) fn single(span: Span) -> Spans {
        Spans(BTreeSet::from([span]))
    }

    pub(crate) fn joined(self, other: Spans) -> Spans {
        Spans(self.0.into_iter().chain(other.0).collect())
    }

    fn enclosing(&self, other: &Spans) -> Option<&Spans> {
        other.0.iter().all(|inner| self.0.iter().any(|outer| outer.enclosing(*inner).is_some())).then_some(self)
    }
}

impl PartialOrd for Spans {
    fn partial_cmp(&self, other: &Spans) -> Option<Ordering> {
        let above = self.enclosing(other).map(|_| Ordering::Greater);
        let below = other.enclosing(self).map(|_| Ordering::Less);
        above.zip(below).map(|_| Ordering::Equal).or(above).or(below)
    }
}

impl fmt::Display for Spans {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.iter().try_for_each(|span| write!(formatter, " over {span}"))
    }
}
