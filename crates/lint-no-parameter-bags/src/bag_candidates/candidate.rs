mod finding;

use std::path::Path;

use lint_core::{Site, Violation};
use lint_layout::LineNumber;
use lint_layout::Name;

use finding::Finding;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Candidate<'name> {
    name: &'name Name,
    line: LineNumber,
}

impl<'name> Candidate<'name> {
    pub fn new(name: &'name Name, line: LineNumber) -> Self {
        Candidate { name, line }
    }

    pub fn name(&self) -> &'name Name {
        self.name
    }

    pub fn violation<'file>(&self, path: &'file Path) -> Violation<'file> {
        Violation::new(Site::new(path, self.line), Finding::ParameterBag { name: self.name })
    }
}
