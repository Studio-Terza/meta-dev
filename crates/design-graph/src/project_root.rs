use design_model::Project;
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::Resolved;
use crate::graph::NodeRow;

#[derive(Clone, Debug, Op)]
pub struct ProjectRoot(Project);

impl From<Project> for ProjectRoot {
    fn from(project: Project) -> ProjectRoot {
        ProjectRoot(project)
    }
}

impl Query for ProjectRoot {
    type Rows = Option<Resolved>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<Resolved>, StoreFailure> {
        NodeRow::root(&self.0, statements)
    }
}
