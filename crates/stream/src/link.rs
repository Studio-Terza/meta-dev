use crate::{Anchor, Relation};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct MechanicalLink {
    pub from: Anchor,
    pub relation: Relation,
    pub to: Anchor,
}
