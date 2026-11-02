use std::path::{Path, PathBuf};

use newtype::{Arbitrary, Newtype};

#[derive(Newtype, Arbitrary, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Untested(Vec<PathBuf>);

impl Untested {
    #[must_use]
    pub fn holds(&self, file: &Path) -> bool {
        self.0.iter().any(|untested| untested == file)
    }
}
