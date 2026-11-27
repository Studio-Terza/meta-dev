use std::path::{Path, PathBuf};

use port_core::port;

use crate::{Bytes, FileFailure, Kind, Offset, Size, Stamps, Tail, Text};

#[port(laws)]
pub trait Files {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure>;

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure>;

    fn read_from(&self, path: &Path, offset: Offset) -> Result<Bytes, FileFailure>;

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure>;

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure>;

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure>;

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure>;

    fn kind(&self, path: &Path) -> Kind;

    fn size(&self, path: &Path) -> Result<Size, FileFailure>;

    fn stamps(&self, path: &Path) -> Result<Stamps, FileFailure>;
}

pub trait Batched: Files {
    fn entries_many(&self, directories: &[impl AsRef<Path>]) -> Vec<Result<Vec<PathBuf>, FileFailure>> {
        directories.iter().map(|directory| self.entries(directory.as_ref())).collect()
    }

    fn kinds(&self, paths: &[impl AsRef<Path>]) -> Vec<Kind> {
        paths.iter().map(|path| self.kind(path.as_ref())).collect()
    }

    fn sizes(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Size, FileFailure>> {
        paths.iter().map(|path| self.size(path.as_ref())).collect()
    }

    fn stamps_many(&self, paths: &[impl AsRef<Path>]) -> Vec<Result<Stamps, FileFailure>> {
        paths.iter().map(|path| self.stamps(path.as_ref())).collect()
    }

    fn read_tails(&self, tails: &[Tail]) -> Vec<Result<Bytes, FileFailure>> {
        tails.iter().map(|tail| self.read_from(tail.as_ref(), tail.offset())).collect()
    }
}

impl<Ported: Files> Batched for Ported {}
