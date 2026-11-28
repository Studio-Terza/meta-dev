mod boundary;

use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use grammar_lines::Lines;

use crate::failure::Failure;

#[derive(Debug)]
pub(crate) struct Ran {
    status: ExitStatus,
    spent: Duration,
}

#[derive(Debug)]
pub(crate) struct Disk;

#[derive(Debug)]
pub(crate) struct Mark;

impl Mark {
    pub(crate) fn elapsed(path: &Path) -> Result<Duration, Failure> {
        let text = Disk::read(path)?;
        let first = text.parse::<Lines>().into_iter().flatten().next().unwrap_or_default();
        let seconds: f64 = first.parse().map_err(|_unparsed| Failure::Unmarked(path.to_path_buf()))?;
        let start = UNIX_EPOCH + Duration::try_from_secs_f64(seconds)?;
        SystemTime::now().duration_since(start).map_err(|_later| Failure::Unmarked(path.to_path_buf()))
    }
}

impl Ran {
    pub(crate) fn started(program: &OsStr, arguments: &[OsString]) -> Result<Ran, Failure> {
        let start = Instant::now();
        let status = Command::new(program)
            .args(arguments)
            .status()
            .map_err(|source| Failure::Unstarted { program: PathBuf::from(program), source })?;
        Ok(Ran { status, spent: start.elapsed() })
    }

    pub(crate) fn status(&self) -> ExitStatus {
        self.status
    }

    pub(crate) fn spent(&self) -> Duration {
        self.spent
    }
}
