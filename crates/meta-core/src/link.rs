use crate::event::Event;
use crate::ids::{PrNumber, TaskId};
use crate::ledger::Ledger;
use crate::pull::Pull;
use crate::reply::{Accepted, Refusal};
use crate::task::Progress;

#[derive(Debug)]
pub struct Linking {
    task: TaskId,
    pr: PrNumber,
}

impl Ledger {
    const MAIN: &str = "main";

    pub fn link(&self, task: TaskId, pr: PrNumber) -> Result<Linking, Refusal> {
        let other = self.linked(pr).map(|linked| linked.id()).filter(|linked| *linked != task);
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
    pub fn pin(self, pull: &Pull) -> Result<Accepted, Refusal> {
        let Self { task, pr } = self;
        let branch = task.to_string();
        let wrong_branch = (pull.branch().to_string() != branch).then(|| {
            format!("{pr} comes from branch {}, not {branch}. Next: push the work to branch {branch}.", pull.branch())
        });
        let wrong_base = (pull.base().to_string() != Ledger::MAIN).then(|| {
            format!("{pr} targets {}, not {}. Next: retarget it to {}.", pull.base(), Ledger::MAIN, Ledger::MAIN)
        });
        let problems: Vec<String> = wrong_branch.into_iter().chain(wrong_base).collect();
        match problems.is_empty() {
            false => Err(Refusal::because(format!("{task} cannot link {pr}:\n{}", problems.join("\n")))),
            true => {
                let (pin, repo) = (pull.head().clone(), pull.repo().clone());
                let text = format!("{task} linked to {pr} in {repo}, head pinned at {pin}\n");
                Ok(Accepted::new(vec![Event::Linked { task, pr, pin, repo }], text))
            }
        }
    }
}
