use std::borrow::Cow;
use std::fmt;
use std::path::{Path, PathBuf};

use file_ports::{Bytes, FileFailure, Files, Kind};

use crate::cli::command::failure::Failure;
use crate::cli::ports::Text;

#[derive(Debug)]
pub struct Journal(PathBuf);

impl Journal {
    pub(crate) fn file(path: &Path) -> Journal {
        Journal(path.to_path_buf())
    }

    pub(crate) fn append(&self, files: &impl Files, entry: impl fmt::Display) -> Result<(), Failure<'_>> {
        self.prepared(files)
            .and_then(|()| files.append(&self.0, &Bytes::from(&entry)))
            .map_err(|failure| Failure::unusable(self, failure))
    }

    pub(crate) fn emptied(&self, files: &impl Files) -> Result<&Journal, Failure<'_>> {
        self.replace(files, &Text::default()).map(|()| self)
    }

    pub(crate) fn replace(&self, files: &impl Files, text: &Text) -> Result<(), Failure<'_>> {
        self.prepared(files)
            .and_then(|()| files.write(&self.0, Bytes::from(text.as_ref())))
            .map_err(|failure| Failure::unusable(self, failure))
    }

    pub(crate) fn read(&self, files: &impl Files) -> Result<Text, Failure<'_>> {
        files.read_text(&self.0).map_err(|failure| Failure::unusable(self, failure))
    }

    pub(crate) fn read_or(&self, files: &impl Files, absent: Text) -> Result<Text, Failure<'_>> {
        self.existing(files).map_or_else(|| Ok(absent), |journal| journal.read(files))
    }

    pub(crate) fn existing(&self, files: &impl Files) -> Option<&Journal> {
        (files.kind(&self.0) == Kind::File).then_some(self)
    }

    pub(crate) fn read_present<'path, Held>(
        files: &impl Files,
        journals: Vec<Held>,
    ) -> Vec<Option<Result<Text, Failure<'path>>>>
    where
        Held: AsRef<Path> + Into<Cow<'path, Path>>,
    {
        let kinds = files.kinds(&journals);
        let present: Vec<Held> = journals
            .into_iter()
            .zip(&kinds)
            .filter(|(_, kind)| **kind == Kind::File)
            .map(|(journal, _)| journal)
            .collect();
        let read = files.read_text_many(&present);
        let texts = present
            .into_iter()
            .zip(read)
            .map(|(journal, text)| text.map_err(|failure| Failure::unusable(journal, failure)));
        kinds
            .into_iter()
            .scan(texts, |remaining, kind| {
                Some(Some(kind).filter(|candidate| *candidate == Kind::File).and_then(|_| remaining.next()))
            })
            .collect()
    }

    fn prepared(&self, files: &impl Files) -> Result<(), FileFailure> {
        self.0.parent().map_or(Ok(()), |parent| files.create_directories(parent))
    }
}

impl fmt::Display for Journal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0.display())
    }
}

impl From<PathBuf> for Journal {
    fn from(path: PathBuf) -> Journal {
        Journal(path)
    }
}

impl AsRef<Path> for Journal {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl<'journal> From<&'journal Journal> for Cow<'journal, Path> {
    fn from(journal: &'journal Journal) -> Cow<'journal, Path> {
        Cow::Borrowed(&journal.0)
    }
}

impl From<Journal> for Cow<'_, Path> {
    fn from(journal: Journal) -> Self {
        Cow::Owned(journal.0)
    }
}
