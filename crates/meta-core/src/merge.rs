use crate::event::Event;
use crate::ids::{PrNumber, RunCount, Sha, TaskId};
use crate::ledger::Ledger;
use crate::pull::{Outcome, Pull};
use crate::reply::{Accepted, Refusal};
use crate::task::Progress;

#[derive(Debug)]
pub struct Merging {
    task: TaskId,
    pr: PrNumber,
    pin: Sha,
}

#[derive(Debug)]
pub struct Cleared {
    task: TaskId,
    pr: PrNumber,
    head: Sha,
    runs: RunCount,
}

impl Ledger {
    pub fn merge(&self, pr: PrNumber) -> Result<Merging, Refusal> {
        let linked = self.links.get(&pr).and_then(|task| match self.tasks.get(task) {
            Some(Progress::Linked(linked)) => Some(linked),
            _ => None,
        });
        linked.map(|task| Merging { task: task.id(), pr, pin: task.pin().clone() }).ok_or_else(|| {
            Refusal::because(format!(
                "{pr} is unlinked. Next: meta link <task> {} from the agent that claimed the task.",
                pr.digits()
            ))
        })
    }
}

impl Merging {
    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    pub fn clear(self, pull: &Pull) -> Result<Cleared, Refusal> {
        let Self { task, pr, pin } = self;
        let relink = format!("meta link {task} {}", pr.digits());
        let moved = (pull.head != pin).then(|| {
            format!("head moved from pinned {pin} to {}. Next: review the new head, then {relink}.", pull.head)
        });
        let unconfigured =
            pull.required.is_empty().then(|| "missing check: no required checks are configured.".to_owned());
        let checks = pull.required.iter().filter_map(|name| match pull.outcome(name) {
            Some(Outcome::Green) => None,
            Some(Outcome::Red) => {
                Some(format!("red check {name} on {}. Next: fix it, push, then {relink}.", pull.head))
            }
            Some(Outcome::Pending) | None => Some(format!(
                "missing check {name} on {}. Next: wait for CI, then meta merge {} again.",
                pull.head,
                pr.digits()
            )),
        });
        let blockers: Vec<String> = moved.into_iter().chain(unconfigured).chain(checks).collect();
        match blockers.is_empty() {
            true => Ok(Cleared { task, pr, head: pull.head.clone(), runs: pull.runs }),
            false => Err(Refusal::because(format!("{pr} cannot merge:\n{}", blockers.join("\n")))),
        }
    }
}

impl Cleared {
    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    pub fn head(&self) -> &Sha {
        &self.head
    }

    pub fn merged(self) -> Accepted {
        let Self { task, pr, head, runs } = self;
        let text = format!("{pr} squash-merged at {head}; {task} done\n");
        Accepted { events: vec![Event::Merged { pr, head, runs }, Event::Done { task }], text }
    }
}
