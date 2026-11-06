use design_model::{NodeId, Project};
use port_core::Op;
use store_ports::{Query, Statements, StoreFailure};

use crate::graph::NodeRow;

#[derive(Clone, Debug, Op)]
pub struct ProjectRoot(Project);

impl From<Project> for ProjectRoot {
    fn from(project: Project) -> ProjectRoot {
        ProjectRoot(project)
    }
}

impl Query for ProjectRoot {
    type Rows = Option<NodeId>;

    fn rows(self, statements: &mut Statements<'_>) -> Result<Option<NodeId>, StoreFailure> {
        Ok(NodeRow::root(&self.0, statements)?.into_iter().next())
    }
}
