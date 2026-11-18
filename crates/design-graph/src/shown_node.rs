mod boundary;

use design_model::NodeId;
use port_core::Op;

#[derive(Clone, Copy, Debug, Op)]
pub struct ShownNode(NodeId);

impl From<NodeId> for ShownNode {
    fn from(id: NodeId) -> ShownNode {
        ShownNode(id)
    }
}
