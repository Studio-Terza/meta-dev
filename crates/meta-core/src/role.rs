use crate::event::Event;
use crate::ids::{Malformed, TaskId};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// What a task is, beyond work to finish. The critic's task is long-running by design: none of
/// its leases, past or future, is running work, and the task is never stalled. A project's task is
/// the project's root: its id names the project, and its agent runs while the task is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Critic,
    Project,
}

impl FromStr for Role {
    type Err = Malformed;

    fn from_str(text: &str) -> Result<Self, Malformed> {
        match text {
            "critic" => Ok(Self::Critic),
            "project" => Ok(Self::Project),
            other => Err(Malformed::because(format!("{other:?} is not a role; the roles are critic and project"))),
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Critic => "critic",
            Self::Project => "project",
        })
    }
}

impl Role {
    /// What the mark means, as `meta role` says it.
    fn meaning(self) -> &'static str {
        match self {
            Self::Critic => "its leases are not running work and it is never stalled",
            Self::Project => "it is a project's root and its agent's lease runs while it is active",
        }
    }
}

impl Ledger {
    /// Mark a task with its role. The mark is the task's, so it covers every lease on the task. A
    /// task has one role, and a project's member has none: projects do not nest.
    pub fn role(&self, task: TaskId, role: Role) -> Result<Accepted, Refusal> {
        match (self.tasks.get(&task), self.role_of(task), self.project_of(task)) {
            (None, _, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
            (Some(_), Some(held), _) => Err(Refusal::because(format!("{task} is already marked {held}."))),
            (Some(_), None, Some(project)) => {
                Err(Refusal::because(format!("{task} is a member of {project}, so it takes no role.")))
            }
            (Some(_), None, None) => Ok(Accepted::new(
                vec![Event::TaskRole { task, role }],
                format!("{task} is marked {role}: {}\n", role.meaning()),
            )),
        }
    }

    /// The task's role, if it was marked with one.
    pub(crate) fn role_of(&self, task: TaskId) -> Option<Role> {
        match (self.critics.contains(&task), self.projects.contains_key(&task)) {
            (true, _) => Some(Role::Critic),
            (false, true) => Some(Role::Project),
            (false, false) => None,
        }
    }
}
