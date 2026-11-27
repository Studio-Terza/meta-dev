use std::io;
use std::path::PathBuf;
use std::time::TryFromFloatSecsError;

#[derive(Debug, thiserror::Error)]
pub(crate) enum Failure {
    #[error(
        "usage: ci timed <limits> <stage> [--since <mark>] <program> [<argument>...]\n       ci mutants-plan <workspace> <diff> <changed> <reused> <caught> <output> [<numstat> <outcomes>...]"
    )]
    Usage,
    #[error("cannot read {}: {source}", path.display())]
    Unreadable { path: PathBuf, source: io::Error },
    #[error("cannot write {}: {source}", path.display())]
    Unwritable { path: PathBuf, source: io::Error },
    #[error("cannot read the limits: {0}")]
    Limits(#[from] toml::de::Error),
    #[error("the limits hold no stage named {0}")]
    Unlimited(String),
    #[error("a limit is not a number of seconds: {0}")]
    Seconds(#[from] TryFromFloatSecsError),
    #[error("{} does not hold a start time in seconds since 1970, before now", .0.display())]
    Unmarked(PathBuf),
    #[error("cannot start {program}: {source}")]
    Unstarted { program: String, source: io::Error },
    #[error("cannot read the mutants: {0}")]
    Mutants(#[from] serde_json::Error),
    #[error("a numstat row is not added, deleted and path separated by tabs: {0}")]
    Numstat(String),
    #[error("cannot write the report: {0}")]
    Report(#[from] io::Error),
}
