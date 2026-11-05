use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

use law_facts::Kind;

use super::product::Product;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Fact {
    Holds,
    Shape,
    Value,
    Distinct,
    Law(Kind),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    Holds,
    Pattern,
    Differs,
    Equal,
    Law(Kind),
}

impl Relation {
    fn facts(self) -> BTreeSet<Fact> {
        match self {
            Relation::Holds => BTreeSet::from([Fact::Holds]),
            Relation::Pattern => BTreeSet::from([Fact::Holds, Fact::Shape]),
            Relation::Equal => BTreeSet::from([Fact::Holds, Fact::Shape, Fact::Value]),
            Relation::Differs => BTreeSet::from([Fact::Holds, Fact::Distinct]),
            Relation::Law(kind) => BTreeSet::from([Fact::Law(kind)]),
        }
    }
}

impl PartialOrd for Relation {
    fn partial_cmp(&self, other: &Relation) -> Option<Ordering> {
        let mine = self.facts();
        let theirs = other.facts();
        Product::combined(mine.union(&theirs).map(|fact| Some(mine.contains(fact).cmp(&theirs.contains(fact)))))
    }
}

impl fmt::Display for Relation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Relation::Holds => formatter.write_str("a condition"),
            Relation::Pattern => formatter.write_str("a pattern match"),
            Relation::Differs => formatter.write_str("an inequality"),
            Relation::Equal => formatter.write_str("an equality"),
            Relation::Law(kind) => write!(formatter, "the {kind} law"),
        }
    }
}
