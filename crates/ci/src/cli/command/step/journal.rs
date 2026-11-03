use std::fmt;
use std::io;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, FileFailure, Files, Kind};

use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

#[derive(Clone, Debug)]
pub struct Journal(PathBuf);

impl Journal {
    pub(crate) fn file(path: &Path) -> Journal {
        Journal(path.to_path_buf())
    }

    pub(crate) fn append(&self, files: &impl Files, entry: impl fmt::Display) -> Result<(), Failure> {
        self.prepared(files)
            .and_then(|()| files.append(&self.0, &Bytes::from(&entry)))
            .map_err(|failure| self.failed(failure))
    }

    pub(crate) fn emptied(&self, files: &impl Files) -> Result<Journal, Failure> {
        self.replace(files, &Text::default()).map(|()| self.clone())
    }

    pub(crate) fn replace(&self, files: &impl Files, text: &Text) -> Result<(), Failure> {
        self.prepared(files)
            .and_then(|()| files.write(&self.0, &Bytes::from(text.as_ref())))
            .map_err(|failure| self.failed(failure))
    }

    pub(crate) fn read(&self, files: &impl Files) -> Result<Text, Failure> {
        files.read_text(&self.0).map_err(|failure| self.failed(failure))
    }

    pub(crate) fn read_or(&self, files: &impl Files, absent: Text) -> Result<Text, Failure> {
        self.existing(files).map_or_else(|| Ok(absent), |journal| journal.read(files))
    }

    pub(crate) fn existing(&self, files: &impl Files) -> Option<&Journal> {
        (files.kind(&self.0) == Kind::File).then_some(self)
    }

    fn prepared(&self, files: &impl Files) -> Result<(), FileFailure> {
        self.0.parent().map_or(Ok(()), |parent| files.create_directories(parent))
    }

    fn failed(&self, failure: FileFailure) -> Failure {
        Failure::Io { path: self.0.clone(), source: io::Error::from(failure) }
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
