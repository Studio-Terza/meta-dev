use std::collections::BTreeSet;

use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Candidate<'file> {
    called: Name,
    violation: Violation<'file>,
}

impl<'file> Candidate<'file> {
    pub(crate) fn new(called: Name, violation: Violation<'file>) -> Candidate<'file> {
        Candidate { called, violation }
    }

    pub(crate) fn tested_only(self, names: &BTreeSet<Name>) -> Option<Violation<'file>> {
        names.get(&self.called).map(|_| self.violation)
    }
}
