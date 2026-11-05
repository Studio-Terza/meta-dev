use std::collections::BTreeSet;
use std::fmt;

use super::accepted::Order;
use super::ranges::Ranges;
use super::term::{Callee, Depth, Depths, Operator, Substituted, Substitution, Term, Turn};
use crate::boundary::{Quantity, Truth};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Implied;

#[derive(Debug)]
pub(crate) struct Bounding {
    depth: Depth,
    ranges: Ranges,
}

impl Bounding {
    pub(crate) fn depth(&self) -> Depth {
        self.depth
    }

    pub(crate) fn ranges(&self) -> &Ranges {
        &self.ranges
    }
}

#[derive(Debug)]
struct Comparands<'term> {
    operator: Operator,
    left: &'term Term,
    right: &'term Term,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Premise {
    term: Term,
    holds: Truth,
}

impl Premise {
    const COMPARING: [Operator; 6] =
        [Operator::Equal, Operator::Unequal, Operator::Below, Operator::AtMost, Operator::Above, Operator::AtLeast];

    pub(crate) fn new(term: Term, holds: Truth) -> Premise {
        Premise { term, holds }
    }

    pub(crate) fn never() -> Premise {
        Premise { term: Term::Truth(Truth::Fails), holds: Truth::Holds }
    }

    pub(crate) fn conjuncts(self) -> Vec<Premise> {
        let negated = Callee::Operator(Operator::Not)
            .arguments_of(&self.term)
            .and_then(|arguments| arguments.listed().first())
            .map(|inner| Premise::new(inner.clone(), self.holds.negated()).conjuncts());
        let joined = Callee::Operator(Operator::joining(self.holds)).arguments_of(&self.term).map(|arguments| {
            arguments.listed().iter().flat_map(|part| Premise::new(part.clone(), self.holds).conjuncts()).collect()
        });
        negated.or(joined).unwrap_or_else(|| vec![self])
    }

    pub(crate) fn bounding(&self) -> Option<Bounding> {
        let comparands = self.comparison()?;
        let held = self.under(comparands.operator);
        let direct =
            comparands.left.bound().zip(comparands.right.number()).map(|(depth, number)| (depth, number, held));
        let reversed = comparands
            .right
            .bound()
            .zip(comparands.left.number())
            .map(|(depth, number)| (depth, number, held.turned(Turn::Reversal)));
        direct.or(reversed).and_then(|(depth, number, operator)| {
            Self::spanned(operator, number).map(|ranges| Bounding { depth, ranges })
        })
    }

    pub(crate) fn verdict(&self) -> Option<Truth> {
        self.term.truth().map(|actual| Truth::judged(actual == self.holds))
    }

    fn matched(&self, target: &Premise, found: &mut Substitution) -> Option<()> {
        let mut attempt = found.clone();
        (self.holds == target.holds).then_some(()).and_then(|()| self.term.matched(&target.term, &mut attempt)).map(
            |()| {
                *found = attempt;
            },
        )
    }

    fn comparison(&self) -> Option<Comparands<'_>> {
        Self::COMPARING.iter().find_map(|operator| {
            Callee::Operator(*operator)
                .arguments_of(&self.term)
                .and_then(|arguments| arguments.listed().first().zip(arguments.listed().get(1)))
                .map(|(left, right)| Comparands { operator: *operator, left, right })
        })
    }

    fn under(&self, operator: Operator) -> Operator {
        match self.holds {
            Truth::Holds => operator,
            Truth::Fails => operator.turned(Turn::Complement),
        }
    }

    fn spanned(operator: Operator, number: Quantity) -> Option<Ranges> {
        match operator {
            Operator::Below => Some(Order::Below.ranges(number)),
            Operator::AtMost => Some(Order::AtMost.ranges(number)),
            Operator::Above => Some(Order::Above.ranges(number)),
            Operator::AtLeast => Some(Order::AtLeast.ranges(number)),
            Operator::Equal => Some(Ranges::between(number, number)),
            Operator::Unequal => Some(Ranges::excluding(number)),
            _ => None,
        }
    }
}

impl Substituted for Premise {
    fn substituted(&self, values: &Substitution) -> Premise {
        Premise { term: self.term.substituted(values), holds: self.holds }
    }
}

impl fmt::Display for Premise {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.holds {
            Truth::Holds => write!(formatter, "{}", self.term),
            Truth::Fails => write!(formatter, "not {}", self.term),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Guard(BTreeSet<Premise>);

impl Guard {
    pub(crate) fn falsified() -> Guard {
        Guard(BTreeSet::from([Premise::never()]))
    }

    pub(crate) fn joined(self, other: Guard) -> Guard {
        Guard(self.0.into_iter().chain(other.0).collect())
    }

    pub(crate) fn conjoined(&self, premise: Premise) -> Guard {
        Guard(self.0.iter().cloned().chain([premise]).collect())
    }

    pub(crate) fn pruned(&self) -> Option<Guard> {
        self.0
            .iter()
            .try_for_each(|premise| {
                premise.verdict().filter(|verdict| *verdict == Truth::Fails).map_or(Some(()), |_| None)
            })
            .map(|()| Guard(self.0.iter().filter(|premise| premise.verdict().is_none()).cloned().collect()))
    }

    pub(crate) fn implied_by(&self, other: &Guard) -> Option<Implied> {
        self.0.is_subset(&other.0).then_some(Implied)
    }

    pub(crate) fn matched(&self, target: &Guard, found: &mut Substitution) -> Option<()> {
        self.0.iter().try_for_each(|premise| target.0.iter().find_map(|inner| premise.matched(inner, found)))
    }

    pub(crate) fn depths(&self) -> Depths {
        self.0.iter().map(|premise| premise.term.depths()).collect()
    }
}

impl Substituted for Guard {
    fn substituted(&self, values: &Substitution) -> Guard {
        Guard(self.0.substituted(values))
    }
}

impl fmt::Display for Guard {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        Some(spelled)
            .filter(|spelled| spelled.iter().next().is_some())
            .map_or(Ok(()), |spelled| write!(formatter, ", when {}", spelled.join(" and ")))
    }
}
