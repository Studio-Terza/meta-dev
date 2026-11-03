mod reach_table;

use std::path::PathBuf;

use serde::Deserialize;

use reach_table::ReachTable;

#[derive(Debug, Deserialize)]
pub struct Declared {
    reach: Option<ReachTable>,
}

impl Declared {
    pub fn reads(&self) -> &[PathBuf] {
        self.reach.as_ref().map_or(&[], ReachTable::as_ref)
    }
}
