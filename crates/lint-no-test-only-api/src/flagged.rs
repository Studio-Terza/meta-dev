use std::path::Path;

use lint_core::{LineNumber, Site, Violation};

use crate::finding::Finding;

#[derive(Debug)]
pub(crate) struct Flagged {
    line: LineNumber,
    finding: Finding,
}

impl Flagged {
    pub(crate) fn new(line: LineNumber, finding: Finding) -> Flagged {
        Flagged { line, finding }
    }

    pub(crate) fn in_file(self, path: &Path) -> Violation {
        Violation::new(Site::new(path, self.line), self.finding)
    }
}
