use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Target {
    package: PathBuf,
    files: Vec<PathBuf>,
}

impl Target {
    #[must_use]
    pub const fn new(package: PathBuf, files: Vec<PathBuf>) -> Self {
        Self { package, files }
    }

    #[must_use]
    pub fn package(&self) -> &Path {
        &self.package
    }

    #[must_use]
    pub fn files(&self) -> &[PathBuf] {
        &self.files
    }
}
