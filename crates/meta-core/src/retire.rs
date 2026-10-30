use crate::event::Event;
use crate::ids::{PrNumber, Reason, TaskId};
use crate::ledger::Ledger;
use crate::pull::{Pull, PullState};
use crate::reply::{Accepted, Refusal};
use crate::task::Progress;

/// A retirement the ledger allows. A linked task still needs its PR's state from GitHub.
#[derive(Debug)]
pub enum Retiring {
    Unlinked(Retirement),
    Linked { retirement: Retirement, pr: PrNumber },
}

#[derive(Debug)]
pub struct Retirement {
    task: TaskId,
    reason: Reason,
}

impl Ledger {
    pub fn retire(&self, task: TaskId, reason: Reason) -> Result<Retiring, Refusal> {
        let retirement = Retirement { task, reason };
        match self.tasks.get(&task) {
            Some(Progress::Open(_) | Progress::Claimed(_)) => Ok(Retiring::Unlinked(retirement)),
            Some(Progress::Linked(linked)) => Ok(Retiring::Linked { retirement, pr: linked.pr() }),
            Some(deferred @ Progress::Deferred(_)) => Err(Refusal::because(format!(
                "{deferred}; only an open or claimed task can be retired. Next: meta undefer {task}, then retire it."
            ))),
            Some(over @ (Progress::Done(_) | Progress::Retired(_))) => {
                Err(Refusal::because(format!("{over}; only an open or claimed task can be retired.")))
            }
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }

    pub fn defer(&self, task: TaskId, reason: Reason) -> Result<Accepted, Refusal> {
        match self.tasks.get(&task) {
            Some(Progress::Open(_)) => {
                let text = format!("{task} deferred: {reason}\n");
                Ok(Accepted::new(vec![Event::Deferred { task, reason }], text))
            }
            Some(other) => Err(Refusal::because(format!("{other}; only an open task can be deferred."))),
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }

    pub fn undefer(&self, task: TaskId) -> Result<Accepted, Refusal> {
        match self.tasks.get(&task) {
            Some(Progress::Deferred(_)) => {
                Ok(Accepted::new(vec![Event::Undeferred { task }], format!("{task} is open again\n")))
            }
            Some(other) => Err(Refusal::because(format!("{other}; only a deferred task can be undeferred."))),
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }
}

impl Retiring {
    pub fn pr(&self) -> Option<PrNumber> {
        match self {
            Self::Unlinked(_) => None,
            Self::Linked { pr, .. } => Some(*pr),
        }
    }

    /// Retires the task. `pull` is the linked PR as GitHub reports it; an unlinked task needs none.
    pub fn settle(self, pull: Option<&Pull>) -> Result<Accepted, Refusal> {
        let digits = |pr: PrNumber| pr.digits();
        match (self, pull.map(Pull::state)) {
            (Self::Unlinked(retirement), _) | (Self::Linked { retirement, .. }, Some(PullState::Closed)) => {
                Ok(retirement.record())
            }
            (Self::Linked { pr, .. }, Some(PullState::Open)) => Err(Refusal::because(format!(
                "{pr} is linked and open. Next: close it with gh pr close {}, then retire the task again.",
                digits(pr)
            ))),
            (Self::Linked { pr, .. }, Some(PullState::Merged)) => Err(Refusal::because(format!(
                "{pr} is merged. Next: meta merge {} records it and marks the task done.",
                digits(pr)
            ))),
            (Self::Linked { pr, .. }, None) => {
                Err(Refusal::because(format!("the state of {pr} is unknown. Next: meta retire again.")))
            }
        }
    }
}

impl Retirement {
    fn record(self) -> Accepted {
        let Self { task, reason } = self;
        let text = format!("{task} retired: {reason}\n");
        Accepted::new(vec![Event::Retired { task, reason }], text)
    }
}
