use store_ports::{Query, Statements, StoreFailure};

use super::ProjectRoot;
use crate::Resolved;
use crate::graph::NodeRow;

impl Query for ProjectRoot {
    type Rows = Option<Resolved>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Resolved>, StoreFailure> {
        NodeRow::root(&self.0, statements)
    }
}
