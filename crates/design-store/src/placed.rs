use design_model::Node;

use crate::{Link, Relative};

#[derive(Debug)]
pub struct Placed {
    node: Node,
    relatives: Vec<Relative>,
    links: Vec<Link>,
}

impl Placed {
    pub(crate) fn new(node: Node, relatives: Vec<Relative>, links: Vec<Link>) -> Placed {
        Placed { node, relatives, links }
    }

    #[must_use]
    pub fn node(&self) -> &Node {
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
