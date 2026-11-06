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

    #[must_use]
    pub fn said(self) -> Option<Record> {
        match self {
            Line::Said(record) => Some(record),
            Line::Skipped(_) => None,
        }
    }

    #[must_use]
    pub fn skipped(self) -> Option<Skipped> {
        match self {
            Line::Said(_) => None,
            Line::Skipped(skipped) => Some(skipped),
        }
    }
}
