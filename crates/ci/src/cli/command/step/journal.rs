use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

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

    pub(crate) fn emptied(&self) -> Result<Journal, Failure> {
        self.replace(&Text::default()).map(|()| self.clone())
    }

    pub(crate) fn replace(&self, text: &Text) -> Result<(), Failure> {
        self.prepared().and_then(|()| fs::write(&self.0, text.as_ref())).map_err(|source| self.failed(source))
    }

    pub(crate) fn read(&self) -> Result<Text, Failure> {
        fs::read_to_string(&self.0).map(Text::new).map_err(|source| self.failed(source))
    }

    pub(crate) fn read_or(&self, absent: Text) -> Result<Text, Failure> {
        self.existing().map_or_else(|| Ok(absent), Journal::read)
    }

    pub(crate) fn existing(&self) -> Option<&Journal> {
        self.0.is_file().then_some(self)
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
