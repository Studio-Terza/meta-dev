use crate::ids::{LeaseId, Timestamp, Verdict};
use crate::lease::{LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::task::{Linked, Progress, Task};

impl Ledger {
    /// The idle alarm: no running work, while a task has no next step. Names each stalled task.
    pub(crate) fn idle(&self) -> Vec<String> {
        let stalled: Vec<String> = self
            .tasks
            .iter()
            .filter(|(task, _)| !self.waiting.contains_key(task))
            .filter_map(|(task, progress)| self.stall(progress).map(|why| format!("{task} ({why})")))
            .collect();
        match (self.leases.iter().any(|(id, slot)| self.runs(*id, slot)), stalled.is_empty()) {
            (false, false) => vec![format!("idle: no live work; stalled: {}", stalled.join(", "))],
            _ => Vec::new(),
        }
    }

    /// Why a task has no next step, if it has none. A task waiting on the owner is left out by the caller.
    fn stall(&self, progress: &Progress) -> Option<&'static str> {
        match progress {
            Progress::Open(_) => Some("open"),
            Progress::Claimed(_) => Some("claimed"),
            Progress::Linked(linked) => self.linked_stall(linked),
            Progress::Done(_) | Progress::Retired(_) | Progress::Deferred(_) => None,
        }
    }

    /// A linked task is stalled when no review of it is live and: its pinned head has no verdict,
    /// or passed without a merge, or failed with no work lease issued since.
    fn linked_stall(&self, linked: &Task<Linked>) -> Option<&'static str> {
        let reviewing = self.leases.values().any(|slot| {
            matches!(slot.lease().purpose(), Purpose::Review { pr, .. } if *pr == linked.pr())
                && slot.lease().task() == linked.id()
                && !slot.is_finished()
        });
        match (reviewing, self.judged_head(linked)) {
            (true, _) => None,
            (false, None) => Some("no verdict on head"),
            (false, Some((_, Verdict::Pass))) => {
                (!self.landed.contains_key(&linked.pr())).then_some("pass on head, PR not merged")
            }
            (false, Some((judged, Verdict::Fail))) => {
                let fixing = self.leases.iter().any(|(id, slot)| *id > judged && self.runs(*id, slot));
                (!fixing).then_some("fail on head, no live work lease")
            }
        }
    }

    /// The latest verdict on the linked task's pinned head, and the review lease that gave it.
    fn judged_head(&self, linked: &Task<Linked>) -> Option<(LeaseId, Verdict)> {
        self.leases.iter().rev().find_map(|(id, slot)| match slot {
            LeaseSlot::Spent(spent) => match (spent.lease().purpose(), spent.verdict()) {
                (Purpose::Review { pr, head }, Some(verdict)) if *pr == linked.pr() && head == linked.pin() => {
                    Some((*id, verdict))
                }
                _ => None,
            },
            LeaseSlot::Unused(_) => None,
        })
    }

    /// Running work: a work lease with a deadline, on a task an agent is working (open or claimed),
    /// or on a linked task whose pinned head failed review before the lease was issued. A task that
    /// waits on the owner is paused, so its lease is not running.
    fn runs(&self, id: LeaseId, slot: &LeaseSlot) -> bool {
        let lease = slot.lease();
        let working = *lease.purpose() == Purpose::Work
            && slot.deadline().time().is_some()
            && !self.waiting.contains_key(&lease.task());
        working
            && match self.tasks.get(&lease.task()) {
                Some(Progress::Open(_) | Progress::Claimed(_)) => true,
                Some(Progress::Linked(linked)) => {
                    self.judged_head(linked).is_some_and(|(judged, verdict)| verdict == Verdict::Fail && id > judged)
                }
                _ => false,
            }
    }

    /// How long each waiting task has waited, for the flagged section.
    pub(crate) fn waits(&self, at: Timestamp) -> Vec<String> {
        self.waiting
            .iter()
            .map(|(task, waiting)| {
                let minutes = waiting.started().span_to(at).whole_minutes();
                format!("{task}: {} (waiting {minutes} min)", waiting.question())
            })
            .collect()
    }
}
