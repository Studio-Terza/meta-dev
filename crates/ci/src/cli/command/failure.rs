mod refusal;

use std::borrow::Cow;
use std::io;
use std::path::Path;

use file_ports::FileFailure;

pub use refusal::Refusal;

use crate::cli::ports::Invocation;

#[derive(Debug, thiserror::Error)]
pub enum Failure<'path> {
    #[error(
        "usage: ci timed <measured file> <stage> <program> [<argument>...] | ci logged <log file> <program> [<argument>...] | ci libtest-seconds <measured file> <stage> <test output> | ci reach <metadata file> <event> <base branch> <environment file> <summary file> | ci history <metadata file> <history file> <revision> | ci measured-history <measured file> <history file> <revision> | ci ratchet-lower <record file> <measured file> | ci tools <built directory> <tools directory> | ci tools-key <metadata file> <output file> | ci mutants-shard <shard> <output directory> | ci mutants-changed <diff file> <output directory> [<scope>...] | ci wall-times <repository> <run> <job> | ci compile-times <timings page> <report file> | ci graph-path <timings page> <measured file> <report file> | ci show <file>... | ci summarize <report file> <summary file> | ci stage-issues <limits file> <measured file> | ci compile-trend <cold build directory> <downloads directory> | ci labels <base branch> <pull request> <metadata file> | ci spec-line <pull request> | ci shard-summary <shards directory> <summary file> | ci matrix <shards directory> <sources directory> <output directory> <summary file> | ci cold-size <directory> <output file>"
    )]
    Usage,
    #[error("cannot use {path}: {source}", path = path.display())]
    Io { path: Cow<'path, Path>, source: io::Error },
    #[error("no libtest result line in {path}; the test output format may have changed", path = path.display())]
    NoTestResult { path: &'path Path },
    #[error("no unit timings in {path}; the cargo timings format may have changed", path = path.display())]
    NoTimings { path: Cow<'path, Path> },
    #[error("cannot run {program}: {source}", program = program.to_string_lossy())]
    Program { program: std::ffi::OsString, source: io::Error },
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("reach failed: {0}")]
    Reach(Refusal),
    #[error("the limits cannot be judged: {0}")]
    Limits(gate_limits::Failure<'path>),
    #[error("cannot read the list of issues: {0}")]
    Issues(serde_json::Error),
    #[error("cannot read the stage times in {path}: {source}", path = path.display())]
    Measured { path: Cow<'path, Path>, source: toml::de::Error },
    #[error("cannot read the cargo metadata: {0}")]
    Metadata(serde_json::Error),
    #[error("cannot read the list of cold builds: {0}")]
    Runs(serde_json::Error),
    #[error("the mutation matrix failed: {0}")]
    Matrix(Refusal),
}

impl<'path> Failure<'path> {
    pub(crate) fn unusable(path: impl Into<Cow<'path, Path>>, failure: FileFailure) -> Failure<'path> {
        Failure::Io { path: path.into(), source: io::Error::from(failure) }
    }

    pub(crate) fn starting(invocation: &Invocation) -> impl FnOnce(io::Error) -> Failure<'path> + '_ {
        move |source| Failure::Program { program: invocation.program().to_owned(), source }
    }

    pub(crate) fn started<Value>(
        results: Vec<io::Result<Value>>,
        invocations: &[Invocation],
    ) -> Result<Vec<Value>, Failure<'path>> {
        results
            .into_iter()
            .zip(invocations)
            .map(|(result, invocation)| result.map_err(Failure::starting(invocation)))
            .collect()
    }
}

impl<'path> From<gate_limits::Failure<'path>> for Failure<'path> {
    fn from(failure: gate_limits::Failure<'path>) -> Failure<'path> {
        Failure::Limits(failure)
    }
}
