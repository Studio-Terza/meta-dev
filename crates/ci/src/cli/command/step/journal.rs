use std::fmt;
use std::io;
use std::mem;
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

    pub(crate) fn read_all(files: &impl Files, journals: &[Journal]) -> Vec<Result<Text, Failure>> {
        let paths: Vec<PathBuf> = journals.iter().map(|journal| journal.0.clone()).collect();
        journals
            .iter()
            .zip(files.read_text_many(&paths))
            .map(|(journal, text)| text.map_err(|failure| journal.failed(failure)))
            .collect()
    }

    pub(crate) fn read_groups(files: &impl Files, groups: &[Vec<Journal>]) -> Vec<Vec<Result<Text, Failure>>> {
        let read = Self::read_all(files, &groups.concat());
        groups
            .iter()
            .scan(read, |rest, group| {
                let later = rest.split_off(group.len());
                Some(mem::replace(rest, later))
            })
            .collect()
    }

    pub(crate) fn read_present(files: &impl Files, journals: &[Journal]) -> Vec<Option<Result<Text, Failure>>> {
        let paths: Vec<PathBuf> = journals.iter().map(|journal| journal.0.clone()).collect();
        let groups: Vec<Vec<Journal>> = journals
            .iter()
            .zip(files.kinds(&paths))
            .map(|(journal, kind)| match kind {
                Kind::File => vec![journal.clone()],
                Kind::Directory | Kind::Link | Kind::Missing => Vec::new(),
            })
            .collect();
        Self::read_groups(files, &groups).into_iter().map(|group| group.into_iter().next()).collect()
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
