use std::path::{Path, PathBuf};

use lint_roots::Survey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child {
    path: PathBuf,
    children_directory: PathBuf,
}

impl Child {
    #[must_use]
    pub fn new(path: PathBuf, children_directory: PathBuf) -> Self {
        Self { path, children_directory }
    }

    #[must_use]
    pub fn canonical(self, survey: &Survey) -> Self {
        let path = survey.canonical(&self.path).map(Path::to_path_buf).unwrap_or(self.path);
        Self { path, ..self }
    }

    pub fn visit<Output>(self, then: impl FnOnce(&Path, &Path) -> Output) -> Output {
        then(&self.path, &self.children_directory)
    }
}

impl AsRef<Path> for Child {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
