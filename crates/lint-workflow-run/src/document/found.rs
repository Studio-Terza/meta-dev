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
    pub(crate) const fn new(line: LineNumber, finding: Finding) -> Self {
        Self { line, finding }
    }

    pub(crate) fn violation(self, shown: &Path) -> Violation<'_> {
        Violation::new(Site::new(shown, self.line), self.finding)
    }
}
