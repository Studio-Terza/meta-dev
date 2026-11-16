use std::path::Path;

use syntax::Ident;

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
    owner: Name,
}

#[derive(Debug)]
pub struct Owned {
    callee: Callee,
    owner: Name,
}

impl Owned {
    pub fn new(callee: Callee, owner: &Name) -> Owned {
        Owned { callee, owner: owner.clone() }
    }

    pub fn forwarder(self, name: &Ident) -> Forwarder {
        Forwarder {
            name: Name::from(name),
            line: LineNumber::from(name.span()),
            callee: self.callee,
            owner: self.owner,
        }
    }
}

impl Forwarder {
    pub fn owner(&self) -> &Name {
        &self.owner
    }

    pub fn violation(self, path: &Path) -> Violation<'_> {
        Violation::new(Site::new(path, self.line), Finding::Forwards { name: self.name, callee: self.callee })
    }
}
