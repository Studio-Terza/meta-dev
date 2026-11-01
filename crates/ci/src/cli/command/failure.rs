mod refusal;

use std::io;
use std::path::PathBuf;

pub use refusal::Refusal;

use crate::cli::ports::Invocation;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: ci timed <measured file> <stage> <program> [<argument>...] | ci logged <log file> <program> [<argument>...] | ci libtest-seconds <measured file> <stage> <test output> | ci reach <metadata file> <event> <base branch> <environment file> <summary file> | ci history <metadata file> <history file> <revision> | ci tools <built directory> <tools directory> | ci tools-key <metadata file> <output file> | ci mutants-shard <shard> <output directory> | ci mutants-changed <diff file> <output directory> [<scope>...] | ci wall-times <repository> <run> <job> | ci compile-times <timings page> <report file> | ci show <file>... | ci summarize <report file> <summary file> | ci stage-issues <limits file> <measured file> | ci labels <base branch> <pull request> | ci shard-summary <shards directory> <summary file> | ci matrix <shards directory> <sources directory> <output directory> <summary file>"
    )]
    Usage,
    #[error("cannot use {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("no libtest result line in {path}; the test output format may have changed", path = path.display())]
    NoTestResult { path: PathBuf },
    #[error("no unit timings in {path}; the cargo timings format may have changed", path = path.display())]
    NoTimings { path: PathBuf },
    #[error("cannot run {program}: {source}", program = program.to_string_lossy())]
    Program { program: std::ffi::OsString, source: io::Error },
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("reach failed: {0}")]
    Reach(Refusal),
    #[error("the limits cannot be judged: {0}")]
    Limits(#[from] gate_limits::Failure),
    #[error("cannot read the list of issues: {0}")]
    Issues(serde_json::Error),
    #[error("the mutation matrix failed: {0}")]
    Matrix(Refusal),
}

impl Failure {
    pub(crate) fn starting(invocation: &Invocation) -> impl FnOnce(io::Error) -> Failure + '_ {
        move |source| Failure::Program { program: invocation.program().to_owned(), source }
    }
}
