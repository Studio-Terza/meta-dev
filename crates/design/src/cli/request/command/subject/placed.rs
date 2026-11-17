use crate::cli::request::command::printed::Lines;

use design_model::NodeId;
use design_store::{Placed, Resolved, Resolving, ShownNode};
use store_ports::Store;

use crate::cli::failure::Failure;
use crate::cli::request::command::rendering::Refs;
use console_ports::Shown;

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

    pub fn printed(&self, store: &impl Store) -> Result<Shown, Failure> {
        Ok(Shown::said(Lines::from(&Refs::placed(store, &self.0)?)))
    }
}
