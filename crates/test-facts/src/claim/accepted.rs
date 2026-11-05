mod leaves;
mod single;

use std::collections::BTreeSet;
use std::fmt;

use lint_layout::{Name, Whole, Word, words};

use super::ranges::Ranges;
use super::term::{Arguments, Callee, Substituted, Substitution, Term};
use crate::boundary::Spelling;
pub(crate) use leaves::{Admitting, Exclusion, Inside, Mentions, Order, Relation, Values};
use single::Single;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Accepted {
    Any,
    Exactly(Values),
    Excluding(Exclusion),
    Within(Ranges),
    Shaped(Shape),
    Related(Relation),
    Written(Spelling),
    Joined(Union),
    Met(Intersection),
}

impl Accepted {
    pub(crate) fn equal(value: Term) -> Accepted {
        Accepted::Exactly(Values::from(value))
    }

    pub(crate) fn shaped(constructor: Name, fields: Vec<Accepted>) -> Accepted {
        Accepted::Shaped(Shape { constructor, fields })
    }

    pub(crate) fn unwrapped(inner: Accepted) -> Accepted {
        Accepted::shaped(Name::from(Unwrapping::UNWRAPPED), vec![inner])
    }

    pub(crate) fn consistent(&self) -> Option<&Accepted> {
        match self {
            Accepted::Within(ranges) => ranges.occupied().map(|_| self),
            Accepted::Met(intersection) => intersection.consistent().map(|_| self),
            Accepted::Any
            | Accepted::Exactly(_)
            | Accepted::Excluding(_)
            | Accepted::Shaped(_)
            | Accepted::Related(_)
            | Accepted::Written(_)
            | Accepted::Joined(_) => Some(self),
        }
    }

    pub(crate) fn ordered(order: Order, bound: Term) -> Accepted {
        let number = bound.number();
        number.map_or_else(
            move || Accepted::Related(Relation::new(order, bound)),
            |value| Accepted::Within(order.ranges(value)),
        )
    }

    pub(crate) fn met(self, other: Accepted) -> Accepted {
        [self, other].into_iter().map(Accepted::conjuncts).collect::<Conjuncts>().folded()
    }

    fn conjuncts(self) -> Conjuncts {
        match self {
            Accepted::Met(intersection) => Conjuncts(intersection.0.into_iter().collect()),
            Accepted::Any => Conjuncts::default(),
            other => Conjuncts(vec![other]),
        }
    }

    pub(crate) fn ranges(&self) -> Option<&Ranges> {
        match self {
            Accepted::Within(ranges) => Some(ranges),
            _ => None,
        }
    }

    pub(crate) fn sole(&self) -> Option<&Term> {
        match self {
            Accepted::Exactly(values) => values.sole(),
            _ => None,
        }
    }

    pub(crate) fn mentions(&self) -> Mentions<'_> {
        match self {
            Accepted::Exactly(values) => Mentions::from(values),
            Accepted::Excluding(exclusion) => Mentions::from(exclusion),
            Accepted::Related(relation) => Mentions::from(relation),
            Accepted::Shaped(shape) => shape.fields.iter().map(Accepted::mentions).collect(),
            Accepted::Joined(Union(parts)) | Accepted::Met(Intersection(parts)) => {
                parts.iter().map(Accepted::mentions).collect()
            }
            Accepted::Any | Accepted::Within(_) | Accepted::Written(_) => Mentions::default(),
        }
    }

    pub(crate) fn within(&self, outer: &Accepted) -> Option<Inside> {
        match self {
            Accepted::Exactly(values) => outer.admits_each(values),
            Accepted::Joined(union) => union.each_within(outer),
            Accepted::Met(intersection) => intersection.some_within(outer),
            Accepted::Any => outer.containing(Single::Any),
            Accepted::Within(ranges) => outer.containing(Single::Ranges(ranges)),
            Accepted::Excluding(exclusion) => outer.containing(Single::Exclusion(exclusion)),
            Accepted::Shaped(shape) => outer.containing(Single::Shape(shape)),
            Accepted::Related(relation) => outer.containing(Single::Relation(relation)),
            Accepted::Written(spelling) => outer.containing(Single::Written(spelling)),
        }
    }

    fn containing(&self, inner: Single<'_>) -> Option<Inside> {
        match self {
            Accepted::Any => Some(Inside),
            Accepted::Met(intersection) => intersection.all_containing(inner),
            Accepted::Joined(union) => union.any_containing(inner),
            Accepted::Within(ranges) => inner.in_ranges(ranges),
            Accepted::Excluding(exclusion) => inner.avoiding(exclusion),
            Accepted::Shaped(shape) => inner.in_shape(shape),
            Accepted::Related(relation) => inner.in_relation(relation),
            Accepted::Written(spelling) => inner.written_as(spelling),
            Accepted::Exactly(_) => None,
        }
    }
}

