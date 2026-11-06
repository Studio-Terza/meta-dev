use design_model::Node;

use crate::Link;

#[derive(Debug)]
pub struct Placed {
    node: Node,
    links: Vec<Link>,
}

impl Placed {
    pub(crate) fn new(node: Node, links: Vec<Link>) -> Placed {
        Placed { node, links }
    }

    #[must_use]
    pub fn node(&self) -> &Node {
        &self.node
    }

    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }
}
