use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Failure {
    #[error(
        "usage: ci timed <measured file> <stage> <program> [<argument>...] | ci libtest-seconds <measured file> <stage> <test output> | ci reach <metadata file> <event>"
    )]
    Usage,
    #[error("cannot use {path}: {source}", path = path.display())]
    Io { path: PathBuf, source: io::Error },
    #[error("cannot run {program}: {source}", program = program.to_string_lossy())]
    Program { program: std::ffi::OsString, source: io::Error },
    #[error("cannot write the report: {0}")]
    Write(#[from] io::Error),
    #[error("reach failed: {0}")]
    Reach(String),
}
