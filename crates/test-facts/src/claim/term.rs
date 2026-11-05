use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use law_facts::Kind;
use lint_layout::Name;
use newtype::{Arbitrary, Display, Newtype};

use crate::boundary::{Position, Quantity, Spelling, Truth};

#[derive(Newtype, Arbitrary, Display, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Depth(usize);

pub(crate) trait Substituted {
    fn substituted(&self, values: &Substitution) -> Self;
}

impl<Part: Substituted + Ord> Substituted for BTreeSet<Part> {
    fn substituted(&self, values: &Substitution) -> BTreeSet<Part> {
        self.iter().map(|part| part.substituted(values)).collect()
    }
}

impl<Part: Substituted> Substituted for Vec<Part> {
    fn substituted(&self, values: &Substitution) -> Vec<Part> {
        self.iter().map(|part| part.substituted(values)).collect()
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Substitution(BTreeMap<Depth, Term>);

impl Substitution {
    pub(crate) fn single(depth: Depth, value: Term) -> Substitution {
        Substitution(BTreeMap::from([(depth, value)]))
    }

    pub(crate) fn renumbered<'depth>(depths: impl Iterator<Item = &'depth Depth>) -> Substitution {
        Substitution(depths.enumerate().map(|(index, depth)| (*depth, Term::Bound(Depth::new(index)))).collect())
    }

    pub(crate) fn bound_value(&self, depth: Depth) -> Option<&Term> {
        self.0.get(&depth)
    }

    pub(crate) fn bind(&mut self, depth: Depth, value: &Term) -> Option<()> {
        (self.0.entry(depth).or_insert_with(|| value.clone()) == value).then_some(())
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct Depths(BTreeSet<Depth>);

impl Depths {
    pub(crate) fn iter(&self) -> impl Iterator<Item = &Depth> {
        self.0.iter()
    }
}

impl From<Depth> for Depths {
    fn from(depth: Depth) -> Depths {
        Depths(BTreeSet::from([depth]))
    }
}

impl FromIterator<Depths> for Depths {
    fn from_iter<Found: IntoIterator<Item = Depths>>(found: Found) -> Depths {
        Depths(found.into_iter().flat_map(|depths| depths.0).collect())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Not,
    Both,
    Either,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Arguments<'term>(&'term [Term]);

impl<'term> Arguments<'term> {
    pub(crate) fn listed(&self) -> &'term [Term] {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Callee {
    Function(Name),
    Method(Name),
    Operator(Operator),
    Constructor(Name),
    Field(Name),
    Index,
    Sequence,
    Tuple,
    Law(Kind),
}

impl Callee {
    pub(crate) fn folded(self, arguments: Vec<Term>) -> Term {
        match self {
            Callee::Operator(operator) => Folding::computed(operator, arguments),
            Callee::Field(name) => Folding::projected(name, arguments),
            Callee::Index => Folding::indexed(arguments),
            Callee::Function(_)
            | Callee::Method(_)
            | Callee::Constructor(_)
            | Callee::Sequence
            | Callee::Tuple
            | Callee::Law(_) => Term::Applied(self, arguments),
        }
    }

    pub(crate) fn arguments_of<'term>(&self, term: &'term Term) -> Option<Arguments<'term>> {
        match term {
            Term::Applied(held, arguments) => (held == self).then_some(Arguments(arguments.as_slice())),
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) | Term::Bound(_) | Term::Opaque(_) => None,
        }
    }

    fn closed_over(&self, arguments: &[Term]) -> Option<&Callee> {
        match self {
            Callee::Tuple | Callee::Sequence | Callee::Constructor(_) | Callee::Operator(_) => {
                Callee::all_closed(arguments).map(|()| self)
            }
            Callee::Function(_) | Callee::Method(_) | Callee::Field(_) | Callee::Index | Callee::Law(_) => None,
        }
    }

    fn all_closed(arguments: &[Term]) -> Option<()> {
        arguments.iter().try_for_each(|argument| argument.closed().map(|_| ()))
    }

    fn deferred(&self) -> Option<()> {
        match self {
            Callee::Operator(_) => Some(()),
            Callee::Function(_)
            | Callee::Method(_)
            | Callee::Constructor(_)
            | Callee::Field(_)
            | Callee::Index
            | Callee::Sequence
            | Callee::Tuple
            | Callee::Law(_) => None,
        }
    }

    fn matched_against(&self, arguments: &[Term], target: &Term, found: &mut Substitution) -> Option<()> {
        self.arguments_of(target).filter(|targets| targets.listed().len() == arguments.len()).map_or_else(
            || self.deferred(),
            |targets| {
                arguments.iter().zip(targets.listed()).try_for_each(|(argument, inner)| argument.matched(inner, found))
            },
        )
    }
}

struct Folding;

impl Folding {
    fn computed(operator: Operator, arguments: Vec<Term>) -> Term {
        let numbers: Option<Vec<Quantity>> = arguments.iter().map(Term::number).collect();
        numbers
            .and_then(|values| {
                values.first().zip(values.get(1)).and_then(|(left, right)| left.combined(operator, *right))
            })
            .map_or_else(|| Term::Applied(Callee::Operator(operator), arguments), Term::Number)
    }

    fn projected(name: Name, arguments: Vec<Term>) -> Term {
        let picked = arguments
            .first()
            .and_then(|base| Callee::Tuple.arguments_of(base))
            .zip(Position::named(&name))
            .and_then(|(elements, position)| position.picked(elements.listed()))
            .cloned();
        picked.unwrap_or(Term::Applied(Callee::Field(name), arguments))
    }

