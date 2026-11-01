use lint_layout::{LineNumber, UsePath};

#[derive(Debug)]
pub(crate) struct Spelled {
    path: UsePath,
    started_on: LineNumber,
}

impl Spelled {
    pub(crate) fn new(path: UsePath, started_on: LineNumber) -> Spelled {
        Spelled { path, started_on }
    }

    pub(crate) fn path(&self) -> &UsePath {
        &self.path
    }

    pub(crate) fn started_on(&self) -> LineNumber {
        self.started_on
    }
}
