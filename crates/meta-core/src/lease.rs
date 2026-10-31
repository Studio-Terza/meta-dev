use crate::event::Event;
use crate::ids::{AgentId, AgentName, LeaseId, PrNumber, Sha, Span, TaskId, Timestamp, Verdict};
use crate::worker::WorkerRun;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Lease {
    id: LeaseId,
    task: TaskId,
    /// Who the lease is for; leases from before briefs could be reprinted have none.
    agent: Option<AgentName>,
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
    /// Set by the stop of the agent; a canned continue clears it, for the agent runs again.
    stopped: bool,
    /// Set when the agent is a headless worker.
    worker: Option<WorkerRun>,
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

    /// The deadline moved later by `span`; none stays none.
    pub(crate) fn delayed(self, span: Span) -> Self {
        Self(self.0.map(|deadline| deadline.plus(span)))
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
    pub(crate) fn issue(
        id: LeaseId,
        task: TaskId,
        agent: Option<AgentName>,
        purpose: Purpose,
        issued: Timestamp,
        deadline: Deadline,
    ) -> Self {
        Self { id, task, agent, purpose, issued, deadline }
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

    pub(crate) fn agent(&self) -> Option<&AgentName> {
        self.agent.as_ref()
    }

    pub(crate) fn purpose(&self) -> &Purpose {
        &self.purpose
    }

    pub(crate) fn spend(self) -> (Spent, Event) {
        let used = Event::LeaseUsed { lease: self.id };
        let deadline = self.deadline;
        (Spent { lease: self, agent: None, deadline, resumes: 0, verdict: None, stopped: false, worker: None }, used)
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
    pub(crate) fn start(
        &mut self,
        agent: AgentId,
        deadline: Deadline,
        worker: Option<WorkerRun>,
    ) -> Result<(), String> {
        match self.agent.replace(agent) {
            None => {
                self.deadline = Deadline(deadline.0.or(self.deadline.0));
                self.worker = worker;
                Ok(())
            }
            Some(earlier) => Err(format!("{} started {earlier} before", self.lease.id)),
        }
    }

    pub(crate) fn worker(&self) -> Option<&WorkerRun> {
        self.worker.as_ref()
    }

    pub(crate) fn resumes(&self) -> usize {
        self.resumes
    }

    pub(crate) fn verdict(&self) -> Option<Verdict> {
        self.verdict
    }

    pub(crate) fn resume(&mut self) {
        self.resumes += 1;
        self.stopped = false;
    }

    /// The stop of the agent this lease started.
    pub(crate) fn stop(&mut self) -> Result<(), String> {
        let id = self.lease.id;
        match (&self.agent, self.stopped) {
            (None, _) => Err(format!("{id} finished but no agent is recorded as started on it")),
            (Some(_), true) => Err(format!("{id} finished twice")),
            (Some(_), false) => {
                self.stopped = true;
                Ok(())
            }
        }
    }

    pub(crate) fn is_stopped(&self) -> bool {
        self.stopped
    }

    /// The one verdict of a used review lease.
    /// A verdict that names its head must name the lease's.
    pub(crate) fn judge(&mut self, verdict: Verdict, judged: Option<&Sha>) -> Result<(), String> {
        let id = self.lease.id;
        match (&self.lease.purpose, self.verdict) {
            (Purpose::Work, _) => Err(format!("{id} is not a review lease")),
            (Purpose::Review { .. }, Some(earlier)) => Err(format!("{id} already has the verdict {earlier}")),
            (Purpose::Review { head, .. }, None) if judged.is_some_and(|judged| judged != head) => {
                Err(format!("{id} was issued at head {head} but its verdict judged another head"))
            }
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

    /// Whether the agent on this lease has finished its part: it stopped, or a review gave its verdict.
    pub(crate) fn is_finished(&self) -> bool {
        matches!(self, Self::Spent(spent) if spent.verdict.is_some() || spent.stopped)
    }

    /// The agent running on this lease: started, and not finished.
    pub(crate) fn running_agent(&self) -> Option<&AgentId> {
        match self {
            Self::Spent(spent) if !self.is_finished() => spent.agent.as_ref(),
            _ => None,
        }
    }

    /// The session of a headless worker whose end is not recorded, verdict or none.
    pub(crate) fn running_worker(&self) -> Option<&WorkerRun> {
        match self {
            Self::Spent(spent) if !spent.stopped => spent.worker.as_ref(),
            _ => None,
        }
    }

    /// The model and transcript of the headless worker on this lease, as a status line ends with them.
    pub(crate) fn worker_note(&self) -> String {
        match self {
            Self::Spent(Spent { worker: Some(run), .. }) => {
                format!(", model {}, transcript {}", run.model, run.transcript)
            }
            _ => String::new(),
        }
    }

    /// Whether a headless worker ever started on this lease.
    pub(crate) fn is_worker(&self) -> bool {
        matches!(self, Self::Spent(Spent { worker: Some(_), .. }))
    }

    /// The agent started on this lease, finished or not.
    pub(crate) fn agent(&self) -> Option<&AgentId> {
        match self {
            Self::Spent(spent) => spent.agent.as_ref(),
            Self::Unused(_) => None,
        }
    }

    /// Moves every deadline of the lease later by `span`: the time its task waited on the owner. A
    /// worker's deadline stays where its start put it: its session does not pause for the wait.
    pub(crate) fn delay(&mut self, span: Span) {
        match self {
            Self::Unused(lease) => lease.deadline = lease.deadline.delayed(span),
            Self::Spent(Spent { worker: Some(_), .. }) => {}
            Self::Spent(spent) => {
                spent.lease.deadline = spent.lease.deadline.delayed(span);
                spent.deadline = spent.deadline.delayed(span);
            }
        }
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
