use design_model::AnyNode;

use crate::{Citation, Link, Placement, Relative};

#[derive(Debug)]
pub struct Placed {
    node: AnyNode,
    relatives: Vec<Relative>,
    links: Vec<Link>,
    placements: Vec<Placement>,
    citations: Vec<Citation>,
}

impl Placed {
    pub(crate) fn new(node: AnyNode, relatives: Vec<Relative>, links: Vec<Link>, placements: Vec<Placement>) -> Placed {
        Placed { node, relatives, links, placements, citations: Vec::new() }
    }

    pub(crate) fn cited(self, citations: Vec<Citation>) -> Placed {
        Placed { citations, ..self }
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

    #[must_use]
    pub fn citations(&self) -> &[Citation] {
        &self.citations
    }
}
