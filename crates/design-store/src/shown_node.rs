use design_model::NodeId;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::acyclic::Refines;
use crate::graph::{EdgeRow, NodeRow, RelativeRow};
use crate::{Placed, Placement};

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
        let relatives = RelativeRow::around::<Refines>(self.0, statements)?;
        let links = EdgeRow::links(self.0, statements)?;
        let placements = Placement::placed_on(self.0, statements)?;
        Ok(nodes.into_iter().next().map(|node| Placed::new(node, relatives, links, placements)))
    }
}
