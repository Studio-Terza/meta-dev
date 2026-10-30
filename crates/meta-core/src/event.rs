use crate::ids::{AgentName, LeaseId, PrNumber, Reason, RunCount, Sha, TaskId, Timestamp, Title};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    Opened { task: TaskId, title: Title },
    Claimed { task: TaskId, agent: AgentName },
    Leased { lease: LeaseId, task: TaskId },
    Linked { task: TaskId, pr: PrNumber, pin: Sha },
    Merged { pr: PrNumber, head: Sha, runs: RunCount },
    Done { task: TaskId },
    Escaped { reason: Reason },
    LeaseUsed { lease: LeaseId },
    Denied { attempt: Attempt },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attempt {
    AgentStart,
    Merge,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub at: Timestamp,
    #[serde(flatten)]
    pub event: Event,
}
