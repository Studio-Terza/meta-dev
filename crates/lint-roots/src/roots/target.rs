use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct Target {
    package: PathBuf,
    file: PathBuf,
}

impl Target {
    #[must_use]
    pub fn new(package: PathBuf, file: PathBuf) -> Target {
        Target { package, file }
    }

    #[must_use]
    pub fn package(&self) -> &Path {
        &self.package
    }

    #[must_use]
    pub fn into_file(self) -> PathBuf {
        self.file
    }
}
