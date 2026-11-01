use std::fmt;
use std::path::{Path, PathBuf};

use lint_layout::LineNumber;

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
    pub fn of_file(path: &Path) -> Site {
        Site::new(path, LineNumber::FIRST)
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl fmt::Display for Site {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}:{}", self.path.display(), self.line)
    }
}
