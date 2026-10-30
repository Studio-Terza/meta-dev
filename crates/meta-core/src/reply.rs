use crate::event::Event;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub struct Accepted {
    events: Vec<Event>,
    text: String,
}

impl Accepted {
    pub(crate) fn new(events: Vec<Event>, text: String) -> Self {
        Self { events, text }
    }

    pub fn into_parts(self) -> (Vec<Event>, String) {
        (self.events, self.text)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(String);

impl Refusal {
    pub(crate) fn because(why: impl Into<String>) -> Self {
        Self(why.into())
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "refused: {}", self.0)
    }
}
