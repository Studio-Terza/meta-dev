#![deny(clippy::wildcard_enum_match_arm)]

mod boundary;
mod letters;
mod unlexed;
mod unspelled;

pub use boundary::{
    AgentReport, Bare, BranchName, Cited, Ended, Fields, Heading, Inked, JobLink, KeptLines, Lines, MigrationFile,
    Prose, Queue, RepositoryName, SpecTrailer, Stamp, Target,
};
pub use invariants::Ignoring;
pub use letters::Letters;
pub use logos;
pub use unlexed::Unlexed;
pub use unspelled::Unspelled;
