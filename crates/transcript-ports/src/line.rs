use crate::{Record, Skipped, Unrecognised};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Skipped(Skipped),
    Unrecognised(Unrecognised),
}

pub trait Gather {
    #[must_use]
    fn said(self, record: Record) -> Self;

    #[must_use]
    fn skipped(self, skipped: Skipped) -> Self;

    #[must_use]
    fn unrecognised(self, unrecognised: Unrecognised) -> Self;
}

impl Line {
    #[must_use]
    pub fn gathered<Into: Gather>(self, into: Into) -> Into {
        match self {
            Line::Said(record) => into.said(record),
            Line::Skipped(skipped) => into.skipped(skipped),
            Line::Unrecognised(unrecognised) => into.unrecognised(unrecognised),
        }
    }
}
