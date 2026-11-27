use crate::transcript::{Record, Skipped, Unrecognised};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Skipped(Skipped),
    Unrecognised(Unrecognised),
}

impl Line {
    #[must_use]
    pub fn gathered<Into: From<Record> + From<Skipped> + From<Unrecognised>>(self) -> Into {
        match self {
            Line::Said(record) => Into::from(record),
            Line::Skipped(skipped) => Into::from(skipped),
            Line::Unrecognised(unrecognised) => Into::from(unrecognised),
        }
    }
}
