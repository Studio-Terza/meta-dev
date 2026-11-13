use std::fmt;
use std::path::Path;

use lint_layout::LineNumber;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Site<'file> {
    path: &'file Path,
    line: LineNumber,
}

impl<'file> Site<'file> {
    #[must_use]
    pub fn new(path: &'file Path, line: LineNumber) -> Site<'file> {
        Site { path, line }
    }

    #[must_use]
    pub fn of_file(path: &'file Path) -> Site<'file> {
        Site::new(path, LineNumber::FIRST)
    }

    #[must_use]
    pub fn path(&self) -> &'file Path {
        self.path
    }

    #[must_use]
    pub fn moved(self, path: &Path) -> Site<'_> {
        Site { path, line: self.line }
    }
}

impl fmt::Display for Site<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.path.display(), self.line)
    }
}
