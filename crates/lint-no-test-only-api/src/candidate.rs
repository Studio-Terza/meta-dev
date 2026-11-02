use lint_core::Violation;
use lint_layout::Name;

#[derive(Debug)]
pub(crate) struct Candidate {
    called: Name,
    violation: Violation,
}

impl Candidate {
    pub(crate) fn new(called: Name, violation: Violation) -> Candidate {
        Candidate { called, violation }
    }

    pub(crate) fn called(&self) -> &Name {
        &self.called
    }

    pub(crate) fn violation(self) -> Violation {
        self.violation
    }
}
