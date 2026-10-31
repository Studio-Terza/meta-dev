use crate::event::Event;
use crate::ids::{PrNumber, TaskId};
use crate::ledger::Ledger;
use crate::pull::{Pull, PullState};
use crate::reply::{Accepted, Refusal};
use crate::task::Progress;

/// A task the ledger lets another task's PR finish, once GitHub says that PR is merged.
#[derive(Debug)]
pub struct Finishing {
    task: TaskId,
    by: PrNumber,
}

impl Ledger {
    pub fn done_by(&self, task: TaskId, by: PrNumber) -> Result<Finishing, Refusal> {
        match self.tasks.get(&task) {
            Some(Progress::Open(_) | Progress::Claimed(_) | Progress::Linked(_)) => Ok(Finishing { task, by }),
            Some(deferred @ Progress::Deferred(_)) => Err(Refusal::because(format!(
                "{deferred}; only an open, claimed or linked task can be done by a PR. Next: meta undefer {task}, then meta done again."
            ))),
            Some(over @ (Progress::Done(_) | Progress::Retired(_))) => {
                Err(Refusal::because(format!("{over}; only an open, claimed or linked task can be done by a PR.")))
            }
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }
}

impl Finishing {
    pub fn pr(&self) -> PrNumber {
        self.by
    }

    /// Records the task done by the PR; `pull` is that PR as GitHub reports it.
    pub fn settle(self, pull: &Pull) -> Result<Accepted, Refusal> {
        let Self { task, by } = self;
        match pull.state() {
            PullState::Merged => {
                let text = format!("{task} done by {by}\n");
                Ok(Accepted::new(vec![Event::Done { task, by: Some(by) }], text))
            }
            PullState::Open => Err(Refusal::because(format!(
                "{by} is not merged yet. Next: merge it, then meta done {task} --by {} again.",
                by.digits()
            ))),
            PullState::Closed => Err(Refusal::because(format!(
                "{by} is closed without merging. Next: name the PR that was merged, or meta retire {task} <reason>."
            ))),
        }
    }
}
