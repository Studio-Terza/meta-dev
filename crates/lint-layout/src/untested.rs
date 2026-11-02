use std::path::{Path, PathBuf};

use newtype::Newtype;

#[derive(Newtype, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Untested(Vec<PathBuf>);

impl Untested {
    #[must_use]
    pub fn holds<'file>(&self, file: &'file Path) -> Option<&'file Path> {
        self.0.iter().any(|untested| untested == file).then_some(file)
    }
}
