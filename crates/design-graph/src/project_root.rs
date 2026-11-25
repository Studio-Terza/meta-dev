mod boundary;

use design_model::Project;
use port_core::Op;

#[derive(Clone, Debug, Op)]
pub struct ProjectRoot(Project);

impl From<Project> for ProjectRoot {
    fn from(project: Project) -> ProjectRoot {
        ProjectRoot(project)
    }
}
