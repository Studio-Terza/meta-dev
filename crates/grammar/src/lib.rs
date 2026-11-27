#![deny(clippy::wildcard_enum_match_arm)]

mod bare;
mod detail;
mod git;
mod letters;
mod lines;
mod lstart;
mod prose;
mod queue;
mod reply;
mod report;
mod repository;
mod spec_line;
mod unlexed;
mod unspelled;

pub use bare::Bare;
pub use detail::{Cited, Heading};
pub use git::{MergeCommand, Worktree, Worktrees};
pub use invariants::Ignoring;
pub use letters::Letters;
pub use lines::{Fields, KeptLines, Lines};
pub use lstart::Stamp;
pub use prose::Prose;
pub use queue::Queue;
pub use reply::Inked;
pub use report::{AgentReport, BranchName, Ended, JobLink, Target};
pub use repository::{RepositoryName, SpecTrailer};
pub use spec_line::{Attachment, Line};
pub use unlexed::Unlexed;
pub(crate) use unspelled::Unspelled;