impl Admitting for Accepted {
    fn admits(&self, value: &Term) -> Option<Inside> {
        match self {
            Accepted::Any => Some(Inside),
            Accepted::Exactly(values) => values.admits(value),
            Accepted::Excluding(exclusion) => exclusion.admits(value),
            Accepted::Within(ranges) => value.number().and_then(|number| ranges.admits(number)).map(|_| Inside),
            Accepted::Shaped(shape) => shape.admits(value),
            Accepted::Joined(union) => union.admits(value),
            Accepted::Met(intersection) => intersection.admits(value),
            Accepted::Related(_) | Accepted::Written(_) => None,
        }
    }
}

impl Substituted for Accepted {
    fn substituted(&self, values: &Substitution) -> Accepted {
        match self {
            Accepted::Exactly(held) => Accepted::Exactly(held.substituted(values)),
            Accepted::Excluding(exclusion) => Accepted::Excluding(exclusion.substituted(values)),
            Accepted::Shaped(shape) => Accepted::Shaped(shape.substituted(values)),
            Accepted::Related(relation) => relation.resolved(values),
            Accepted::Joined(union) => Accepted::Joined(Union(union.0.substituted(values))),
            Accepted::Met(intersection) => Conjuncts(intersection.0.substituted(values).into_iter().collect()).folded(),
            Accepted::Any | Accepted::Within(_) | Accepted::Written(_) => self.clone(),
        }
    }
}

impl fmt::Display for Accepted {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Accepted::Any => formatter.write_str("is anything"),
            Accepted::Exactly(values) => write!(formatter, "equals {values}"),
            Accepted::Excluding(exclusion) => write!(formatter, "differs from {exclusion}"),
            Accepted::Within(ranges) => write!(formatter, "lies in {ranges}"),
            Accepted::Shaped(shape) => write!(formatter, "matches {shape}"),
            Accepted::Related(relation) => write!(formatter, "is {relation}"),
            Accepted::Written(spelling) => write!(formatter, "matches {spelling}"),
            Accepted::Joined(union) => {
                write!(formatter, "either {}", Spelled { parts: &union.0, separator: Separator::Or })
            }
            Accepted::Met(intersection) => {
                write!(formatter, "{}", Spelled { parts: &intersection.0, separator: Separator::And })
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Separator {
    Or,
    And,
}

impl fmt::Display for Separator {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Separator::Or => formatter.write_str(" or "),
            Separator::And => formatter.write_str(" and "),
        }
    }
}

struct Spelled<'parts> {
    parts: &'parts BTreeSet<Accepted>,
    separator: Separator,
}

impl fmt::Display for Spelled<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.parts.iter().map(Accepted::to_string).collect();
        formatter.write_str(&spelled.join(&self.separator.to_string()))
    }
}

#[derive(Debug, Default)]
struct Conjuncts(Vec<Accepted>);

impl FromIterator<Conjuncts> for Conjuncts {
    fn from_iter<Found: IntoIterator<Item = Conjuncts>>(found: Found) -> Conjuncts {
        Conjuncts(found.into_iter().flat_map(|conjuncts| conjuncts.0).collect())
    }
}

impl Conjuncts {
    fn folded(self) -> Accepted {
        let ranges = self
            .0
            .iter()
            .filter_map(Accepted::ranges)
            .fold(None, |held: Option<Ranges>, next| Some(held.map_or_else(|| next.clone(), |held| &held & next)));
        let kept: BTreeSet<Accepted> =
            self.0.into_iter().filter(|part| part.ranges().is_none()).chain(ranges.map(Accepted::Within)).collect();
        kept.iter().nth(1).map_or_else(
            || kept.first().cloned().unwrap_or(Accepted::Any),
            |_| Accepted::Met(Intersection(kept.clone())),
        )
    }
}

