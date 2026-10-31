use crate::ids::{LeaseId, Timestamp, Verdict};
use crate::lease::{LeaseSlot, Purpose};
use crate::ledger::Ledger;
use crate::task::{Linked, Progress, Task};

impl Ledger {
    /// The idle alarm: no running work at `at`, while a task has no next step. Names each stalled task.
    /// The critic's task is long-running by design, so it is never stalled.
    pub(crate) fn idle(&self, at: Timestamp) -> Vec<String> {
        let stalled: Vec<String> = self
            .tasks
            .iter()
            .filter(|(task, _)| !self.waiting.contains_key(task) && !self.critics.contains(task))
            .filter_map(|(task, progress)| self.stall(progress, at).map(|why| format!("{task} ({why})")))
            .collect();
        match (self.leases.iter().any(|(id, slot)| self.runs(*id, slot, at)), stalled.is_empty()) {
            (false, false) => vec![format!("idle: no live work known; stalled: {}", stalled.join(", "))],
            _ => Vec::new(),
        }
    }

    /// Why a task has no next step, if it has none. A task waiting on the owner is left out by the caller.
    fn stall(&self, progress: &Progress, at: Timestamp) -> Option<&'static str> {
        match progress {
            Progress::Open(_) => Some("open"),
            Progress::Claimed(_) => Some("claimed"),
            Progress::Linked(linked) => self.linked_stall(linked, at),
            Progress::Done(_) | Progress::Retired(_) | Progress::Deferred(_) => None,
        }
    }

    /// A linked task is stalled when no review of it is live and: its pinned head has no verdict,
    /// or passed without a merge, or failed with no work lease issued since.
    fn linked_stall(&self, linked: &Task<Linked>, at: Timestamp) -> Option<&'static str> {
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
                let fixing = self.leases.iter().any(|(id, slot)| *id > judged && self.runs(*id, slot, at));
                (!fixing).then_some("fail on head, no live work lease")
            }
        }
    }

    /// The latest verdict, in log order, on the linked task's pinned head, and the review lease that gave it.
    fn judged_head(&self, linked: &Task<Linked>) -> Option<(LeaseId, Verdict)> {
        self.latest_verdict(linked.pr(), |head| head == linked.pin()).map(|(lease, _, verdict)| (lease, verdict))
    }

    /// Running work at `at`: a worker's work lease with a deadline not yet passed, on a task an agent is working (open or claimed),
    /// or on a linked task whose pinned head failed review before the lease was issued. A task that
    /// waits on the owner is paused, so its lease is not running. A lease past its deadline is not
    /// running: the agent was to report or stop by then. A finished lease is not counted either: past a
    /// turn end its agent may still wait on background work, which is why the alarm says "known".
    /// A lease on a critic task never counts.
    fn runs(&self, id: LeaseId, slot: &LeaseSlot, at: Timestamp) -> bool {
        let lease = slot.lease();
        let working = *lease.purpose() == Purpose::Work
            && !self.critics.contains(&lease.task())
            && slot.deadline().time().is_some()
            && !slot.deadline().passed(at)
            && !slot.is_finished()
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
