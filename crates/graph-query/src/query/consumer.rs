use std::path::{Path, PathBuf};

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, PartialEq, Eq)]
pub(crate) struct Consumer(PathBuf);

impl AsRef<Path> for Consumer {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
