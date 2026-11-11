use std::fmt;

use crate::Relation;
use crate::anchor::Anchor;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MechanicalLink {
    source: Anchor,
    relation: Relation,
    target: Anchor,
}

impl MechanicalLink {
    pub const ENDS: usize = 2;

    pub(crate) fn new(source: Anchor, relation: Relation, target: Anchor) -> MechanicalLink {
        MechanicalLink { source, relation, target }
    }

    #[must_use]
    pub fn relation(&self) -> Relation {
        self.relation
    }

    #[must_use]
    pub fn anchors(&self) -> [&Anchor; Self::ENDS] {
        [&self.source, &self.target]
    }
}

impl fmt::Display for MechanicalLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.source, self.relation, self.target)
    }
}
