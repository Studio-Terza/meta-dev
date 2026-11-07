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
    pub const fn new(path: &'file Path, line: LineNumber) -> Self {
        Site { path, line }
    }

    #[must_use]
    pub const fn of_file(path: &'file Path) -> Self {
        Site::new(path, LineNumber::FIRST)
    }

    #[must_use]
    pub const fn path(&self) -> &'file Path {
        self.path
    }
}

impl fmt::Display for Site<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.path.display(), self.line)
    }
}
