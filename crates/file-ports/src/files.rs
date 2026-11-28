use std::path::{Path, PathBuf};

use invariants::Ignoring;

use crate::ignored::Ignored;
use crate::{Bytes, FileFailure, Kind, Offset, Size, Stamps, Text};

pub trait Files {
    fn read(&self, path: &Path) -> Result<Bytes, FileFailure>;

    fn read_text(&self, path: &Path) -> Result<Text, FileFailure>;

    fn read_from(&self, path: &Path, offset: Offset) -> Result<Bytes, FileFailure> {
        let skipped = usize::try_from(offset.position()).ignoring(Ignored::BeyondRange).unwrap_or(usize::MAX);
        self.read(path).map(|bytes| Bytes::new(bytes.as_ref().iter().skip(skipped).copied().collect()))
    }

    fn write(&self, path: &Path, bytes: Bytes) -> Result<(), FileFailure>;

    fn append(&self, path: &Path, bytes: &Bytes) -> Result<(), FileFailure>;

    fn create_directories(&self, path: &Path) -> Result<(), FileFailure>;

    fn copy(&self, from: &Path, to: &Path) -> Result<(), FileFailure>;

    fn entries(&self, directory: &Path) -> Result<Vec<PathBuf>, FileFailure>;

    fn canonical(&self, path: &Path) -> Result<PathBuf, FileFailure>;

    fn kind(&self, path: &Path) -> Kind;

    fn size(&self, path: &Path) -> Result<Size, FileFailure>;

    fn stamps(&self, path: &Path) -> Result<Stamps, FileFailure>;
}
