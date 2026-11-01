use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::command::failure::Failure;

#[derive(Clone, Debug)]
pub struct Journal(PathBuf);

impl Journal {
    pub(crate) fn file(path: &Path) -> Journal {
        Journal(path.to_path_buf())
    }

    pub(crate) fn append(&self, entry: impl fmt::Display) -> Result<(), Failure> {
        self.prepared()
            .and_then(|()| OpenOptions::new().append(true).create(true).open(&self.0))
            .and_then(|mut file| write!(file, "{entry}"))
            .map_err(|source| self.failed(source))
    }

    pub(crate) fn replace(&self, text: &str) -> Result<(), Failure> {
        self.prepared().and_then(|()| fs::write(&self.0, text)).map_err(|source| self.failed(source))
    }

    pub(crate) fn read(&self) -> Result<String, Failure> {
        fs::read_to_string(&self.0).map_err(|source| self.failed(source))
    }

    pub(crate) fn read_or(&self, absent: &str) -> Result<String, Failure> {
        if self.exists() { self.read() } else { Ok(absent.to_owned()) }
    }

    pub(crate) fn exists(&self) -> bool {
        self.0.is_file()
    }

    fn prepared(&self) -> io::Result<()> {
        self.0.parent().map_or(Ok(()), fs::create_dir_all)
    }

    fn failed(&self, source: io::Error) -> Failure {
        Failure::Io { path: self.0.clone(), source }
    }
}

impl fmt::Display for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}

impl AsRef<Path> for Journal {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
