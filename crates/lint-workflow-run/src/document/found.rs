use std::path::Path;

use lint_core::{Site, Violation};
use lint_layout::LineNumber;

use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Found {
    line: LineNumber,
    finding: Finding,
}

impl Found {
    pub(crate) fn new(line: LineNumber, finding: Finding) -> Found {
        Found { line, finding }
    }

    pub(crate) fn violation(self, shown: &Path) -> Violation {
        Violation::new(Site::new(shown, self.line), self.finding)
    }
}
