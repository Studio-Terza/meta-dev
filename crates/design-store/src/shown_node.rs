use design_model::NodeId;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::Placed;
use crate::graph::{NodeRow, OutgoingRow};

#[derive(Clone, Copy, Debug, Op)]
pub struct ShownNode(NodeId);

impl From<NodeId> for ShownNode {
    fn from(id: NodeId) -> ShownNode {
        ShownNode(id)
    }
}

impl Query for ShownNode {
    type Rows = Option<Placed>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Placed>, StoreFailure> {
        let nodes = NodeRow::named(self.0, statements)?;
        let links = OutgoingRow::from_node(self.0, statements)?;
        Ok(nodes.into_iter().next().map(|node| Placed::new(node, links)))
    }
}
