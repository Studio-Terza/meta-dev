#![deny(clippy::wildcard_enum_match_arm)]

mod bare;
mod detail;
mod letters;
mod lines;
mod notice;
mod prose;
mod report;
mod repository;
mod shell;
mod unlexed;
mod unspelled;

pub use bare::Bare;
pub use detail::{Cited, Heading};
pub use invariants::Ignoring;
pub use letters::Letters;
pub use lines::{Fields, KeptLines, Lines};
pub use logos;
pub use notice::{TaskFile, TaskNotice};
pub use prose::Prose;
pub use report::{AgentReport, BranchName, Ended, JobLink, Target};
pub use repository::{RepositoryName, SpecTrailer};
pub use shell::{Construct, Directory, Edit, Edited, Editor, Flagged, Nested};
pub use unlexed::Unlexed;
pub use unspelled::Unspelled;
