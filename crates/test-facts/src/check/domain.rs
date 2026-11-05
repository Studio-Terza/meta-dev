use std::cmp::Ordering;
use std::fmt;

use newtype::{Arbitrary, Counter, Display, Newtype};

use super::product::Product;
use super::values::Values;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cases(usize);

impl Cases {
    const ONE: Cases = Cases::new(1);

    fn times(self, other: Cases) -> Cases {
        Cases::new(self.count().saturating_mul(other.count()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rows {
    cases: Cases,
    leaves: Values,
}

impl Rows {
    const SINGLE: Rows = Rows { cases: Cases::ONE, leaves: Values::new(0) };

    pub(crate) fn new(cases: Cases, leaves: Values) -> Rows {
        Rows { cases, leaves }
    }

    fn crossed(self, other: Rows) -> Rows {
        Rows { cases: self.cases.times(other.cases), leaves: self.leaves.plus(other.leaves) }
    }
}

impl PartialOrd for Rows {
    fn partial_cmp(&self, other: &Rows) -> Option<Ordering> {
        Product::combined([self.cases.partial_cmp(&other.cases), self.leaves.partial_cmp(&other.leaves)])
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Domain {
    breadth: Breadth,
    rows: Rows,
}

impl Domain {
    pub(crate) const SINGLE: Domain = Domain { breadth: Breadth::Listed, rows: Rows::SINGLE };
    pub(crate) const ENUMERATED: Domain = Domain { breadth: Breadth::Enumerated, rows: Rows::SINGLE };
    pub(crate) const GENERATED: Domain = Domain { breadth: Breadth::Generated, rows: Rows::SINGLE };

    pub(crate) fn listed(rows: Rows) -> Domain {
        Domain { breadth: Breadth::Listed, rows }
    }

    pub(crate) fn max_with(self, other: Domain) -> Domain {
        Domain { breadth: self.breadth.max(other.breadth), rows: self.rows.crossed(other.rows) }
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
        write!(formatter, "{} cases, {} listed holding {} values", self.breadth, self.rows.cases, self.rows.leaves)
    }
}
