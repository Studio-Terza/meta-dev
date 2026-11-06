use design_model::NodeId;
use design_store::{Placed, Resolved, Resolving, ShownNode};
use store_ports::Store;

use crate::cli::failure::Failure;

#[derive(Debug)]
pub struct Found(Placed);

impl Found {
    pub fn find(store: &impl Store, id: NodeId) -> Result<Found, Failure> {
        store.read(ShownNode::from(id))?.map(Found).ok_or(Failure::Unplaced(id))
    }

    pub fn resolved(store: &impl Store, id: NodeId) -> Result<Resolved, Failure> {
        store.read(Resolving::from(id))?.ok_or(Failure::Unplaced(id))
    }

    pub fn placed(&self) -> &Placed {
        &self.0
    }
}