    fn indexed(arguments: Vec<Term>) -> Term {
        let position = arguments.get(1).and_then(Term::number).and_then(Position::numbered);
        let picked = arguments
            .first()
            .and_then(|base| Callee::Sequence.arguments_of(base))
            .zip(position)
            .and_then(|(elements, position)| position.picked(elements.listed()))
            .cloned();
        picked.unwrap_or(Term::Applied(Callee::Index, arguments))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Term {
    Number(Quantity),
    Truth(Truth),
    Spelled(Spelling),
    Bound(Depth),
    Applied(Callee, Vec<Term>),
    Opaque(Spelling),
}

impl Term {
    pub(crate) fn invoked(&self, arguments: Vec<Term>) -> Term {
        let named = self.function_name();
        let callee = named.clone().map_or_else(|| Callee::Method(Name::from("call")), Callee::Function);
        let receiver = named.map_or_else(|| Some(self.clone()), |_| None);
        callee.folded(receiver.into_iter().chain(arguments).collect())
    }

    fn function_name(&self) -> Option<Name> {
        match self {
            Term::Applied(Callee::Function(name), held) => held.is_empty().then(|| name.clone()),
            Term::Number(_)
            | Term::Truth(_)
            | Term::Spelled(_)
            | Term::Bound(_)
            | Term::Applied(..)
            | Term::Opaque(_) => None,
        }
    }

    pub(crate) fn closed(&self) -> Option<&Term> {
        match self {
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) => Some(self),
            Term::Applied(callee, arguments) => callee.closed_over(arguments).map(|_| self),
            Term::Bound(_) | Term::Opaque(_) => None,
        }
    }

    pub(crate) fn bound(&self) -> Option<Depth> {
        match self {
            Term::Bound(depth) => Some(*depth),
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) | Term::Applied(..) | Term::Opaque(_) => None,
        }
    }

    pub(crate) fn truth(&self) -> Option<Truth> {
        match self {
            Term::Truth(truth) => Some(*truth),
            Term::Number(_) | Term::Spelled(_) | Term::Bound(_) | Term::Applied(..) | Term::Opaque(_) => None,
        }
    }

    pub(crate) fn number(&self) -> Option<Quantity> {
        match self {
            Term::Number(value) => Some(*value),
            Term::Truth(_) | Term::Spelled(_) | Term::Bound(_) | Term::Applied(..) | Term::Opaque(_) => None,
        }
    }

    pub(crate) fn depths(&self) -> Depths {
        match self {
            Term::Bound(depth) => Depths::from(*depth),
            Term::Applied(_, arguments) => arguments.iter().map(Term::depths).collect(),
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) | Term::Opaque(_) => Depths::default(),
        }
    }

    pub(crate) fn matched(&self, target: &Term, found: &mut Substitution) -> Option<()> {
        match self {
            Term::Bound(depth) => found.bind(*depth, target),
            Term::Applied(callee, arguments) => callee.matched_against(arguments, target, found),
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) | Term::Opaque(_) => (self == target).then_some(()),
        }
    }
}

impl Substituted for Term {
    fn substituted(&self, values: &Substitution) -> Term {
        match self {
            Term::Bound(depth) => values.bound_value(*depth).cloned().unwrap_or_else(|| self.clone()),
            Term::Applied(callee, arguments) => callee.clone().folded(arguments.substituted(values)),
            Term::Number(_) | Term::Truth(_) | Term::Spelled(_) | Term::Opaque(_) => self.clone(),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Term::Number(value) => write!(formatter, "{value}"),
            Term::Truth(truth) => write!(formatter, "{truth}"),
            Term::Spelled(spelling) | Term::Opaque(spelling) => write!(formatter, "{spelling}"),
            Term::Bound(depth) => write!(formatter, "x{depth}"),
            Term::Applied(callee, arguments) => Applied { callee, arguments }.fmt(formatter),
        }
    }
}

pub(crate) struct Listing<'term>(Vec<&'term Term>);

impl<'term> FromIterator<&'term Term> for Listing<'term> {
    fn from_iter<Found: IntoIterator<Item = &'term Term>>(found: Found) -> Listing<'term> {
        Listing(found.into_iter().collect())
    }
}

impl fmt::Display for Listing<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        formatter.write_str(&spelled.join(", "))
    }
}

struct Parts<'term> {
    receiver: Listing<'term>,
    rest: Listing<'term>,
}

struct Applied<'term> {
    callee: &'term Callee,
    arguments: &'term [Term],
}

impl<'term> Applied<'term> {
    const RECEIVER: usize = 1;

    fn parts(&self) -> Parts<'term> {
        Parts {
            receiver: self.arguments.iter().take(Self::RECEIVER).collect(),
            rest: self.arguments.iter().skip(Self::RECEIVER).collect(),
        }
    }
}

impl fmt::Display for Applied<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.callee {
            Callee::Function(name) | Callee::Constructor(name) => {
                write!(formatter, "{name}({})", self.arguments.iter().collect::<Listing<'_>>())
            }
            Callee::Method(name) => write!(formatter, "{}.{name}({})", self.parts().receiver, self.parts().rest),
            Callee::Field(name) => write!(formatter, "{}.{name}", self.parts().receiver),
            Callee::Index => write!(formatter, "{}[{}]", self.parts().receiver, self.parts().rest),
            Callee::Sequence => write!(formatter, "[{}]", self.arguments.iter().collect::<Listing<'_>>()),
            Callee::Tuple => write!(formatter, "({})", self.arguments.iter().collect::<Listing<'_>>()),
            Callee::Operator(operator) => {
                write!(formatter, "{operator:?}({})", self.arguments.iter().collect::<Listing<'_>>())
            }
            Callee::Law(kind) => {
                write!(formatter, "the {kind} law of ({})", self.arguments.iter().collect::<Listing<'_>>())
            }
        }
    }
}
