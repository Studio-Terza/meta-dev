use crate::{Event, Record, Skipped, Unrecognised};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Line {
    Said(Record),
    Happened(Event),
    Skipped(Skipped),
    Unrecognised(Unrecognised),
}

impl Line {
    #[must_use]
    pub fn gathered<Into: From<Record> + From<Event> + From<Skipped> + From<Unrecognised>>(self) -> Into {
        match self {
            Line::Said(record) => Into::from(record),
            Line::Happened(event) => Into::from(event),
            Line::Skipped(skipped) => Into::from(skipped),
            Line::Unrecognised(unrecognised) => Into::from(unrecognised),
        }
    }
}
