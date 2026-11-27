#![deny(clippy::wildcard_enum_match_arm)]

mod detail;
mod git;
mod prose;
mod queue;
mod report;
mod repository;
mod unlexed;

pub use detail::{Cited, Heading};
pub use git::{MergeCommand, Worktree, Worktrees};
pub use invariants::Ignoring;
pub use logos;
pub use prose::Prose;
pub use queue::Queue;
pub use report::{AgentReport, BranchName, Ended, JobLink, Target};
pub use repository::{RepositoryName, SpecTrailer};
pub use unlexed::Unlexed;
