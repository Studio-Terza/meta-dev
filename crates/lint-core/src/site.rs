use std::fmt;
use std::path::{Path, PathBuf};

use crate::line_number::LineNumber;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Site {
    path: PathBuf,
    line: LineNumber,
}

impl Site {
    #[must_use]
    pub fn new(path: &Path, line: LineNumber) -> Site {
        Site { path: path.to_path_buf(), line }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub fn line(&self) -> LineNumber {
        self.line
    }
}

impl fmt::Display for Site {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.path.display(), self.line)
    }
}
