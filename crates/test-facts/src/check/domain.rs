mod span;

use std::cmp::Ordering;
use std::fmt;

use newtype::{Arbitrary, Counter, Display, Newtype};

use super::product::Product;
use super::values::Values;
use crate::boundary::Quantity;
use span::{Span, Spans};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cases(usize);

impl Cases {
    const ONE: Cases = Cases::new(1);

    fn times(self, other: Cases) -> Cases {
        Cases::new(self.count().saturating_mul(other.count()))
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rows {
    cases: Cases,
    distinct: Cases,
    leaves: Values,
    spans: Spans,
}

impl Rows {
    const SINGLE: Rows = Rows { cases: Cases::ONE, distinct: Cases::ONE, leaves: Values::new(0), spans: Spans::NONE };

    const EMPTY: Rows =
        Rows { cases: Cases::new(0), distinct: Cases::new(0), leaves: Values::new(0), spans: Spans::NONE };

    pub(crate) fn listed(cases: Cases, distinct: Cases, leaves: Values) -> Rows {
        Rows { cases, distinct, leaves, spans: Spans::NONE }
    }

    pub(crate) fn over(low: Quantity, high: Quantity) -> Rows {
        Span::between(low, high).map_or(Rows::EMPTY, |span| Rows {
            cases: span.width(),
            distinct: span.width(),
            leaves: Values::zero(),
            spans: Spans::single(span),
        })
    }

    fn crossed(self, other: Rows) -> Rows {
        Rows {
            cases: self.cases.times(other.cases),
            distinct: self.distinct.times(other.distinct),
            leaves: self.leaves.plus(other.leaves),
            spans: self.spans.joined(other.spans),
        }
    }
}

impl PartialOrd for Rows {
    fn partial_cmp(&self, other: &Rows) -> Option<Ordering> {
        Product::combined([
            self.cases.partial_cmp(&other.cases),
            self.distinct.partial_cmp(&other.distinct),
            self.leaves.partial_cmp(&other.leaves),
            self.spans.partial_cmp(&other.spans),
        ])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Breadth {
    Listed,
    Enumerated,
    Generated,
}

impl fmt::Display for Breadth {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Breadth::Listed => formatter.write_str("listed"),
            Breadth::Enumerated => formatter.write_str("enumerated"),
            Breadth::Generated => formatter.write_str("generated"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Domain {
    breadth: Breadth,
    rows: Rows,
}

impl Domain {
    pub(crate) const SINGLE: Domain = Domain { breadth: Breadth::Listed, rows: Rows::SINGLE };
    pub(crate) const ENUMERATED: Domain = Domain { breadth: Breadth::Enumerated, rows: Rows::SINGLE };
    pub(crate) const GENERATED: Domain = Domain { breadth: Breadth::Generated, rows: Rows::SINGLE };

    pub(crate) fn new(breadth: Breadth, rows: Rows) -> Domain {
        Domain { breadth, rows }
    }

    pub(crate) fn max_with(self, other: Domain) -> Domain {
        Domain { breadth: self.breadth.max(other.breadth), rows: self.rows.crossed(other.rows) }
    }

    pub(crate) fn running(self) -> Option<Domain> {
        Some(self).filter(|domain| domain.rows.cases.count() > 0)
    }
}

impl PartialOrd for Domain {
    fn partial_cmp(&self, other: &Domain) -> Option<Ordering> {
        Some(self.breadth.cmp(&other.breadth))
            .filter(|order| order.is_ne())
            .or_else(|| self.rows.partial_cmp(&other.rows))
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} domain of {} cases ({} distinct, {} values){}",
            self.breadth, self.rows.cases, self.rows.distinct, self.rows.leaves, self.rows.spans
        )
    }
}
