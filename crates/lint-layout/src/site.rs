use std::fmt;
use std::path::Path;

use crate::line_number::LineNumber;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InFile<'file, Value> {
    path: &'file Path,
    value: Value,
}

pub type Site<'file> = InFile<'file, LineNumber>;

impl<'file, Value> InFile<'file, Value> {
    #[must_use]
    pub fn new(path: &'file Path, value: Value) -> InFile<'file, Value> {
        InFile { path, value }
    }

    #[must_use]
    pub fn path(&self) -> &'file Path {
        self.path
    }

    #[must_use]
    pub fn moved(self, path: &Path) -> InFile<'_, Value> {
        InFile { path, value: self.value }
    }
}

impl<'file> InFile<'file, LineNumber> {
    #[must_use]
    pub fn of_file(path: &'file Path) -> Site<'file> {
        Site::new(path, LineNumber::FIRST)
    }
}

impl<Value> AsRef<Value> for InFile<'_, Value> {
    fn as_ref(&self) -> &Value {
        &self.value
    }
}

impl fmt::Display for InFile<'_, LineNumber> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.path.display(), self.value)
    }
}
