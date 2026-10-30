use crate::event::Event;
use crate::ids::{PrNumber, Sha, TaskId};
use crate::ledger::Ledger;
use crate::reply::{Accepted, Refusal};
use crate::task::Progress;

#[derive(Debug)]
pub struct Linking {
    task: TaskId,
    pr: PrNumber,
}

impl Ledger {
    pub fn link(&self, task: TaskId, pr: PrNumber) -> Result<Linking, Refusal> {
        let other = self.links.get(&pr).filter(|linked| **linked != task);
        match (self.tasks.get(&task), other) {
            (_, Some(other)) => Err(Refusal::because(format!("{pr} is already linked to {other}."))),
            (Some(Progress::Claimed(_) | Progress::Linked(_)), None) => Ok(Linking { task, pr }),
            (Some(unclaimed), None) => Err(Refusal::because(format!(
                "{unclaimed}; only a claimed task can be linked. Next: meta claim {task} --agent <name>."
            ))),
            (None, None) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }
}

impl Linking {
    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    pub fn pin(self, head: Sha) -> Accepted {
        let Self { task, pr } = self;
        let text = format!("{task} linked to {pr}, head pinned at {head}\n");
        Accepted { events: vec![Event::Linked { task, pr, pin: head }], text }
    }
}
