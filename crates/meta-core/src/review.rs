use crate::event::Event;
use crate::ids::{AgentName, LeaseId, PrNumber, TaskId, Timestamp, Title, Verdict};
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
                "review lease {unused} on {task} is unused. Next: put {unused} in the reviewer's Agent description."
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

    pub fn verdict(&self, task: TaskId, verdict: Verdict) -> Result<Accepted, Refusal> {
        let awaiting = self.leases.iter().find_map(|(id, slot)| match slot {
            LeaseSlot::Spent(spent)
                if spent.lease().task() == task
                    && matches!(spent.lease().purpose(), Purpose::Review { .. })
                    && spent.verdict().is_none() =>
            {
                Some(*id)
            }
            _ => None,
        });
        match awaiting {
            Some(lease) => {
                let text = format!("{task}: verdict {verdict} recorded for review lease {lease}\n");
                Ok(Accepted::new(vec![Event::Reviewed { lease, verdict }], text))
            }
            None => Err(Refusal::because(format!(
                "no used review lease on {task} awaits a verdict. Next: meta review {task} --agent <name>, \
                 start the reviewer with that lease, then record its verdict."
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
        let digits = pr.digits();
        let deadline_line = Deadline::of(Some(deadline)).brief_line();
        let text = format!(
            "review: {pr} for {task}: {title}\n\
             agent: {agent}\n\
             lease: {lease} (put it in the Agent description; it works once)\n\
             deadline: {deadline_line}\n\
             goal: find what is wrong with {pr} at head {head}, judged against {task} and the rules below.\n\
             constraints:\n  \
             - read the change with gh pr view {digits} and gh pr diff {digits}; push, comment, approve and merge nothing\n  \
             - report each finding with its file, line and evidence, then a verdict: pass or fail\n\
             rules:\n{rules}"
        );
        let deadline = Some(deadline);
        Ok(Accepted::new(vec![Event::ReviewLeased { lease, task, pr, head, deadline }], text))
    }
}
