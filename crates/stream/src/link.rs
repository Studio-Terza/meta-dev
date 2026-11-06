use std::fmt;

use transcript_ports::Uuid;

use crate::Relation;
use crate::anchor::Anchor;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MechanicalLink {
    source: Anchor,
    relation: Relation,
    target: Anchor,
}

impl MechanicalLink {
    pub(crate) fn new(source: Anchor, relation: Relation, target: Anchor) -> MechanicalLink {
        MechanicalLink { source, relation, target }
    }

    #[must_use]
    pub fn relation(&self) -> Relation {
        self.relation
    }

    #[must_use]
    pub fn touching(&self, message: &Uuid) -> Option<&MechanicalLink> {
        [&self.source, &self.target].into_iter().find(|anchor| anchor.message() == message).map(|_| self)
    }
}

impl fmt::Display for MechanicalLink {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {} {}", self.source, self.relation, self.target)
    }
}
