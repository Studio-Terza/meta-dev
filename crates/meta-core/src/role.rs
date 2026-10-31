use crate::event::Event;
use crate::ids::{Malformed, TaskId};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// What a task is, beyond work to finish. The critic's task is long-running by design: none of
/// its leases, past or future, is running work, and the task is never stalled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Critic,
}

impl FromStr for Role {
    type Err = Malformed;

    fn from_str(text: &str) -> Result<Self, Malformed> {
        match text {
            "critic" => Ok(Self::Critic),
            other => Err(Malformed::because(format!("{other:?} is not a role; the only role is critic"))),
        }
    }
}

impl Ledger {
    /// Mark a task with its role. The mark is the task's, so it covers every lease on the task.
    pub fn role(&self, task: TaskId, role: Role) -> Result<Accepted, Refusal> {
        match (self.tasks.get(&task), self.critics.contains(&task)) {
            (None, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
            (Some(_), true) => Err(Refusal::because(format!("{task} is already marked critic."))),
            (Some(_), false) => Ok(Accepted::new(
                vec![Event::TaskRole { task, role }],
                format!("{task} is marked critic: its leases are not running work and it is never stalled\n"),
            )),
        }
    }
}
