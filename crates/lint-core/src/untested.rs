use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Untested(Vec<PathBuf>);

impl Untested {
    #[must_use]
    pub fn new(files: Vec<PathBuf>) -> Untested {
        Untested(files)
    }

    #[must_use]
    pub fn holds(&self, file: &Path) -> bool {
        self.0.iter().any(|untested| untested == file)
    }
}
