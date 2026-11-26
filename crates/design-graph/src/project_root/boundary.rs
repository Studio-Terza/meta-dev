use store_ports::{Execute, Query, StoreFailure};

use super::ProjectRoot;
use crate::Resolved;
use crate::graph::NodeRow;

impl Query for ProjectRoot {
    type Rows = Option<Resolved>;

    fn rows(self, statements: &mut impl Execute) -> Result<Option<Resolved>, StoreFailure> {
        NodeRow::root(&self.0, statements)
    }
}
