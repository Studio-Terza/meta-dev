use design_model::AnyNode;

use crate::{Link, Placement, Relative};

#[derive(Debug)]
pub struct Placed {
    node: AnyNode,
    relatives: Vec<Relative>,
    links: Vec<Link>,
    placements: Vec<Placement>,
}

impl Placed {
    pub(crate) fn new(node: AnyNode, relatives: Vec<Relative>, links: Vec<Link>, placements: Vec<Placement>) -> Placed {
        Placed { node, relatives, links, placements }
    }

    #[must_use]
    pub fn node(&self) -> &AnyNode {
        &self.node
    }

    #[must_use]
    pub fn relatives(&self) -> &[Relative] {
        &self.relatives
    }

    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    #[must_use]
    pub fn placements(&self) -> &[Placement] {
        &self.placements
    }
}