pub(crate) struct Unwrapping;

impl Unwrapping {
    const UNWRAPPED: Word<'static> = Word::new("Unwrapped");
    const VARIANTS: [Word<'static>; 2] = words!["Some", "Ok"];

    fn kin(own: &Name, other: &Name) -> Option<()> {
        let same = (own == other).then_some(());
        same.or(Self::standing(own, other).and(Some(()))).or(Self::standing(other, own).and(Some(())))
    }

    fn standing(unwrapped: &Name, variant: &Name) -> Option<Name> {
        (*unwrapped == Self::UNWRAPPED)
            .then_some(())
            .and_then(|()| variant.matches::<Whole>(&Self::VARIANTS))
            .map(|_| variant.clone())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Shape {
    constructor: Name,
    fields: Vec<Accepted>,
}

impl Shape {
    fn encloses(&self, inner: &Shape) -> Option<Inside> {
        Unwrapping::kin(&self.constructor, &inner.constructor)
            .map(|()| &inner.fields)
            .and_then(|fields| self.fields_enclose(fields))
    }

    fn fields_enclose(&self, inner: &[Accepted]) -> Option<Inside> {
        (inner.len() == self.fields.len()).then_some(())?;
        inner.iter().zip(&self.fields).try_for_each(|(field, wider)| field.within(wider).map(|_| ())).map(|()| Inside)
    }

    fn fields_admit(&self, arguments: Arguments<'_>) -> Option<Inside> {
        arguments.each_paired(&self.fields, |argument, field| field.admits(argument).map(|_| ())).map(|()| Inside)
    }
}

impl Admitting for Shape {
    fn admits(&self, value: &Term) -> Option<Inside> {
        let own = Callee::Constructor(self.constructor.clone())
            .arguments_of(value)
            .filter(|_| self.constructor != Unwrapping::UNWRAPPED);
        let variants = Unwrapping::VARIANTS
            .iter()
            .filter(|_| self.constructor == Unwrapping::UNWRAPPED)
            .find_map(|variant| Callee::Constructor(Name::from(*variant)).arguments_of(value));
        own.or(variants).and_then(|arguments| self.fields_admit(arguments))
    }
}

impl Substituted for Shape {
    fn substituted(&self, values: &Substitution) -> Shape {
        Shape { constructor: self.constructor.clone(), fields: self.fields.substituted(values) }
    }
}

impl fmt::Display for Shape {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let spelled: Vec<String> = self.fields.iter().map(Accepted::to_string).collect();
        write!(formatter, "{}({})", self.constructor, spelled.join(", "))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Union(BTreeSet<Accepted>);

impl FromIterator<Accepted> for Union {
    fn from_iter<Found: IntoIterator<Item = Accepted>>(found: Found) -> Union {
        Union(found.into_iter().collect())
    }
}

impl Union {
    fn each_within(&self, outer: &Accepted) -> Option<Inside> {
        self.0.iter().try_for_each(|case| case.within(outer).map(|_| ())).map(|()| Inside)
    }

    fn any_containing(&self, inner: Single<'_>) -> Option<Inside> {
        self.0.iter().find_map(|case| case.containing(inner))
    }
}

impl Admitting for Union {
    fn admits(&self, value: &Term) -> Option<Inside> {
        self.0.iter().find_map(|case| case.admits(value))
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct Intersection(BTreeSet<Accepted>);

impl Intersection {
    fn consistent(&self) -> Option<Inside> {
        self.0.iter().try_for_each(|part| part.consistent().map(|_| ())).map(|()| Inside)
    }

    fn some_within(&self, outer: &Accepted) -> Option<Inside> {
        outer
            .clone()
            .conjuncts()
            .0
            .iter()
            .all(|wider| self.0.iter().any(|part| part.within(wider).is_some()))
            .then_some(Inside)
    }

    fn all_containing(&self, inner: Single<'_>) -> Option<Inside> {
        self.0.iter().try_for_each(|part| part.containing(inner).map(|_| ())).map(|()| Inside)
    }
}

impl Admitting for Intersection {
    fn admits(&self, value: &Term) -> Option<Inside> {
        self.0.iter().try_for_each(|part| part.admits(value).map(|_| ())).map(|()| Inside)
    }
}
