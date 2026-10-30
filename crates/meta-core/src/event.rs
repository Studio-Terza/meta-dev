use crate::ids::{
    Claimant, EscapeId, EscapeKind, LeaseId, PrNumber, Reason, RunCount, Sha, TaskId, Timestamp, Title, Verdict,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    Opened {
        task: TaskId,
        title: Title,
    },
    /// An agent's claim names it; the lead's self-claim has no agent and gets no lease.
    Claimed {
        task: TaskId,
        #[serde(default, skip_serializing_if = "Claimant::is_lead")]
        agent: Claimant,
    },
    Leased {
        lease: LeaseId,
        task: TaskId,
    },
    Linked {
        task: TaskId,
        pr: PrNumber,
        pin: Sha,
    },
    Merged {
        pr: PrNumber,
        head: Sha,
        runs: RunCount,
        route: Route,
    },
    Done {
        task: TaskId,
    },
    /// The nth `Escaped` record is `escape-n`. Records from before kinds existed fold as unclassified.
    Escaped {
        #[serde(default = "EscapeKind::unclassified")]
        kind: EscapeKind,
        reason: Reason,
    },
    Resolved {
        escape: EscapeId,
        task: TaskId,
    },
    LeaseUsed {
        lease: LeaseId,
    },
    Denied {
        attempt: Attempt,
    },
    Retired {
        task: TaskId,
        reason: Reason,
    },
    Deferred {
        task: TaskId,
        reason: Reason,
    },
    Undeferred {
        task: TaskId,
    },
    /// A lease for one reviewer of the PR linked to the task, at the head it was given.
    ReviewLeased {
        lease: LeaseId,
        task: TaskId,
        pr: PrNumber,
        head: Sha,
    },
    Reviewed {
        lease: LeaseId,
        verdict: Verdict,
    },
    /// A canned `continue` restarted the agent of a used lease.
    Resumed {
        lease: LeaseId,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Attempt {
    AgentStart,
    /// Written by older hooks that denied Bash merges. Logs keep them; the fold ignores them.
    Merge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Route {
    Meta,
    Outside,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub at: Timestamp,
    #[serde(flatten)]
    pub event: Event,
}
