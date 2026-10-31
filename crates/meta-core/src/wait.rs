use crate::event::Event;
use crate::ids::{OwnerAnswer, OwnerQuestion, TaskId};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};

impl Ledger {
    /// The task needs the owner: its lease deadlines pause until `unwait`.
    pub fn wait(&self, task: TaskId, question: OwnerQuestion) -> Result<Accepted, Refusal> {
        match self.tasks.get(&task) {
            Some(progress) if progress.is_active() && self.waiting.contains_key(&task) => Err(Refusal::because(
                format!("{progress} is already waiting on the owner. Next: meta unwait {task} <answer>."),
            )),
            Some(progress) if progress.is_active() => {
                let text = format!("{task} waits on the owner: {question}\n");
                Ok(Accepted::new(vec![Event::WaitingOnOwner { task, question }], text))
            }
            Some(other) => Err(Refusal::because(format!("{other}; only an open, claimed or linked task can wait."))),
            None => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }

    /// Records the owner's answer; the deadlines of the task's leases resume from this point.
    pub fn unwait(&self, task: TaskId, answer: OwnerAnswer) -> Result<Accepted, Refusal> {
        match (self.tasks.get(&task), self.waiting.contains_key(&task)) {
            (Some(_), true) => {
                let text = format!("{task} has the owner's answer, and its deadlines resume now: {answer}\n");
                Ok(Accepted::new(vec![Event::OwnerAnswered { task, answer }], text))
            }
            (Some(other), false) => Err(Refusal::because(format!("{other} is not waiting on the owner."))),
            (None, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }
}
