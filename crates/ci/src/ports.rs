use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;
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
            .map_err(|source| Failure::Unstarted { program: program.to_string_lossy().into_owned(), source })?;
        Ok(Ran { status, spent: start.elapsed() })
    }

    pub(crate) fn status(&self) -> ExitStatus {
        self.status
    }

    pub(crate) fn spent(&self) -> Duration {
        self.spent
    }
}

impl Disk {
    pub(crate) fn read(path: &Path) -> Result<String, Failure> {
        fs::read_to_string(path).map_err(|source| Failure::Unreadable { path: path.to_path_buf(), source })
    }

    pub(crate) fn write(path: &Path, text: &str) -> Result<(), Failure> {
        fs::write(path, text).map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }

    pub(crate) fn append(path: &Path, text: &str) -> Result<(), Failure> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .and_then(|mut file| file.write_all(text.as_bytes()))
            .map_err(|source| Failure::Unwritable { path: path.to_path_buf(), source })
    }
}
