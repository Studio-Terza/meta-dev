use std::collections::BTreeSet;

use design_model::NodeId;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::graph::NodeRow;

#[derive(Clone, Copy, Debug, Op)]
pub struct SupersededBy(NodeId);

impl From<NodeId> for SupersededBy {
    fn from(id: NodeId) -> SupersededBy {
        SupersededBy(id)
    }
}

impl Query for SupersededBy {
    type Rows = BTreeSet<NodeId>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<BTreeSet<NodeId>, StoreFailure> {
        Ok(NodeRow::superseded_by(self.0, statements)?.into_iter().collect())
    }
}
