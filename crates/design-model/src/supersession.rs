use newtype::Newtype;

use crate::{Node, NodeId, Project};

const ENDS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Supersession {
    old: NodeId,
    by: NodeId,
}

#[derive(Debug, thiserror::Error)]
#[error(
    "n{} of project {old_project} cannot be superseded by n{} of project {by_project}: only consumes and generalizes edges cross project roots",
    old.into_inner(),
    by.into_inner()
)]
pub struct Straddling {
    old: NodeId,
    old_project: Project,
    by: NodeId,
    by_project: Project,
}

impl Supersession {
    pub fn within(old: &Node, by: &Node) -> Result<Supersession, Straddling> {
        let straddling = Straddling {
            old: old.identity(),
            old_project: old.entry().project().clone(),
            by: by.identity(),
            by_project: by.entry().project().clone(),
        };
        Some(old.entry().project())
            .filter(|project| *project == by.entry().project())
            .map(|_rooted| Supersession { old: old.identity(), by: by.identity() })
            .ok_or(straddling)
    }

    #[must_use]
    pub fn ends(self) -> [NodeId; ENDS] {
        [self.old, self.by]
    }
}
