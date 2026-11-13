use std::path::Path;

use syn::Ident;

mod finding;

use super::callee::Callee;
use finding::Finding;
use lint_core::{Site, Violation};
use lint_layout::{LineNumber, Name};

#[derive(Debug)]
pub struct Forwarder {
    name: Name,
    line: LineNumber,
    callee: Callee,
}

impl Forwarder {
    pub fn new(name: &Ident, callee: Callee) -> Forwarder {
        Forwarder { name: Name::from(name), line: LineNumber::from(name.span()), callee }
    }

    pub fn violation(self, path: &Path) -> Violation<'_> {
        Violation::new(Site::new(path, self.line), Finding::Forwards { name: self.name, callee: self.callee })
    }
}
