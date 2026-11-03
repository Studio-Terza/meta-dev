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

    pub(crate) fn called(&self) -> &Name {
        &self.called
    }

    pub(crate) fn violation(self) -> Violation<'file> {
        self.violation
    }
}
