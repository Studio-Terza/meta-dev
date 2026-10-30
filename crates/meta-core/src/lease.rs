use crate::event::Event;
use crate::ids::{AgentId, LeaseId, PrNumber, Sha, TaskId, Timestamp, Verdict};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Lease {
    id: LeaseId,
    task: TaskId,
    purpose: Purpose,
    issued: Timestamp,
    deadline: Deadline,
}

/// When the agent on a lease must report or stop; leases from before deadlines have none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Deadline(Option<Timestamp>);

/// What the agent started on a lease is for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Purpose {
    Work,
    Review { pr: PrNumber, head: Sha },
}

/// A lease after its agent started: the agent's id once recorded, the deadline counted from that
/// start, how often a canned continue resumed it, and a review's verdict.
#[derive(Debug)]
pub struct Spent {
    lease: Lease,
    agent: Option<AgentId>,
    deadline: Deadline,
    resumes: usize,
    verdict: Option<Verdict>,
}

impl Deadline {
    pub(crate) fn of(deadline: Option<Timestamp>) -> Self {
        Self(deadline)
    }

    pub(crate) fn time(self) -> Option<Timestamp> {
        self.0
    }

    /// The deadline as a brief states it.
    pub(crate) fn brief_line(self) -> String {
        match self.0 {
            Some(deadline) => format!("{deadline}, report or stop by then"),
            None => "none".to_owned(),
        }
    }

    /// Whether the deadline has passed at `now`; a lease with none never passes it.
    pub(crate) fn passed(self, now: Timestamp) -> bool {
        self.0.is_some_and(|deadline| deadline < now)
    }
}

impl fmt::Display for Deadline {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.0 {
            Some(deadline) => write!(formatter, "deadline {deadline}"),
            None => formatter.write_str("no deadline"),
        }
    }
}

impl Lease {
    pub(crate) fn issue(id: LeaseId, task: TaskId, purpose: Purpose, issued: Timestamp, deadline: Deadline) -> Self {
        Self { id, task, purpose, issued, deadline }
    }

    /// The same allowance as this lease's, counted from `start` instead of from its issue.
    pub(crate) fn deadline_from(&self, start: Timestamp) -> Deadline {
        Deadline(self.deadline.0.map(|deadline| start.plus(self.issued.span_to(deadline))))
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
        let deadline = self.deadline;
        (Spent { lease: self, agent: None, deadline, resumes: 0, verdict: None }, used)
    }
}

impl Spent {
    pub(crate) fn lease(&self) -> &Lease {
        &self.lease
    }

    pub(crate) fn agent(&self) -> Option<&AgentId> {
        self.agent.as_ref()
    }

    /// The one agent a used lease started; a deadline recorded with the start replaces the lease's.
    pub(crate) fn start(&mut self, agent: AgentId, deadline: Deadline) -> Result<(), String> {
        match self.agent.replace(agent) {
            None => {
                self.deadline = Deadline(deadline.0.or(self.deadline.0));
                Ok(())
            }
            Some(earlier) => Err(format!("{} started {earlier} before", self.lease.id)),
        }
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

impl LeaseSlot {
    pub(crate) fn lease(&self) -> &Lease {
        match self {
            Self::Unused(lease) => lease,
            Self::Spent(spent) => &spent.lease,
        }
    }

    /// The deadline in force: from the start once an agent started, else from the lease.
    pub(crate) fn deadline(&self) -> Deadline {
        match self {
            Self::Unused(lease) => lease.deadline,
            Self::Spent(spent) => spent.deadline,
        }
    }

    /// Whether the agent on this lease has finished its part: a review with its verdict.
    pub(crate) fn is_finished(&self) -> bool {
        matches!(self, Self::Spent(spent) if spent.verdict.is_some())
    }

    /// Who holds the lease, for a flag: the agent id once recorded.
    pub(crate) fn holder(&self) -> String {
        match self {
            Self::Unused(_) => "unused".to_owned(),
            Self::Spent(Spent { agent: Some(agent), .. }) => format!("agent {agent}"),
            Self::Spent(Spent { agent: None, .. }) => "agent unknown".to_owned(),
        }
    }
}
