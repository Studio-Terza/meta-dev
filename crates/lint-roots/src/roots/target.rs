use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Target {
    package: PathBuf,
    files: Vec<PathBuf>,
}

impl Target {
    #[must_use]
    pub fn new(package: PathBuf, files: Vec<PathBuf>) -> Target {
        Target { package, files }
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
