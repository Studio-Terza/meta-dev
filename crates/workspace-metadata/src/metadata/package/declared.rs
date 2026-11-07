mod port_table;
mod reach_table;

use std::path::PathBuf;

use serde::Deserialize;

pub use port_table::PortTable;
use reach_table::ReachTable;

#[derive(Clone, Debug, Deserialize)]
pub struct Declared {
    reach: Option<ReachTable>,
    port: Option<PortTable>,
}

impl Declared {
    pub fn reads(&self) -> &[PathBuf] {
        self.reach.as_ref().map_or(&[], ReachTable::as_ref)
    }

    pub fn port(&self) -> Option<&PortTable> {
        self.port.as_ref()
    }
}
