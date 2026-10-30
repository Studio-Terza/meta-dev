use crate::event::Event;
use crate::ids::{LeaseId, PrNumber, Sha, TaskId, Verdict};

#[derive(Debug, Clone)]
pub struct Lease {
    id: LeaseId,
    task: TaskId,
    purpose: Purpose,
}

/// What the agent started on a lease is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purpose {
    Work,
    Review { pr: PrNumber, head: Sha },
}

/// A lease after its agent started: how often a canned continue resumed it, and a review's verdict.
#[derive(Debug)]
pub struct Spent {
    lease: Lease,
    resumes: usize,
    verdict: Option<Verdict>,
}

impl Lease {
    pub(crate) fn issue(id: LeaseId, task: TaskId, purpose: Purpose) -> Self {
        Self { id, task, purpose }
    }

    pub(crate) fn id(&self) -> LeaseId {
        self.id
    }

    pub(crate) fn task(&self) -> TaskId {
        self.task
    }

    pub(crate) fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    pub(crate) fn spend(self) -> (Spent, Event) {
        let used = Event::LeaseUsed { lease: self.id };
        (Spent { lease: self, resumes: 0, verdict: None }, used)
    }
}

impl Spent {
    pub(crate) fn lease(&self) -> &Lease {
        &self.lease
    }

    pub(crate) fn resumes(&self) -> usize {
        self.resumes
    }

    pub(crate) fn verdict(&self) -> Option<Verdict> {
        self.verdict
    }

    pub(crate) fn resume(&mut self) {
        self.resumes += 1;
    }

    /// The one verdict of a used review lease.
    pub(crate) fn judge(&mut self, verdict: Verdict) -> Result<(), String> {
        let id = self.lease.id;
        match (&self.lease.purpose, self.verdict) {
            (Purpose::Work, _) => Err(format!("{id} is not a review lease")),
            (Purpose::Review { .. }, Some(earlier)) => Err(format!("{id} already has the verdict {earlier}")),
            (Purpose::Review { .. }, None) => {
                self.verdict = Some(verdict);
                Ok(())
            }
        }
    }
}

#[derive(Debug)]
pub enum LeaseSlot {
    Unused(Lease),
    Spent(Spent),
}
