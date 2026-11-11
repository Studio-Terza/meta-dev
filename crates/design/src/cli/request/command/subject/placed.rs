use std::io::Write;

use design_model::NodeId;
use design_store::{Placed, Resolved, Resolving, ShownNode};
use store_ports::Store;

use crate::cli::failure::Failure;
use crate::cli::request::command::printed::Printed;
use crate::cli::request::command::rendering::Refs;

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

    pub fn printed(&self, store: &impl Store, out: &mut impl Write) -> Result<Printed, Failure> {
        Printed::lines(out, &Refs::placed(store, &self.0)?)
    }
}
