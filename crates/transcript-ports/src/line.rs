use crate::{Record, Skipped};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Skipped(Skipped),
}

pub trait Gather {
    #[must_use]
    fn said(self, record: Record) -> Self;

    #[must_use]
    fn skipped(self, skipped: Skipped) -> Self;
}

impl Line {
    #[must_use]
    pub fn gathered<Into: Gather>(self, into: Into) -> Into {
        match self {
            Line::Said(record) => into.said(record),
            Line::Skipped(skipped) => into.skipped(skipped),
        }
    }
}
