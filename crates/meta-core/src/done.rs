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
    own: Option<PrNumber>,
}

impl Ledger {
    pub fn done_by(&self, task: TaskId, by: PrNumber) -> Result<Finishing, Refusal> {
        match self.tasks.get(&task) {
            Some(Progress::Open(_) | Progress::Claimed(_)) => Ok(Finishing { task, by, own: None }),
            Some(Progress::Linked(linked)) if linked.pr() == by => Err(Refusal::because(format!(
                "{by} is {task}'s own linked PR. Next: meta merge {} records it and marks the task done.",
                by.digits()
            ))),
            Some(Progress::Linked(linked)) => Ok(Finishing { task, by, own: Some(linked.pr()) }),
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

    /// The task's own linked PR, whose state `settle` also needs.
    pub fn own(&self) -> Option<PrNumber> {
        self.own
    }

    /// Records the task done by the PR; `pull` is that PR as GitHub reports it,
    /// `own` the task's own linked PR when it has one.
    pub fn settle(self, pull: &Pull, own: Option<&Pull>) -> Result<Accepted, Refusal> {
        let Self { task, by, own: own_pr } = self;
        let own_state = own_pr.map(|pr| (pr, own.map(Pull::state)));
        match (pull.state(), own_state) {
            (PullState::Merged, Some((pr, Some(PullState::Open)))) => Err(Refusal::because(format!(
                "{pr} is linked and open. Next: close it with gh pr close {}, then meta done {task} --by {} again.",
                pr.digits(),
                by.digits()
            ))),
            (PullState::Merged, Some((pr, Some(PullState::Merged)))) => Err(Refusal::because(format!(
                "{pr} is merged. Next: meta merge {} records it and marks the task done.",
                pr.digits()
            ))),
            (PullState::Merged, Some((pr, None))) => Err(Refusal::because(format!(
                "the state of {pr} is unknown. Next: meta done {task} --by {} again.",
                by.digits()
            ))),
            (PullState::Merged, _) => {
                let text = format!("{task} done by {by}\n");
                Ok(Accepted::new(vec![Event::Done { task, by: Some(by) }], text))
            }
            (PullState::Open, _) => Err(Refusal::because(format!(
                "{by} is not merged yet. Next: merge it, then meta done {task} --by {} again.",
                by.digits()
            ))),
            (PullState::Closed, _) => Err(Refusal::because(format!(
                "{by} is closed without merging. Next: name the PR that was merged, or meta retire {task} <reason>."
            ))),
        }
    }
}
