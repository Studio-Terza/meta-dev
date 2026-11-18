use std::path::{Path, PathBuf};

use lint_roots::Survey;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Child {
    path: PathBuf,
    children_directory: PathBuf,
}

impl Child {
    #[must_use]
    pub fn new(path: PathBuf, children_directory: PathBuf) -> Child {
        Child { path, children_directory }
    }

    #[must_use]
    pub fn canonical<'child>(&'child self, survey: &'child Survey) -> &'child Path {
        survey.canonical(&self.path).unwrap_or(&self.path)
    }

    #[must_use]
    pub fn children(&self) -> &Path {
        &self.children_directory
    }
}

impl AsRef<Path> for Child {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}
