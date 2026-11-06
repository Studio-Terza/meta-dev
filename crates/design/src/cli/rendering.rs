use std::fmt;

use design_model::{Entered, Node, NodeId};
use newtype::Newtype;

#[derive(Clone, Copy, Debug)]
pub struct Id(NodeId);

#[derive(Clone, Copy, Debug)]
pub struct Line<'node>(&'node Node);

impl From<NodeId> for Id {
    fn from(id: NodeId) -> Id {
        Id(id)
    }
}

impl<'node> From<&'node Node> for Line<'node> {
    fn from(node: &'node Node) -> Line<'node> {
        Line(node)
    }
}

impl fmt::Display for Id {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "n{}", self.0.into_inner())
    }
}

impl fmt::Display for Line<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entry = self.0.entry();
        write!(
            formatter,
            "{} {} {} {} entered {}",
            Id(self.0.identity()),
            entry.project(),
            entry.facet(),
            entry.stage(),
            Entered::from(entry.provenance())
        )
    }
}
