use crate::ids::{AgentName, PrNumber, Sha, TaskId, Title};
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
    agent: AgentName,
}

#[derive(Debug)]
pub struct Linked {
    agent: AgentName,
    pr: PrNumber,
    pin: Sha,
}

#[derive(Debug)]
pub struct Done {
    agent: AgentName,
    pr: PrNumber,
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
}

impl Task<Open> {
    pub fn new(id: TaskId, title: Title) -> Self {
        Self { id, title, stage: Open }
    }

    pub fn claim(self, agent: AgentName) -> Task<Claimed> {
        self.into_stage(Claimed { agent })
    }
}

impl Task<Claimed> {
    pub fn link(self, pr: PrNumber, pin: Sha) -> Task<Linked> {
        let agent = self.stage.agent.clone();
        self.into_stage(Linked { agent, pr, pin })
    }
}

impl Task<Linked> {
    pub fn relink(self, pr: PrNumber, pin: Sha) -> Task<Linked> {
        let agent = self.stage.agent.clone();
        self.into_stage(Linked { agent, pr, pin })
    }

    pub fn finish(self) -> Task<Done> {
        let Linked { agent, pr, .. } = self.stage;
        Task { id: self.id, title: self.title, stage: Done { agent, pr } }
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
}

impl Progress {
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Done(_))
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
            Self::Done(task) => {
                write!(formatter, "{} done in {} by {}: {}", task.id, task.stage.pr, task.stage.agent, task.title)
            }
        }
    }
}
