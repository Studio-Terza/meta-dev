use crate::ids::{Claimant, PrNumber, Reason, Sha, TaskId, Title};
use std::fmt;

#[derive(Debug)]
pub struct Task<Stage> {
    id: TaskId,
    title: Title,
    stage: Stage,
}

#[derive(Debug)]
pub struct Open;

#[derive(Debug)]
pub struct Claimed {
    agent: Claimant,
}

#[derive(Debug)]
pub struct Linked {
    agent: Claimant,
    pr: PrNumber,
    pin: Sha,
}

#[derive(Debug)]
pub struct Done {
    how: Finish,
}

/// How a task came to be done.
#[derive(Debug)]
enum Finish {
    /// Merged through its own linked PR.
    Own { agent: Claimant, pr: PrNumber },
    /// Finished by another task's merged PR.
    By(PrNumber),
}

/// Off the open list for good, without a merge.
#[derive(Debug)]
pub struct Retired {
    reason: Reason,
}

/// Off the open list until undeferred.
#[derive(Debug)]
pub struct Deferred {
    reason: Reason,
}

impl<Stage> Task<Stage> {
    pub fn id(&self) -> TaskId {
        self.id
    }

    pub fn title(&self) -> &Title {
        &self.title
    }

    fn into_stage<Next>(self, stage: Next) -> Task<Next> {
        Task { id: self.id, title: self.title, stage }
    }

    pub fn retire(self, reason: Reason) -> Task<Retired> {
        self.into_stage(Retired { reason })
    }
}

/// A stage a task can leave by another task's merged PR.
pub trait Finishable {}

impl Finishable for Open {}
impl Finishable for Claimed {}
impl Finishable for Linked {}

impl<Stage: Finishable> Task<Stage> {
    pub fn finish_by(self, pr: PrNumber) -> Task<Done> {
        self.into_stage(Done { how: Finish::By(pr) })
    }
}

impl Task<Open> {
    pub fn new(id: TaskId, title: Title) -> Self {
        Self { id, title, stage: Open }
    }

    pub fn claim(self, agent: Claimant) -> Task<Claimed> {
        self.into_stage(Claimed { agent })
    }

    pub fn defer(self, reason: Reason) -> Task<Deferred> {
        self.into_stage(Deferred { reason })
    }
}

impl Task<Claimed> {
    pub fn agent(&self) -> &Claimant {
        &self.stage.agent
    }
}

impl Task<Deferred> {
    pub fn undefer(self) -> Task<Open> {
        self.into_stage(Open)
    }
}

impl Task<Claimed> {
    pub fn link(self, pr: PrNumber, pin: Sha) -> Task<Linked> {
        let agent = self.stage.agent.clone();
        self.into_stage(Linked { agent, pr, pin })
    }
}

impl Task<Linked> {
    pub fn agent(&self) -> &Claimant {
        &self.stage.agent
    }

    pub fn relink(self, pr: PrNumber, pin: Sha) -> Task<Linked> {
        let agent = self.stage.agent.clone();
        self.into_stage(Linked { agent, pr, pin })
    }

    pub fn finish(self) -> Task<Done> {
        let Linked { agent, pr, .. } = self.stage;
        Task { id: self.id, title: self.title, stage: Done { how: Finish::Own { agent, pr } } }
    }

    pub fn pr(&self) -> PrNumber {
        self.stage.pr
    }

    pub fn pin(&self) -> &Sha {
        &self.stage.pin
    }
}

#[derive(Debug)]
pub enum Progress {
    Open(Task<Open>),
    Claimed(Task<Claimed>),
    Linked(Task<Linked>),
    Done(Task<Done>),
    Retired(Task<Retired>),
    Deferred(Task<Deferred>),
}

impl Progress {
    /// Open, claimed or linked: on the open list, and its unused leases are live.
    pub fn is_active(&self) -> bool {
        matches!(self, Self::Open(_) | Self::Claimed(_) | Self::Linked(_))
    }

    /// The stage's name, as the rework report shows it.
    pub fn stage(&self) -> &'static str {
        match self {
            Self::Open(_) => "open",
            Self::Claimed(_) => "claimed",
            Self::Linked(_) => "linked",
            Self::Done(_) => "done",
            Self::Retired(_) => "retired",
            Self::Deferred(_) => "deferred",
        }
    }

    /// The PR that finished the task when it was not its own: set by `meta done --by`.
    pub fn done_by(&self) -> Option<PrNumber> {
        match self {
            Self::Done(task) => match task.stage.how {
                Finish::By(pr) => Some(pr),
                Finish::Own { .. } => None,
            },
            _ => None,
        }
    }

    /// Claimed or linked by the lead working it itself.
    pub fn is_self_claimed(&self) -> bool {
        match self {
            Self::Claimed(task) => task.agent().is_lead(),
            Self::Linked(task) => task.agent().is_lead(),
            _ => false,
        }
    }
}

impl fmt::Display for Progress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Open(task) => write!(formatter, "{} open: {}", task.id, task.title),
            Self::Claimed(task) => write!(formatter, "{} claimed by {}: {}", task.id, task.stage.agent, task.title),
            Self::Linked(task) => {
                write!(formatter, "{} linked to {} by {}: {}", task.id, task.stage.pr, task.stage.agent, task.title)
            }
            Self::Done(task) => match &task.stage.how {
                Finish::Own { agent, pr } => write!(formatter, "{} done in {pr} by {agent}: {}", task.id, task.title),
                Finish::By(pr) => write!(formatter, "{} done by {pr}: {}", task.id, task.title),
            },
            Self::Retired(task) => write!(formatter, "{} retired ({}): {}", task.id, task.stage.reason, task.title),
            Self::Deferred(task) => write!(formatter, "{} deferred ({}): {}", task.id, task.stage.reason, task.title),
        }
    }
}
