use crate::brief::ReviewText;
use crate::event::Event;
use crate::ids::{AgentName, LeaseId, PrNumber, Sha, TaskId, Timestamp, Title, Verdict};
use crate::lease::{Deadline, LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::pull::{Pull, PullState};
use crate::reply::{Accepted, Refusal};
use crate::rules::Rules;
use crate::task::Progress;

/// A review the ledger allows; the PR must still be open on GitHub.
#[derive(Debug)]
pub struct Reviewing {
    task: TaskId,
    title: Title,
    agent: AgentName,
    lease: LeaseId,
    pr: PrNumber,
}

impl Ledger {
    pub fn review(&self, task: TaskId, agent: AgentName) -> Result<Reviewing, Refusal> {
        let lease = LeaseId::after(self.leases.len());
        let pending = self.leases.iter().find_map(|(id, slot)| match slot {
            LeaseSlot::Unused(unused) if unused.task() == task && *unused.purpose() != Purpose::Work => Some(*id),
            _ => None,
        });
        match (self.tasks.get(&task), pending) {
            (Some(Progress::Linked(_)), Some(unused)) => Err(Refusal::because(format!(
                "review lease {unused} on {task} is unused. Next: meta brief {unused} prints its brief again; put {unused} in the reviewer's Agent description."
            ))),
            (Some(Progress::Linked(linked)), None) => {
                Ok(Reviewing { task, title: linked.title().clone(), agent, lease, pr: linked.pr() })
            }
            (Some(other), _) => Err(Refusal::because(format!(
                "{other}; only a task linked to an open PR can be reviewed. Next: meta link {task} <pr>."
            ))),
            (None, _) => Err(Refusal::because(format!("{task} is unknown. Next: meta status."))),
        }
    }

    /// The PR a used review lease of `task` awaits a verdict on: the one to ask GitHub about.
    pub fn awaiting(&self, task: TaskId) -> Result<PrNumber, Refusal> {
        self.awaiting_lease(task).map(|(_, pr, _)| pr)
    }

    /// The first used review lease of `task` with no verdict, and the PR and head it reviews.
    fn awaiting_lease(&self, task: TaskId) -> Result<(LeaseId, PrNumber, Sha), Refusal> {
        self.leases
            .iter()
            .find_map(|(id, slot)| match slot {
                LeaseSlot::Spent(spent) if spent.lease().task() == task && spent.verdict().is_none() => {
                    match spent.lease().purpose() {
                        Purpose::Review { pr, head } => Some((*id, *pr, head.clone())),
                        Purpose::Work => None,
                    }
                }
                _ => None,
            })
            .ok_or_else(|| {
                Refusal::because(format!(
                    "no used review lease on {task} awaits a verdict. Next: meta review {task} --agent <name>, \
                     start the reviewer with that lease, then the reviewer records its verdict."
                ))
            })
    }

    /// Records the verdict on the review lease of `task`, given the head the PR has now: the head
    /// the verdict judged must be the one the lease was issued at.
    pub fn verdict(&self, task: TaskId, verdict: Verdict, now: &Pull) -> Result<Accepted, Refusal> {
        let (lease, pr, head) = self.awaiting_lease(task)?;
        match now.is_at(&head) {
            true => {
                let text = format!("{task}: verdict {verdict} recorded for review lease {lease} at head {head}\n");
                Ok(Accepted::new(vec![Event::Reviewed { lease, verdict, head: Some(head) }], text))
            }
            false => Err(Refusal::because(format!(
                "{pr} is at head {}, but review lease {lease} judged head {head}; a verdict must land on the head it judged. \
                 Next: meta review {task} --agent <name> starts a review of the current head.",
                now.head()
            ))),
        }
    }

    /// The latest verdict on each linked PR, at the head it was given.
    pub(crate) fn verdicts(&self) -> Vec<String> {
        self.tasks
            .values()
            .filter_map(|task| match task {
                Progress::Linked(linked) => Some(linked),
                _ => None,
            })
            .filter_map(|linked| {
                let latest = self.leases.values().rev().find_map(|slot| match slot {
                    LeaseSlot::Spent(spent) => match (spent.lease().purpose(), spent.verdict()) {
                        (Purpose::Review { pr, head }, Some(verdict)) if *pr == linked.pr() => Some((head, verdict)),
                        _ => None,
                    },
                    LeaseSlot::Unused(_) => None,
                });
                latest.map(|(head, verdict)| format!("{} {}: {verdict} at {head}", linked.pr(), linked.id()))
            })
            .collect()
    }
}

impl Reviewing {
    pub fn pr(&self) -> PrNumber {
        self.pr
    }

    /// The review brief and its lease, which ends at `deadline`.
    pub fn brief(self, pull: &Pull, rules: &Rules, deadline: Timestamp) -> Result<Accepted, Refusal> {
        let Self { task, title, agent, lease, pr } = self;
        if pull.state() != PullState::Open {
            return Err(Refusal::because(format!("{pr} is not open; only an open PR can be reviewed.")));
        }
        let head = pull.head().clone();
        let deadline_line = Deadline::of(Some(deadline)).brief_line();
        let agent_name = agent;
        let agent = agent_name.to_string();
        let text = ReviewText {
            pr,
            task,
            title: &title,
            agent: &agent,
            lease,
            deadline_line: &deadline_line,
            head: &head,
            rules,
        }
        .text();
        let deadline = Some(deadline);
        let agent = Some(agent_name);
        Ok(Accepted::new(vec![Event::ReviewLeased { lease, task, agent, pr, head, deadline }], text))
    }
}
