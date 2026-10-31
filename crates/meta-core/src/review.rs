use crate::brief::ReviewText;
use crate::event::Event;
use crate::ids::{AgentName, Binary, LeaseId, PrNumber, Sha, TaskId, Timestamp, Title, Verdict};
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

    /// The PR that review `lease` awaits a verdict on: the one to ask GitHub about.
    pub fn awaiting(&self, lease: LeaseId) -> Result<PrNumber, Refusal> {
        self.awaiting_review(lease).map(|(_, pr, _)| pr)
    }

    /// The task, PR and head of `lease`, which must be a used review lease with no verdict yet. A
    /// headless worker's lease is not one: its verdict comes only from its own session
    /// (`verdict_of`). The deadline plays no part: it tells the agent when to stop, and does not
    /// void an honest verdict reported after it.
    fn awaiting_review(&self, lease: LeaseId) -> Result<(TaskId, PrNumber, Sha), Refusal> {
        let slot = self
            .leases
            .get(&lease)
            .ok_or_else(|| Refusal::because(format!("{lease} is unknown. Next: meta status.")))?;
        let task = slot.lease().task();
        let review = format!("meta review {task} --agent <name> starts a review");
        match (slot, slot.lease().purpose()) {
            (_, Purpose::Work) => Err(Refusal::because(format!(
                "{lease} is a work lease on {task}, not a review lease. \
                 Next: name the lease your review brief gives, or {review}."
            ))),
            (LeaseSlot::Unused(_), Purpose::Review { .. }) => Err(Refusal::because(format!(
                "review lease {lease} is unused: no reviewer started on it. \
                 Next: meta brief {lease} prints its brief again; put {lease} in the reviewer's Agent description."
            ))),
            (LeaseSlot::Spent(spent), Purpose::Review { pr, head }) => match (spent.verdict(), spent.worker()) {
                (Some(verdict), _) => Err(Refusal::because(format!(
                    "{lease} already has the verdict {verdict}. Next: {review} of the current head."
                ))),
                (None, Some(_)) => Err(Refusal::because(format!(
                    "{lease} is a headless worker's lease; its verdict comes only from its session. \
                     Next: meta worker wait {lease} records it, or {review} of your own."
                ))),
                (None, None) => Ok((task, *pr, head.clone())),
            },
        }
    }

    /// Records the verdict on review `lease`, given the head the PR has now: the head the verdict
    /// judged must be the one the lease was issued at.
    pub fn verdict(&self, lease: LeaseId, verdict: Verdict, now: &Pull) -> Result<Accepted, Refusal> {
        let (task, pr, head) = self.awaiting_review(lease)?;
        Self::on_head(task, lease, pr, head, verdict, now)
    }

    /// Records the verdict of the worker on `lease`, the very lease whose head it judged.
    pub(crate) fn verdict_of(&self, lease: LeaseId, verdict: Verdict, now: &Pull) -> Result<Accepted, Refusal> {
        match self.leases.get(&lease) {
            Some(LeaseSlot::Spent(spent)) if spent.verdict().is_none() => match spent.lease().purpose() {
                Purpose::Review { pr, head } => {
                    Self::on_head(spent.lease().task(), lease, *pr, head.clone(), verdict, now)
                }
                Purpose::Work => Err(Refusal::because(format!("{lease} is not a review lease."))),
            },
            _ => Err(Refusal::because(format!("{lease} awaits no verdict."))),
        }
    }

    fn on_head(
        task: TaskId,
        lease: LeaseId,
        pr: PrNumber,
        head: Sha,
        verdict: Verdict,
        now: &Pull,
    ) -> Result<Accepted, Refusal> {
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
    pub fn brief(self, pull: &Pull, binary: &Binary, rules: &Rules, deadline: Timestamp) -> Result<Accepted, Refusal> {
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
            binary,
            rules,
        }
        .text();
        let deadline = Some(deadline);
        let agent = Some(agent_name);
        Ok(Accepted::new(vec![Event::ReviewLeased { lease, task, agent, pr, head, deadline }], text))
    }
}
