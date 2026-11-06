use design_model::AnyNode;

use crate::{Link, Relative};

#[derive(Debug)]
pub struct Placed {
    node: AnyNode,
    relatives: Vec<Relative>,
    links: Vec<Link>,
}

impl Placed {
    pub(crate) fn new(node: AnyNode, relatives: Vec<Relative>, links: Vec<Link>) -> Placed {
        Placed { node, relatives, links }
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
}
