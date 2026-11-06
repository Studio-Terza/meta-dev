use std::fmt;

use design_model::NodeId;
use design_store::{Placed, ShownNode};
use store_ports::Store;

use crate::cli::failure::Failure;
use crate::cli::rendering::{Id, Line};

#[derive(Debug)]
pub struct Neighbourhood(Placed);

impl Neighbourhood {
    pub fn find(store: &impl Store, id: NodeId) -> Result<Neighbourhood, Failure> {
        store.read(ShownNode::from(id))?.map(Neighbourhood).ok_or(Failure::Unplaced(id))
    }
}

impl fmt::Display for Neighbourhood {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(formatter, "{}", Line::from(self.0.node()))?;
        self.0
            .relatives()
            .iter()
            .try_for_each(|relative| writeln!(formatter, "  {} {}", relative.kin(), Line::from(relative.node())))?;
        self.0
            .links()
            .iter()
            .try_for_each(|link| writeln!(formatter, "  {} {}", link.relation(), Id::from(link.target())))
    }
}
