use lint_layout::{LineNumber, UsePath};

#[derive(Debug)]
pub(crate) struct Spelled {
    path: UsePath,
    started_on: LineNumber,
}

impl Spelled {
    pub(crate) const fn new(path: UsePath, started_on: LineNumber) -> Self {
        Self { path, started_on }
    }

    pub(crate) fn handed<Made>(self, to: impl FnOnce(UsePath, LineNumber) -> Made) -> Made {
        to(self.path, self.started_on)
    }
}
