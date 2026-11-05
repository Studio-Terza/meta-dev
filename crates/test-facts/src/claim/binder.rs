use std::collections::BTreeSet;
use std::fmt;

use newtype::{Arbitrary, Display, Newtype};

use super::ranges::Ranges;
use super::term::{Listing, Substituted, Substitution, Term};
use crate::boundary::Spelling;

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Cases(usize);

impl Cases {
    pub(crate) const DEFAULT: Cases = Cases::new(256);
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Draw {
    Every,
    Sampled(Cases),
    Written(Spelling),
}

impl Draw {
    fn every(&self) -> Option<&Draw> {
        match self {
            Draw::Every => Some(self),
            Draw::Sampled(_) | Draw::Written(_) => None,
        }
    }

    fn covers_draw(&self, inner: &Draw) -> Option<&Draw> {
        match self {
            Draw::Every => Some(self),
            Draw::Sampled(cases) => inner.sampled_within(*cases).map(|_| self),
            Draw::Written(spelling) => inner.written_as(spelling).map(|_| self),
        }
    }

    fn sampled_within(&self, cases: Cases) -> Option<&Draw> {
        match self {
            Draw::Sampled(drawn) => (*drawn <= cases).then_some(self),
            Draw::Every | Draw::Written(_) => None,
        }
    }

    fn written_as(&self, spelling: &Spelling) -> Option<&Spelling> {
        match self {
            Draw::Written(mine) => (mine == spelling).then_some(mine),
            Draw::Every | Draw::Sampled(_) => None,
        }
    }
}

impl fmt::Display for Draw {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Draw::Every => formatter.write_str("every value"),
            Draw::Sampled(cases) => write!(formatter, "{cases} sampled values"),
            Draw::Written(spelling) => write!(formatter, "values sampled as {spelling}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Listed(BTreeSet<Term>);

impl FromIterator<Term> for Listed {
    fn from_iter<Found: IntoIterator<Item = Term>>(found: Found) -> Listed {
        Listed(found.into_iter().collect())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Occupancy {
    Inhabited,
    Vacant,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Domain {
    Listed(Listed),
    Ranged(Ranges),
    Drawn(Term),
}

impl Domain {
    pub(crate) fn covers(&self, inner: &Domain) -> Option<&Domain> {
        match self {
            Domain::Listed(_) => None,
            Domain::Ranged(outer) => inner.ranged_within(outer).map(|_| self),
            Domain::Drawn(source) => inner.drawn_as(source).map(|_| self),
        }
    }

    pub(crate) fn ranged_within(&self, outer: &Ranges) -> Option<&Domain> {
        match self {
            Domain::Ranged(ranges) => outer.encloses(ranges).map(|_| self),
            Domain::Listed(_) | Domain::Drawn(_) => None,
        }
    }

    pub(crate) fn drawn_as(&self, source: &Term) -> Option<&Term> {
        match self {
            Domain::Drawn(mine) => (mine == source).then_some(mine),
            Domain::Listed(_) | Domain::Ranged(_) => None,
        }
    }

    pub(crate) fn holding(&self, value: &Term) -> Option<&Domain> {
        match self {
            Domain::Listed(values) => values.0.contains(value).then_some(self),
            Domain::Ranged(ranges) => value.number().and_then(|number| ranges.admits(number)).map(|_| self),
            Domain::Drawn(_) => None,
        }
    }

    pub(crate) fn occupancy(&self) -> Occupancy {
        match self {
            Domain::Listed(values) => values.0.first().map_or(Occupancy::Vacant, |_| Occupancy::Inhabited),
            Domain::Ranged(ranges) => ranges.occupancy(),
            Domain::Drawn(_) => Occupancy::Unknown,
        }
    }

    pub(crate) fn values(&self) -> Option<&BTreeSet<Term>> {
        match self {
            Domain::Listed(values) => Some(&values.0),
            Domain::Ranged(_) | Domain::Drawn(_) => None,
        }
    }

    pub(crate) fn ranges(&self) -> Option<&Ranges> {
        match self {
            Domain::Ranged(ranges) => Some(ranges),
            Domain::Listed(_) | Domain::Drawn(_) => None,
        }
    }
}

impl Substituted for Domain {
    fn substituted(&self, values: &Substitution) -> Domain {
        match self {
            Domain::Drawn(source) => Domain::Drawn(source.substituted(values)),
            Domain::Listed(_) | Domain::Ranged(_) => self.clone(),
        }
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Domain::Listed(values) => write!(formatter, "[{}]", values.0.iter().collect::<Listing<'_>>()),
            Domain::Ranged(ranges) => write!(formatter, "{ranges}"),
            Domain::Drawn(source) => write!(formatter, "{source}"),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Binder {
    domain: Domain,
    draw: Draw,
}

impl Binder {
    pub(crate) fn new(domain: Domain, draw: Draw) -> Binder {
        Binder { domain, draw }
    }

    pub(crate) fn every(&self) -> Option<&Binder> {
        self.draw.every().map(|_| self)
    }

    pub(crate) fn domain(&self) -> &Domain {
        &self.domain
    }

    pub(crate) fn restricted(&self, ranges: Ranges) -> Binder {
        Binder { domain: Domain::Ranged(ranges), draw: self.draw.clone() }
    }

    pub(crate) fn admits(&self, value: &Term) -> Option<&Binder> {
        self.every().and_then(|binder| binder.domain.holding(value)).map(|_| self)
    }

    pub(crate) fn encloses(&self, inner: &Binder) -> Option<&Binder> {
        self.draw.covers_draw(&inner.draw).and_then(|_| self.domain.covers(&inner.domain)).map(|_| self)
    }
}

impl Substituted for Binder {
    fn substituted(&self, values: &Substitution) -> Binder {
        Binder { domain: self.domain.substituted(values), draw: self.draw.clone() }
    }
}

impl fmt::Display for Binder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} of {}", self.draw, self.domain)
    }
}
