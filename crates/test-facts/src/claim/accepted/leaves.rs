use std::collections::BTreeSet;
use std::fmt;

use super::Accepted;
use crate::boundary::Quantity;
use crate::claim::ranges::Ranges;
use crate::claim::term::{Depths, Listing, Operator, Substituted, Substitution, Term};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Inside;

pub(crate) trait Admitting {
    fn admits(&self, value: &Term) -> Option<Inside>;

    fn admits_each<'value>(&self, values: impl IntoIterator<Item = &'value Term>) -> Option<Inside> {
        values.into_iter().try_for_each(|value| self.admits(value).map(|_| ())).map(|()| Inside)
    }
}

#[derive(Debug, Default)]
pub(crate) struct Mentions<'term>(Vec<&'term Term>);

impl<'term> From<&Mentions<'term>> for Depths {
    fn from(mentions: &Mentions<'term>) -> Depths {
        mentions.0.iter().map(|term| term.depths()).collect()
    }
}

impl<'term> Mentions<'term> {
    pub(crate) fn listed(&self) -> &[&'term Term] {
        &self.0
    }
}

impl<'term> FromIterator<Mentions<'term>> for Mentions<'term> {
    fn from_iter<Found: IntoIterator<Item = Mentions<'term>>>(found: Found) -> Mentions<'term> {
        Mentions(found.into_iter().flat_map(|mentions| mentions.0).collect())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Values(BTreeSet<Term>);

impl From<Term> for Values {
    fn from(value: Term) -> Values {
        Values(BTreeSet::from([value]))
    }
}

impl<'term> From<&'term Values> for Mentions<'term> {
    fn from(values: &'term Values) -> Mentions<'term> {
        Mentions(values.0.iter().collect())
    }
}

impl Values {
    pub(crate) fn sole(&self) -> Option<&Term> {
        self.0.first().filter(|_| self.0.len() == 1)
    }
}

impl<'values> IntoIterator for &'values Values {
    type Item = &'values Term;
    type IntoIter = std::collections::btree_set::Iter<'values, Term>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl Admitting for Values {
    fn admits(&self, value: &Term) -> Option<Inside> {
        self.0.contains(value).then_some(Inside)
    }
}

impl Substituted for Values {
    fn substituted(&self, values: &Substitution) -> Values {
        Values(self.0.substituted(values))
    }
}

impl fmt::Display for Values {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.iter().collect::<Listing<'_>>())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Exclusion(BTreeSet<Term>);

impl From<Term> for Exclusion {
    fn from(value: Term) -> Exclusion {
        Exclusion(BTreeSet::from([value]))
    }
}

impl<'term> From<&'term Exclusion> for Mentions<'term> {
    fn from(exclusion: &'term Exclusion) -> Mentions<'term> {
        Mentions(exclusion.0.iter().collect())
    }
}

impl Exclusion {
    pub(crate) fn missing_from(&self, inner: &Ranges) -> Option<Inside> {
        self.0
            .iter()
            .try_for_each(|value| value.number().and_then(|number| Exclusion::outside(inner, number)))
            .map(|()| Inside)
    }

    pub(crate) fn excluded_by(&self, inner: &Exclusion) -> Option<Inside> {
        self.0.is_subset(&inner.0).then_some(Inside)
    }

    fn settled(&self) -> Option<&Exclusion> {
        self.0.iter().try_for_each(|value| value.closed().map(|_| ())).map(|()| self)
    }

    fn absent(found: Option<&Term>) -> Option<Inside> {
        match found {
            Some(_) => None,
            None => Some(Inside),
        }
    }

    fn outside(ranges: &Ranges, number: Quantity) -> Option<()> {
        match ranges.admits(number) {
            Some(_) => None,
            None => Some(()),
        }
    }
}

impl Admitting for Exclusion {
    fn admits(&self, value: &Term) -> Option<Inside> {
        value.closed().zip(self.settled()).and_then(|(closed, _)| Exclusion::absent(self.0.get(closed)))
    }
}

impl Substituted for Exclusion {
    fn substituted(&self, values: &Substitution) -> Exclusion {
        Exclusion(self.0.substituted(values))
    }
}

impl fmt::Display for Exclusion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.iter().collect::<Listing<'_>>())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Relation {
    order: Order,
    bound: Term,
}

impl<'term> From<&'term Relation> for Mentions<'term> {
    fn from(relation: &'term Relation) -> Mentions<'term> {
        Mentions(vec![&relation.bound])
    }
}

impl Relation {
    pub(crate) fn new(order: Order, bound: Term) -> Relation {
        Relation { order, bound }
    }

    pub(crate) fn resolved(&self, values: &Substitution) -> Accepted {
        Accepted::ordered(self.order, self.bound.substituted(values))
    }

    pub(crate) fn implied_by(&self, inner: &Relation) -> Option<Inside> {
        (inner.bound == self.bound).then_some(inner.order).and_then(|order| order.implies(self.order)).map(|_| Inside)
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{:?} {}", self.order, self.bound)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Order {
    Below,
    AtMost,
    Above,
    AtLeast,
}

impl Order {
    const IMPLIED: [(Order, Order); 6] = [
        (Order::Below, Order::Below),
        (Order::AtMost, Order::AtMost),
        (Order::Above, Order::Above),
        (Order::AtLeast, Order::AtLeast),
        (Order::Below, Order::AtMost),
        (Order::Above, Order::AtLeast),
    ];

    pub(crate) fn reversed(self) -> Order {
        match self {
            Order::Below => Order::Above,
            Order::AtMost => Order::AtLeast,
            Order::Above => Order::Below,
            Order::AtLeast => Order::AtMost,
        }
    }

    pub(crate) fn implies(self, weaker: Order) -> Option<Order> {
        Self::IMPLIED.contains(&(self, weaker)).then_some(weaker)
    }

    pub(crate) fn ranges(self, bound: Quantity) -> Ranges {
        match self {
            Order::Below => {
                Ranges::between(Quantity::LEAST, bound.combined(Operator::Subtract, Quantity::UNIT).unwrap_or(bound))
            }
            Order::AtMost => Ranges::between(Quantity::LEAST, bound),
            Order::Above => {
                Ranges::between(bound.combined(Operator::Add, Quantity::UNIT).unwrap_or(bound), Quantity::GREATEST)
            }
            Order::AtLeast => Ranges::between(bound, Quantity::GREATEST),
        }
    }
}
