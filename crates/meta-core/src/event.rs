use crate::escape::Checked;
use crate::ids::{
    AgentId, AgentName, Claimant, EscapeId, EscapeKind, LeaseId, Observed, OwnerAnswer, OwnerQuestion, PrNumber,
    PredictionId, Reason, RunCount, Sha, TaskId, Timestamp, Title, Verdict,
};
use crate::lease::LeaseRole;
use crate::predictions::Measure;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "event")]
pub enum Event {
    /// The first record of the lead's real log, written once by `meta mark-real-log`. Only a main
    /// build may write a log that starts with it, wherever the log is reached from.
    RealLog,
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
        /// The agent the lease is for, so `meta brief` prints the same brief. Older leases have none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent: Option<AgentName>,
        /// When the agent must report or stop. Leases from before deadlines have none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deadline: Option<Timestamp>,
        /// The critic's lease is not running work. Leases from before roles are workers.
        #[serde(default, skip_serializing_if = "LeaseRole::is_worker")]
        role: LeaseRole,
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
    /// An escape of a kind that names a merged PR carries it (P1). One recorded while no kinds
    /// could be read is marked unchecked. A build not verified to be from main never writes one:
    /// its escape goes to the side file (`UnverifiedEscape`).
    Escaped {
        #[serde(default = "EscapeKind::unclassified")]
        kind: EscapeKind,
        reason: Reason,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        pr: Option<PrNumber>,
        #[serde(default, skip_serializing_if = "Checked::is_listed")]
        checked: Checked,
    },
    Resolved {
        escape: EscapeId,
        task: TaskId,
    },
    /// A one-off bypass that cannot recur and has no fixing task, closed with a reason.
    ResolvedHistorical {
        escape: EscapeId,
        reason: Reason,
    },
    /// Moves an escape resolved against a task to historical. The `Resolved` record stays in the log.
    Corrected {
        escape: EscapeId,
        reason: Reason,
    },
    LeaseUsed {
        lease: LeaseId,
    },
    /// The agent the used lease started, by the id Claude Code gave it, and the lease's deadline
    /// counted from this start.
    Started {
        lease: LeaseId,
        agent: AgentId,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deadline: Option<Timestamp>,
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
        #[serde(default, skip_serializing_if = "Option::is_none")]
        agent: Option<AgentName>,
        pr: PrNumber,
        head: Sha,
        /// When the agent must report or stop. Leases from before deadlines have none.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        deadline: Option<Timestamp>,
    },
    Reviewed {
        lease: LeaseId,
        verdict: Verdict,
    },
    /// The first time a prediction's falsifier is met by one of its measures, kept for good: the
    /// record's own time is when. Resolving escapes later cannot erase it.
    Falsified {
        prediction: PredictionId,
        measure: Measure,
        observed: Observed,
    },
    /// The task needs a decision only the owner can give. Its lease deadlines pause until the answer.
    WaitingOnOwner {
        task: TaskId,
        question: OwnerQuestion,
    },
    /// The owner's answer. The deadlines of the task's leases move later by the time it waited.
    OwnerAnswered {
        task: TaskId,
        answer: OwnerAnswer,
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
